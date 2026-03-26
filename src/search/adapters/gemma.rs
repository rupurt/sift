use std::fs;
use std::path::Path;

use anyhow::{Result, anyhow};
use candle_core::{DType, Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::gemma3::{Config as GemmaConfig, Model as GemmaModel};
use tokenizers::Tokenizer;

use super::llm_utils::{Gemma3ConfigPartial, ensure_hf_asset, load_mmaped_safetensors_with_repair};
use crate::cache::cache_dir;
use crate::search::domain::{CandidateList, GenerativeModel, Reranker};

pub const DEFAULT_GEMMA_MODEL_ID: &str = "google/gemma-3-1b-it";
pub const DEFAULT_GEMMA_REVISION: &str = "main";
pub const DEFAULT_GEMMA_MAX_LENGTH: usize = 512;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GemmaModelSpec {
    pub model_id: String,
    pub revision: String,
    pub max_length: usize,
}

impl Default for GemmaModelSpec {
    fn default() -> Self {
        Self {
            model_id: DEFAULT_GEMMA_MODEL_ID.to_string(),
            revision: DEFAULT_GEMMA_REVISION.to_string(),
            max_length: DEFAULT_GEMMA_MAX_LENGTH,
        }
    }
}

pub struct GemmaReranker {
    pub model_id: String,
    pub revision: String,
    pub max_length: usize,
    tokenizer: Tokenizer,
    config: GemmaConfig,
    vb: VarBuilder<'static>,
    device: Device,
}

impl GemmaReranker {
    pub fn load(spec: GemmaModelSpec) -> Result<Self> {
        let cache_root = cache_dir("models")?;
        let root = cache_root
            .join(Path::new(&spec.model_id))
            .join(Path::new(&spec.revision));

        let config_path = root.join("config.json");
        let tokenizer_path = root.join("tokenizer.json");
        let weights_path = root.join("model.safetensors");

        ensure_hf_asset(&spec.model_id, &spec.revision, &config_path, "config.json")?;
        ensure_hf_asset(
            &spec.model_id,
            &spec.revision,
            &tokenizer_path,
            "tokenizer.json",
        )?;
        ensure_hf_asset(
            &spec.model_id,
            &spec.revision,
            &weights_path,
            "model.safetensors",
        )?;

        let config_json = fs::read_to_string(&config_path)?;
        let config_partial: Gemma3ConfigPartial = serde_json::from_str(&config_json)?;
        let config = config_partial.into_config()?;

        let tokenizer = Tokenizer::from_file(&tokenizer_path)
            .map_err(|m| anyhow!("failed to load tokenizer: {}", m))?;

        let device = Device::Cpu;
        let vb = load_mmaped_safetensors_with_repair(
            &spec.model_id,
            &spec.revision,
            &weights_path,
            DType::F32,
            &device,
        )?;

        Ok(Self {
            model_id: spec.model_id,
            revision: spec.revision,
            max_length: spec.max_length,
            tokenizer,
            config,
            vb,
            device,
        })
    }

    pub fn score_pair(&self, query: &str, filename: &str, document: &str) -> Result<f64> {
        // Gemma 3 Instruct prompt format
        let prompt = format!(
            "<start_of_turn>user
You are a technical search expert. Evaluate if the document snippet provides the logic, implementation, or test case for the user's query.

Focus on matching the logical intent, not just common words like 'test' or 'config'.

Query: {}
File: {}
Snippet: {}
Is this document a strong match for the query logic? Answer with only 'Yes' or 'No'.<end_of_turn>
<start_of_turn>model
",
            query, filename, document
        );

        let encoding = self
            .tokenizer
            .encode(prompt, true)
            .map_err(|m| anyhow!("encoding failed: {}", m))?;

        let tokens = encoding.get_ids();
        let tokens_tensor = Tensor::new(tokens, &self.device)?.unsqueeze(0)?;

        let mut model = GemmaModel::new(false, &self.config, self.vb.clone())?;

        let hidden_states = model.forward(&tokens_tensor, 0)?;
        let last_hidden = hidden_states.narrow(1, tokens.len() - 1, 1)?;

        let lm_head = candle_nn::linear_no_bias(
            self.config.hidden_size,
            self.config.vocab_size,
            self.vb.pp("model.embed_tokens"), // Gemma often ties weights or uses embed_tokens for head
        )?;

        let logits = last_hidden.apply(&lm_head)?;

        let last_logit = logits.get(0)?.get(0)?;
        let probs = candle_nn::ops::softmax(&last_logit, 0)?;
        let probs_vec = probs.to_vec1::<f32>()?;

        let yes_id = self.tokenizer.token_to_id("Yes").unwrap_or(5510) as usize;
        let no_id = self.tokenizer.token_to_id("No").unwrap_or(3465) as usize;

        let yes_prob = probs_vec.get(yes_id).cloned().unwrap_or(0.0) as f64;
        let no_prob = probs_vec.get(no_id).cloned().unwrap_or(0.0) as f64;

        let score = yes_prob / (yes_prob + no_prob + 1e-6);

        Ok(score)
    }
}

impl Reranker for GemmaReranker {
    fn rerank(
        &self,
        query: &str,
        mut candidates: CandidateList,
        limit: usize,
    ) -> Result<CandidateList> {
        if candidates.results.is_empty() {
            return Ok(candidates);
        }

        tracing::info!(
            "reranking {} candidates with Gemma 3...",
            candidates.results.len()
        );
        let start = std::time::Instant::now();

        for candidate in &mut candidates.results {
            let snippet = candidate.snippet.as_deref().unwrap_or("");
            if !snippet.is_empty() {
                candidate.score =
                    self.score_pair(query, &candidate.path.display().to_string(), snippet)?;
            }
        }

        candidates.results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.id.cmp(&b.id))
        });

        tracing::info!("reranking complete in {:.2?}", start.elapsed());

        Ok(CandidateList {
            results: candidates.results.into_iter().take(limit).collect(),
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_generative(&self) -> Option<&dyn GenerativeModel> {
        None // For now only reranking is requested
    }
}
