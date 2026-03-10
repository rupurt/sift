use std::fs;
use std::path::Path;
use std::sync::Mutex;

use anyhow::{Result, anyhow, bail};
use candle_core::{Device, Tensor, DType};
use candle_nn::VarBuilder;
use candle_transformers::models::qwen2::{Config as QwenConfig, ModelForCausalLM as QwenModel};
use serde::Deserialize;
use tokenizers::Tokenizer;

use crate::cache::cache_dir;
use crate::search::domain::{CandidateList, Reranker};

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

#[derive(Debug, Deserialize)]
struct QwenConfigPartial {
    vocab_size: usize,
    hidden_size: usize,
    intermediate_size: usize,
    num_hidden_layers: usize,
    num_attention_heads: usize,
    num_key_value_heads: usize,
    max_position_embeddings: usize,
    rms_norm_eps: f64,
    rope_theta: f64,
    hidden_act: String,
    sliding_window: Option<usize>,
}

pub struct QwenReranker {
    tokenizer: Tokenizer,
    model: Mutex<QwenModel>,
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

        ensure_qwen_asset(&spec, &config_path, "config.json")?;
        ensure_qwen_asset(&spec, &tokenizer_path, "tokenizer.json")?;
        ensure_qwen_asset(&spec, &weights_path, "model.safetensors")?;

        let config_partial: QwenConfigPartial = serde_json::from_str(&fs::read_to_string(&config_path)?)?;
        
        let config = QwenConfig {
            vocab_size: config_partial.vocab_size,
            hidden_size: config_partial.hidden_size,
            intermediate_size: config_partial.intermediate_size,
            num_hidden_layers: config_partial.num_hidden_layers,
            num_attention_heads: config_partial.num_attention_heads,
            num_key_value_heads: config_partial.num_key_value_heads,
            hidden_act: match config_partial.hidden_act.as_str() {
                "silu" => candle_nn::Activation::Silu,
                _ => bail!("unsupported activation: {}", config_partial.hidden_act),
            },
            max_position_embeddings: config_partial.max_position_embeddings,
            rms_norm_eps: config_partial.rms_norm_eps,
            rope_theta: config_partial.rope_theta,
            use_sliding_window: config_partial.sliding_window.is_some(),
            sliding_window: config_partial.sliding_window.unwrap_or(config_partial.max_position_embeddings),
            max_window_layers: config_partial.num_hidden_layers,
            tie_word_embeddings: true,
        };

        let tokenizer = Tokenizer::from_file(&tokenizer_path)
            .map_err(|m| anyhow!("failed to load tokenizer: {}", m))?;
        
        let device = Device::Cpu;
        let vb = unsafe { VarBuilder::from_mmaped_safetensors(&[weights_path], DType::F32, &device)? };
        let model = QwenModel::new(&config, vb)?;

        Ok(Self {
            tokenizer,
            model: Mutex::new(model),
            device,
        })
    }

    pub fn score_pair(&self, query: &str, filename: &str, document: &str) -> Result<f64> {
        let prompt = format!(
            "<|im_start|>system\nYou are a technical search expert. Evaluate if the document snippet provides the logic, implementation, or test case for the user's query.\n\nFocus on matching the logical intent, not just common words like 'test' or 'config'.<|im_end|>\n<|im_start|>user\nQuery: {}\nFile: {}\nSnippet: {}\nIs this document a strong match for the query logic? Answer with only 'Yes' or 'No'.<|im_end|>\n<|im_start|>assistant\n",
            query, filename, document
        );
        
        let encoding = self.tokenizer.encode(prompt, true)
            .map_err(|m| anyhow!("encoding failed: {}", m))?;
        
        let tokens = encoding.get_ids();
        let tokens_tensor = Tensor::new(tokens, &self.device)?.unsqueeze(0)?;
        
        let mut model = self.model.lock().map_err(|_| anyhow!("model mutex poisoned"))?;
        
        // ModelForCausalLM handles slicing hidden states internally. 
        // It returns [batch, 1, vocab_size] for the LAST token.
        let logits = model.forward(&tokens_tensor, 0)?;
        
        let last_logit = logits.get(0)?.get(0)?;
        let probs = candle_nn::ops::softmax(&last_logit, 0)?;
        let probs_vec = probs.to_vec1::<f32>()?;

        let yes_id = self.tokenizer.token_to_id("Yes").unwrap_or(5510) as usize;
        let no_id = self.tokenizer.token_to_id("No").unwrap_or(3465) as usize;

        let yes_prob = probs_vec.get(yes_id).cloned().unwrap_or(0.0) as f64;
        let no_prob = probs_vec.get(no_id).cloned().unwrap_or(0.0) as f64;

        let score = yes_prob / (yes_prob + no_prob + 1e-6);
        
        // Ensure we don't carry over KV cache between documents
        model.clear_kv_cache();
        
        Ok(score)
    }
}

impl Reranker for QwenReranker {
    fn rerank(&self, query: &str, mut candidates: CandidateList, limit: usize) -> Result<CandidateList> {
        if candidates.results.is_empty() {
            return Ok(candidates);
        }

        tracing::info!("reranking {} candidates with Qwen2.5-0.5B...", candidates.results.len());
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
}

fn ensure_qwen_asset(spec: &QwenModelSpec, path: &Path, filename: &str) -> Result<()> {
    if path.exists() {
        return Ok(());
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let url = format!(
        "https://huggingface.co/{}/resolve/{}/{}",
        spec.model_id, spec.revision, filename
    );
    
    tracing::info!("downloading {}...", url);
    let mut response = ureq::get(&url).call()?;
    let mut file = fs::File::create(path)?;
    let mut reader = response.body_mut().as_reader();
    std::io::copy(&mut reader, &mut file)?;
    
    Ok(())
}
