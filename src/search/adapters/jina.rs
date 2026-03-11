use std::fs;
use std::path::Path;

use anyhow::{Result, anyhow};
use candle_core::{DType, Device, Module, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::qwen3::{Config as QwenConfig, Model as QwenModel};
use tokenizers::Tokenizer;

use super::llm_utils::{QwenConfigPartial, ensure_hf_asset, qwen_generate};
use crate::cache::cache_dir;
use crate::search::domain::{CandidateList, GenerativeModel, Reranker};

pub const DEFAULT_JINA_MODEL_ID: &str = "jinaai/jina-reranker-v3";
pub const DEFAULT_JINA_REVISION: &str = "main";
pub const DEFAULT_JINA_MAX_LENGTH: usize = 512;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JinaModelSpec {
    pub model_id: String,
    pub revision: String,
    pub max_length: usize,
}

impl Default for JinaModelSpec {
    fn default() -> Self {
        Self {
            model_id: DEFAULT_JINA_MODEL_ID.to_string(),
            revision: DEFAULT_JINA_REVISION.to_string(),
            max_length: DEFAULT_JINA_MAX_LENGTH,
        }
    }
}

pub struct Projector {
    linear1: candle_nn::Linear,
    linear2: candle_nn::Linear,
}

impl Projector {
    pub fn new(vb: VarBuilder) -> Result<Self> {
        let linear1 = candle_nn::linear_no_bias(1024, 512, vb.pp("0"))?;
        let linear2 = candle_nn::linear_no_bias(512, 512, vb.pp("2"))?;
        Ok(Self { linear1, linear2 })
    }

    pub fn forward(&self, x: &Tensor) -> Result<Tensor> {
        let x = self.linear1.forward(x)?;
        let x = x.relu()?;
        let x = self.linear2.forward(&x)?;
        // Normalize for cosine similarity
        let norm = x.sqr()?.sum_keepdim(1)?.sqrt()?;
        Ok(x.broadcast_div(&norm)?)
    }
}

pub struct JinaReranker {
    pub model_id: String,
    pub revision: String,
    pub max_length: usize,
    tokenizer: Tokenizer,
    config: QwenConfig,
    vb: VarBuilder<'static>,
    projector: Projector,
    device: Device,
}

impl JinaReranker {
    pub fn load(spec: JinaModelSpec) -> Result<Self> {
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

        let device = Device::Cpu;
        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(
                std::slice::from_ref(&weights_path),
                DType::F32,
                &device,
            )?
        };
        let projector = Projector::new(vb.pp("projector"))?;

        Ok(Self {
            model_id: spec.model_id,
            revision: spec.revision,
            max_length: spec.max_length,
            tokenizer,
            config,
            vb,
            projector,
            device,
        })
    }

    pub fn score_pair(&self, query: &str, document: &str) -> Result<f64> {
        let q_token = "<|rerank_token|>";
        let d_token = "<|embed_token|>";

        let prompt = format!("{} {} {} {}", query, q_token, document, d_token);

        let encoding = self
            .tokenizer
            .encode(prompt, true)
            .map_err(|m| anyhow!("encoding failed: {}", m))?;

        let tokens = encoding.get_ids();
        let tokens_tensor = Tensor::new(tokens, &self.device)?.unsqueeze(0)?;

        let q_id = self
            .tokenizer
            .token_to_id(q_token)
            .ok_or_else(|| anyhow!("missing q_token in vocab"))?;
        let d_id = self
            .tokenizer
            .token_to_id(d_token)
            .ok_or_else(|| anyhow!("missing d_token in vocab"))?;

        let q_pos = tokens
            .iter()
            .position(|&id| id == q_id)
            .ok_or_else(|| anyhow!("missing query token in sequence"))?;
        let d_pos = tokens
            .iter()
            .position(|&id| id == d_id)
            .ok_or_else(|| anyhow!("missing doc token in sequence"))?;

        // Recreate the Model structure to ensure a fresh KV cache.
        // This is fast as weights are already in memory via VarBuilder.
        let mut model = QwenModel::new(&self.config, self.vb.clone())?;

        let hidden_states = model.forward(&tokens_tensor, 0)?;

        // hidden_states: [batch, seq_len, hidden_size]
        let q_hidden = hidden_states.narrow(1, q_pos, 1)?.squeeze(1)?;
        let d_hidden = hidden_states.narrow(1, d_pos, 1)?.squeeze(1)?;

        let q_emb = self.projector.forward(&q_hidden)?;
        let d_emb = self.projector.forward(&d_hidden)?;

        // Dot product of normalized embeddings = cosine similarity
        let score = (q_emb * d_emb)?.sum_all()?.to_scalar::<f32>()? as f64;

        Ok(score)
    }
}

impl GenerativeModel for JinaReranker {
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
}

impl Reranker for JinaReranker {
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
            "reranking {} candidates with Jina Reranker v3...",
            candidates.results.len()
        );
        let start = std::time::Instant::now();

        for candidate in &mut candidates.results {
            let snippet = candidate.snippet.as_deref().unwrap_or("");
            if !snippet.is_empty() {
                candidate.score = self.score_pair(query, snippet)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Requires model download and HF_TOKEN
    fn test_jina_load() {
        let spec = JinaModelSpec::default();
        let res = JinaReranker::load(spec);
        assert!(res.is_ok(), "Failed to load Jina model: {:?}", res.err());
    }
}
