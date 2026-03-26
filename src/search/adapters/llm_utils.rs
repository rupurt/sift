use std::fs;
use std::path::Path;

use anyhow::{Result, anyhow, bail};
use candle_core::{Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::qwen2::{Config as QwenConfig, Model as QwenModel};
use serde::Deserialize;
use tokenizers::Tokenizer;

#[derive(Debug, Deserialize)]
pub struct QwenConfigPartial {
    pub vocab_size: usize,
    pub hidden_size: usize,
    pub intermediate_size: usize,
    pub num_hidden_layers: usize,
    pub num_attention_heads: usize,
    pub num_key_value_heads: usize,
    pub max_position_embeddings: usize,
    pub rms_norm_eps: f64,
    pub rope_theta: f64,
    pub hidden_act: String,
    pub sliding_window: Option<usize>,
    pub head_dim: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct Gemma3ConfigPartial {
    pub vocab_size: usize,
    pub hidden_size: usize,
    pub intermediate_size: usize,
    pub num_hidden_layers: usize,
    pub num_attention_heads: usize,
    pub num_key_value_heads: usize,
    pub head_dim: usize,
    pub rms_norm_eps: f64,
    pub rope_theta: f64,
    pub max_position_embeddings: usize,
    pub attention_bias: bool,
    pub attn_logit_softcapping: Option<f64>,
    pub final_logit_softcapping: Option<f64>,
    pub hidden_activation: Option<String>,
    pub query_pre_attn_scalar: Option<usize>,
    pub rope_local_base_freq: f64,
    pub sliding_window: Option<usize>,
    pub sliding_window_pattern: Option<usize>,
}

impl Gemma3ConfigPartial {
    pub fn into_config(self) -> Result<candle_transformers::models::gemma3::Config> {
        Ok(candle_transformers::models::gemma3::Config {
            vocab_size: self.vocab_size,
            hidden_size: self.hidden_size,
            intermediate_size: self.intermediate_size,
            num_hidden_layers: self.num_hidden_layers,
            num_attention_heads: self.num_attention_heads,
            num_key_value_heads: self.num_key_value_heads,
            head_dim: self.head_dim,
            rms_norm_eps: self.rms_norm_eps,
            rope_theta: self.rope_theta,
            max_position_embeddings: self.max_position_embeddings,
            attention_bias: self.attention_bias,
            attn_logit_softcapping: self.attn_logit_softcapping,
            final_logit_softcapping: self.final_logit_softcapping,
            hidden_activation: match self.hidden_activation.as_deref().unwrap_or("silu") {
                "silu" => candle_nn::Activation::Silu,
                "gelu" => candle_nn::Activation::Gelu,
                _ => bail!("unsupported activation: {:?}", self.hidden_activation),
            },
            query_pre_attn_scalar: self.query_pre_attn_scalar.unwrap_or(0),
            rope_local_base_freq: self.rope_local_base_freq,
            sliding_window: self.sliding_window.unwrap_or(0),
            sliding_window_pattern: self.sliding_window_pattern.unwrap_or(0),
        })
    }
}

impl QwenConfigPartial {
    pub fn into_config(self) -> Result<QwenConfig> {
        Ok(QwenConfig {
            vocab_size: self.vocab_size,
            hidden_size: self.hidden_size,
            intermediate_size: self.intermediate_size,
            num_hidden_layers: self.num_hidden_layers,
            num_attention_heads: self.num_attention_heads,
            num_key_value_heads: self.num_key_value_heads,
            hidden_act: match self.hidden_act.as_str() {
                "silu" => candle_nn::Activation::Silu,
                _ => bail!("unsupported activation: {}", self.hidden_act),
            },
            max_position_embeddings: self.max_position_embeddings,
            rms_norm_eps: self.rms_norm_eps,
            rope_theta: self.rope_theta,
            use_sliding_window: false,
            sliding_window: self.sliding_window.unwrap_or(self.max_position_embeddings),
            max_window_layers: self.num_hidden_layers,
            tie_word_embeddings: true,
        })
    }
}

pub fn ensure_hf_asset(model_id: &str, revision: &str, path: &Path, filename: &str) -> Result<()> {
    if path.exists() {
        return Ok(());
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let url = format!(
        "https://huggingface.co/{}/resolve/{}/{}",
        model_id, revision, filename
    );

    tracing::info!("downloading {}...", url);
    let mut request = ureq::get(&url);
    if let Ok(token) = std::env::var("HF_TOKEN") {
        request = request.header("Authorization", &format!("Bearer {}", token));
    }

    let mut response = request.call()?;
    let mut file = fs::File::create(path)?;
    let mut reader = response.body_mut().as_reader();
    std::io::copy(&mut reader, &mut file)?;

    Ok(())
}

pub fn get_device() -> Result<Device> {
    #[cfg(feature = "cuda")]
    {
        match Device::new_cuda(0) {
            Ok(d) => {
                tracing::info!("Using CUDA device 0");
                Ok(d)
            }
            Err(e) => {
                tracing::warn!("Failed to initialize CUDA, falling back to CPU: {:?}", e);
                Ok(Device::Cpu)
            }
        }
    }
    #[cfg(not(feature = "cuda"))]
    {
        Ok(Device::Cpu)
    }
}

pub struct QwenModelSession {
    model: QwenModel,
    lm_head: candle_nn::Linear,
    tokens: Vec<u32>,
    device: Device,
}

impl QwenModelSession {
    pub fn new(config: &QwenConfig, vb: &VarBuilder<'static>, device: &Device) -> Result<Self> {
        let model = QwenModel::new(config, vb.clone())?;
        let lm_head = if config.tie_word_embeddings {
            candle_nn::Linear::new(
                vb.pp("model.embed_tokens")
                    .get((config.vocab_size, config.hidden_size), "weight")?,
                None,
            )
        } else {
            candle_nn::linear_no_bias(config.hidden_size, config.vocab_size, vb.pp("lm_head"))?
        };

        Ok(Self {
            model,
            lm_head,
            tokens: Vec::new(),
            device: device.clone(),
        })
    }

    pub fn generate(
        &mut self,
        prompt: &str,
        max_tokens: usize,
        tokenizer: &Tokenizer,
    ) -> Result<String> {
        let encoding = tokenizer
            .encode(prompt, true)
            .map_err(|m| anyhow!("encoding failed: {}", m))?;

        let new_tokens = encoding.get_ids();
        let mut generated_tokens = Vec::new();
        let eos_token_id = tokenizer.token_to_id("<|im_end|>").unwrap_or(151645);

        if !new_tokens.is_empty() {
            let tokens_tensor = Tensor::new(new_tokens, &self.device)?.unsqueeze(0)?;
            let pos = self.tokens.len();
            self.model.forward(&tokens_tensor, pos, None)?;
            self.tokens.extend_from_slice(new_tokens);
        }

        for _ in 0..max_tokens {
            let last_token = *self.tokens.last().unwrap();
            let tokens_tensor = Tensor::new(&[last_token], &self.device)?.unsqueeze(0)?;

            let hidden_states = self
                .model
                .forward(&tokens_tensor, self.tokens.len() - 1, None)?;
            let last_hidden = hidden_states.narrow(1, 0, 1)?;
            let logits = last_hidden.apply(&self.lm_head)?;
            let last_logit = logits.get(0)?.get(0)?;

            // Simple greedy decoding
            let next_token = last_logit
                .to_vec1::<f32>()?
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(i, _)| i as u32)
                .unwrap();

            if next_token == eos_token_id {
                break;
            }

            self.tokens.push(next_token);
            generated_tokens.push(next_token);

            // Check for stop sequences in the decoded text so far
            let current_text = tokenizer
                .decode(&generated_tokens, true)
                .map_err(|m| anyhow!("decoding failed: {}", m))?;

            if current_text.contains("<|im_end|>")
                || current_text.contains("Human:")
                || current_text.contains("User:")
                || current_text.contains("<|im_start|>")
            {
                break;
            }
        }

        let mut decoded = tokenizer
            .decode(&generated_tokens, true)
            .map_err(|m| anyhow!("decoding failed: {}", m))?;

        // Clean up any trailing stop sequences
        for stop_seq in ["<|im_end|>", "Human:", "User:", "<|im_start|>"] {
            if let Some(idx) = decoded.find(stop_seq) {
                decoded.truncate(idx);
            }
        }

        Ok(decoded.trim().to_string())
    }
}

pub fn qwen_generate(
    prompt: &str,
    max_tokens: usize,
    config: &QwenConfig,
    vb: &VarBuilder<'static>,
    tokenizer: &Tokenizer,
    device: &Device,
) -> Result<String> {
    let mut session = QwenModelSession::new(config, vb, device)?;
    session.generate(prompt, max_tokens, tokenizer)
}
