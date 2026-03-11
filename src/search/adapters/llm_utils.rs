use std::fs;
use std::path::Path;

use anyhow::{Result, anyhow, bail};
use candle_core::{Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::qwen3::{Config as QwenConfig, Model as QwenModel};
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
            use_sliding_window: self.sliding_window.is_some(),
            sliding_window: Some(self.sliding_window.unwrap_or(self.max_position_embeddings)),
            max_window_layers: self.num_hidden_layers,
            tie_word_embeddings: true,
            attention_bias: false,
            head_dim: self
                .head_dim
                .unwrap_or(self.hidden_size / self.num_attention_heads),
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

pub fn qwen_generate(
    prompt: &str,
    max_tokens: usize,
    config: &QwenConfig,
    vb: &VarBuilder<'static>,
    tokenizer: &Tokenizer,
    device: &Device,
) -> Result<String> {
    let encoding = tokenizer
        .encode(prompt, true)
        .map_err(|m| anyhow!("encoding failed: {}", m))?;

    let mut tokens = encoding.get_ids().to_vec();
    let mut generated_tokens = Vec::new();

    let mut model = QwenModel::new(config, vb.clone())?;

    let lm_head = if config.tie_word_embeddings {
        candle_nn::Linear::new(
            vb.pp("model.embed_tokens")
                .get((config.vocab_size, config.hidden_size), "weight")?,
            None,
        )
    } else {
        candle_nn::linear_no_bias(config.hidden_size, config.vocab_size, vb.pp("lm_head"))?
    };

    let eos_token_id = tokenizer.token_to_id("<|im_end|>").unwrap_or(151645);

    for i in 0..max_tokens {
        let context_size = if i == 0 { tokens.len() } else { 1 };
        let tokens_tensor = if i == 0 {
            Tensor::new(tokens.as_slice(), device)?.unsqueeze(0)?
        } else {
            let last_token = *tokens.last().unwrap();
            Tensor::new(&[last_token], device)?.unsqueeze(0)?
        };

        let hidden_states = model.forward(&tokens_tensor, tokens.len() - context_size)?;
        let last_hidden = hidden_states.narrow(1, tokens_tensor.dim(1)? - 1, 1)?;
        let logits = last_hidden.apply(&lm_head)?;
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

        tokens.push(next_token);
        generated_tokens.push(next_token);
    }

    let decoded = tokenizer
        .decode(&generated_tokens, true)
        .map_err(|m| anyhow!("decoding failed: {}", m))?;

    Ok(decoded)
}
