use std::fs;
use std::path::Path;

use anyhow::{Result, anyhow};
use candle_core::{DType, Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::qwen2::{Config as QwenConfig, Model as QwenModel};
use tokenizers::Tokenizer;

use super::llm_utils::{
    QwenConfigPartial, ensure_hf_asset, load_mmaped_safetensors_with_repair, qwen_generate,
};
use crate::cache::cache_dir;
use crate::search::domain::{CandidateList, GenerativeModel, Reranker};

pub const DEFAULT_QWEN_MODEL_ID: &str = "Qwen/Qwen2.5-0.5B-Instruct";
pub const DEFAULT_QWEN_REVISION: &str = "main";
pub const DEFAULT_QWEN_MAX_LENGTH: usize = 512;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QwenModelSpec {
    pub model_id: String,
    pub revision: String,
    pub max_length: usize,
}

impl Default for QwenModelSpec {
    fn default() -> Self {
        Self {
            model_id: DEFAULT_QWEN_MODEL_ID.to_string(),
            revision: DEFAULT_QWEN_REVISION.to_string(),
            max_length: DEFAULT_QWEN_MAX_LENGTH,
        }
    }
}

pub struct QwenReranker {
    pub model_id: String,
    pub revision: String,
    pub max_length: usize,
    tokenizer: Tokenizer,
    config: QwenConfig,
    vb: VarBuilder<'static>,
    device: Device,
}

impl QwenReranker {
    pub fn load(spec: QwenModelSpec) -> Result<Self> {
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

        let config_partial: QwenConfigPartial =
            serde_json::from_str(&fs::read_to_string(&config_path)?)?;
        let config = config_partial.into_config()?;

        let tokenizer = Tokenizer::from_file(&tokenizer_path)
            .map_err(|m| anyhow!("failed to load tokenizer: {}", m))?;

        let device = super::llm_utils::get_device()?;
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
        let prompt = format!(
            "<|im_start|>system\nYou are a technical search expert. Evaluate if the document snippet provides the logic, implementation, or test case for the user's query.\n\nFocus on matching the logical intent, not just common words like 'test' or 'config'.<|im_end|>\n<|im_start|>user\nQuery: {}\nFile: {}\nSnippet: {}\nIs this document a strong match for the query logic? Answer with only 'Yes' or 'No'.<|im_end|>\n<|im_start|>assistant\n",
            query, filename, document
        );

        let encoding = self
            .tokenizer
            .encode(prompt, true)
            .map_err(|m| anyhow!("encoding failed: {}", m))?;

        let tokens = encoding.get_ids();
        let tokens_tensor = Tensor::new(tokens, &self.device)?.unsqueeze(0)?;

        let mut model = QwenModel::new(&self.config, self.vb.clone())?;

        let hidden_states = model.forward(&tokens_tensor, 0, None)?;
        let last_hidden = hidden_states.narrow(1, tokens.len() - 1, 1)?;

        let lm_head = if self.config.tie_word_embeddings {
            candle_nn::Linear::new(
                self.vb
                    .pp("model.embed_tokens")
                    .get((self.config.vocab_size, self.config.hidden_size), "weight")?,
                None,
            )
        } else {
            candle_nn::linear_no_bias(
                self.config.hidden_size,
                self.config.vocab_size,
                self.vb.pp("lm_head"),
            )?
        };

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

impl GenerativeModel for QwenReranker {
    fn generate(&self, prompt: &str, max_tokens: usize) -> Result<String> {
        qwen_generate(
            prompt,
            max_tokens,
            &self.config,
            &self.vb,
            &self.tokenizer,
            &self.device,
        )
    }

    fn start_conversation(&self) -> Result<Box<dyn Conversation>> {
        let mut session =
            super::llm_utils::QwenModelSession::new(&self.config, &self.vb, &self.device)?;

        let system_prompt = "<|im_start|>system\nYou are Paddles, a helpful AI assistant and mech suit operator. You provide concise and accurate technical advice.<|im_end|>\n";
        session.generate(system_prompt, 0, &self.tokenizer)?;

        Ok(Box::new(QwenConversation {
            session,
            tokenizer: self.tokenizer.clone(),
            history: Vec::new(),
        }))
    }
}

pub struct QwenConversation {
    session: super::llm_utils::QwenModelSession,
    tokenizer: Tokenizer,
    history: Vec<String>,
}

use crate::search::domain::Conversation;

impl Conversation for QwenConversation {
    fn send(&mut self, message: &str, max_tokens: usize) -> Result<String> {
        self.history.push(message.to_string());
        let prompted = format!(
            "<|im_start|>user\n{}<|im_end|>\n<|im_start|>assistant\n",
            message
        );
        let response = self
            .session
            .generate(&prompted, max_tokens, &self.tokenizer)?;
        self.history.push(response.clone());
        Ok(response)
    }

    fn history(&self) -> &[String] {
        &self.history
    }
}

impl Reranker for QwenReranker {
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
            "reranking {} candidates with Qwen2.5-0.5B...",
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
        Some(self)
    }
}
