use std::env;
use std::fs;
use std::path::{Component, Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use candle_core::{Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config as BertConfig, DTYPE};
use rust_tokenizers::tokenizer::{BertTokenizer, Tokenizer, TruncationStrategy};
use serde::Deserialize;

pub const DEFAULT_MODEL_ID: &str = "sentence-transformers/all-MiniLM-L6-v2";
pub const DEFAULT_MODEL_REVISION: &str = "main";
pub const DEFAULT_MAX_LENGTH: usize = 40;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DenseModelSpec {
    pub model_id: String,
    pub revision: String,
    pub max_length: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelAssets {
    pub root: PathBuf,
    pub config: PathBuf,
    pub tokenizer_config: PathBuf,
    pub weights: PathBuf,
    pub vocab: PathBuf,
    pub pooling: PathBuf,
}

pub struct DenseReranker {
    spec: DenseModelSpec,
    pad_token_id: i64,
    tokenizer: BertTokenizer,
    model: BertModel,
    device: Device,
}

#[derive(Debug, Deserialize, Default)]
struct TokenizerConfig {
    #[serde(default)]
    do_lower_case: bool,
    strip_accents: Option<bool>,
    model_max_length: Option<usize>,
}

#[derive(Debug, Deserialize, Default)]
struct PoolingConfig {
    #[serde(default)]
    pooling_mode_mean_tokens: bool,
}

impl Default for DenseModelSpec {
    fn default() -> Self {
        Self {
            model_id: DEFAULT_MODEL_ID.to_string(),
            revision: DEFAULT_MODEL_REVISION.to_string(),
            max_length: DEFAULT_MAX_LENGTH,
        }
    }
}

impl DenseModelSpec {
    pub fn with_overrides(
        model_id: Option<String>,
        revision: Option<String>,
        max_length: Option<usize>,
    ) -> Self {
        Self {
            model_id: model_id.unwrap_or_else(|| DEFAULT_MODEL_ID.to_string()),
            revision: revision.unwrap_or_else(|| DEFAULT_MODEL_REVISION.to_string()),
            max_length: max_length.unwrap_or(DEFAULT_MAX_LENGTH).max(8),
        }
    }

    pub fn local_assets(&self, cache_root: &Path) -> Result<ModelAssets> {
        validate_repo_path("model_id", &self.model_id)?;
        validate_repo_path("revision", &self.revision)?;

        let root = cache_root
            .join(Path::new(&self.model_id))
            .join(Path::new(&self.revision));

        Ok(ModelAssets {
            root: root.clone(),
            config: root.join("config.json"),
            tokenizer_config: root.join("tokenizer_config.json"),
            weights: root.join("model.safetensors"),
            vocab: root.join("vocab.txt"),
            pooling: root.join("1_Pooling").join("config.json"),
        })
    }

    pub fn resolve_assets(&self) -> Result<ModelAssets> {
        let cache_root = model_cache_root()?;
        let assets = self.local_assets(&cache_root)?;
        ensure_asset(
            self,
            &assets.config,
            "config.json",
            "transformer model config",
        )?;
        ensure_asset(
            self,
            &assets.tokenizer_config,
            "tokenizer_config.json",
            "tokenizer config",
        )?;
        ensure_asset(self, &assets.weights, "model.safetensors", "model weights")?;
        ensure_asset(self, &assets.vocab, "vocab.txt", "tokenizer vocabulary")?;
        ensure_asset(
            self,
            &assets.pooling,
            "1_Pooling/config.json",
            "pooling config",
        )?;
        Ok(assets)
    }
}

impl DenseReranker {
    pub fn load(spec: DenseModelSpec) -> Result<Self> {
        let assets = spec.resolve_assets()?;
        let config: BertConfig = serde_json::from_str(
            &fs::read_to_string(&assets.config)
                .with_context(|| format!("read {}", assets.config.display()))?,
        )
        .with_context(|| format!("parse {}", assets.config.display()))?;
        let tokenizer_config: TokenizerConfig = serde_json::from_str(
            &fs::read_to_string(&assets.tokenizer_config)
                .with_context(|| format!("read {}", assets.tokenizer_config.display()))?,
        )
        .with_context(|| format!("parse {}", assets.tokenizer_config.display()))?;
        let pooling: PoolingConfig = serde_json::from_str(
            &fs::read_to_string(&assets.pooling)
                .with_context(|| format!("read {}", assets.pooling.display()))?,
        )
        .with_context(|| format!("parse {}", assets.pooling.display()))?;

        if !pooling.pooling_mode_mean_tokens {
            bail!(
                "model '{}' does not advertise mean pooling in {}",
                spec.model_id,
                assets.pooling.display()
            );
        }

        let strip_accents = tokenizer_config
            .strip_accents
            .unwrap_or(tokenizer_config.do_lower_case);
        let tokenizer =
            BertTokenizer::from_file(&assets.vocab, tokenizer_config.do_lower_case, strip_accents)
                .map_err(|err| anyhow!(err.to_string()))
                .with_context(|| format!("load tokenizer from {}", assets.vocab.display()))?;

        let device = Device::Cpu;
        let vb = unsafe { VarBuilder::from_mmaped_safetensors(&[assets.weights], DTYPE, &device)? };
        let max_length = spec
            .max_length
            .min(tokenizer_config.model_max_length.unwrap_or(spec.max_length))
            .max(8);
        let pad_token_id = config.pad_token_id as i64;
        let model = BertModel::load(vb, &config).context("load candle bert model")?;

        Ok(Self {
            spec: DenseModelSpec { max_length, ..spec },
            pad_token_id,
            tokenizer,
            model,
            device,
        })
    }

    pub fn model_id(&self) -> &str {
        &self.spec.model_id
    }

    pub fn revision(&self) -> &str {
        &self.spec.revision
    }

    pub fn max_length(&self) -> usize {
        self.spec.max_length
    }

    pub fn score(&self, query: &str, documents: &[&str]) -> Result<Vec<f64>> {
        if documents.is_empty() {
            return Ok(Vec::new());
        }

        let mut texts = Vec::with_capacity(documents.len() + 1);
        texts.push(query.to_string());
        texts.extend(documents.iter().map(|text| (*text).to_string()));

        let embeddings = self.embed_texts(&texts)?;
        let query_embedding = embeddings.get(0)?;
        let mut scores = Vec::with_capacity(documents.len());

        for index in 1..texts.len() {
            let doc_embedding = embeddings.get(index)?;
            let similarity = (&query_embedding * &doc_embedding)?
                .sum_all()?
                .to_scalar::<f32>()? as f64;
            scores.push(similarity);
        }

        Ok(scores)
    }

    fn embed_texts(&self, texts: &[String]) -> Result<Tensor> {
        let encoded = self.tokenizer.encode_list(
            texts,
            self.spec.max_length,
            &TruncationStrategy::LongestFirst,
            0,
        );
        let max_len = encoded
            .iter()
            .map(|input| input.token_ids.len())
            .max()
            .unwrap_or(0);

        let mut token_ids = Vec::with_capacity(encoded.len());
        let mut token_type_ids = Vec::with_capacity(encoded.len());
        let mut attention_masks = Vec::with_capacity(encoded.len());

        for input in encoded {
            let mut ids = input.token_ids;
            let mut types = input
                .segment_ids
                .into_iter()
                .map(i64::from)
                .collect::<Vec<_>>();
            let mut mask = vec![1_i64; ids.len()];

            ids.resize(max_len, self.pad_token_id);
            types.resize(max_len, 0_i64);
            mask.resize(max_len, 0_i64);

            token_ids.push(Tensor::new(ids.as_slice(), &self.device)?);
            token_type_ids.push(Tensor::new(types.as_slice(), &self.device)?);
            attention_masks.push(Tensor::new(mask.as_slice(), &self.device)?);
        }

        let token_ids = Tensor::stack(&token_ids, 0)?;
        let token_type_ids = Tensor::stack(&token_type_ids, 0)?;
        let attention_mask = Tensor::stack(&attention_masks, 0)?;
        let hidden_states =
            self.model
                .forward(&token_ids, &token_type_ids, Some(&attention_mask))?;

        let pooled = mean_pool(&hidden_states, &attention_mask)?;
        normalize_l2(&pooled)
    }
}

fn validate_repo_path(label: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        bail!("{label} must not be empty");
    }

    for component in Path::new(value).components() {
        match component {
            Component::Normal(_) => {}
            Component::CurDir => continue,
            Component::ParentDir | Component::Prefix(_) | Component::RootDir => {
                bail!("{label} contains unsupported path component: {value}");
            }
        }
    }

    Ok(())
}

fn model_cache_root() -> Result<PathBuf> {
    if let Some(path) = env::var_os("SIFT_MODEL_CACHE") {
        return Ok(PathBuf::from(path));
    }
    if let Some(path) = env::var_os("XDG_CACHE_HOME") {
        return Ok(PathBuf::from(path).join("sift").join("models"));
    }
    if let Some(path) = env::var_os("HOME") {
        return Ok(PathBuf::from(path)
            .join(".cache")
            .join("sift")
            .join("models"));
    }

    bail!(
        "unable to determine a model cache directory; set SIFT_MODEL_CACHE, XDG_CACHE_HOME, or HOME"
    )
}

fn ensure_asset(
    spec: &DenseModelSpec,
    path: &Path,
    relative_path: &str,
    label: &str,
) -> Result<()> {
    if path.exists() {
        return Ok(());
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create model asset dir {}", parent.display()))?;
    }

    let url = format!(
        "https://huggingface.co/{}/resolve/{}/{}",
        spec.model_id, spec.revision, relative_path
    );
    let mut response = ureq::get(&url)
        .call()
        .with_context(|| format!("download {label} from {url}"))?;
    let temp_path = path.with_extension("tmp");
    let mut temp_file = fs::File::create(&temp_path)
        .with_context(|| format!("create temp model asset {}", temp_path.display()))?;
    let mut reader = response.body_mut().as_reader();
    std::io::copy(&mut reader, &mut temp_file)
        .with_context(|| format!("stream response body for {url}"))?;
    fs::rename(&temp_path, path)
        .with_context(|| format!("install model asset {}", path.display()))?;

    Ok(())
}

fn mean_pool(hidden_states: &Tensor, attention_mask: &Tensor) -> Result<Tensor> {
    let attention_mask = attention_mask.to_dtype(DTYPE)?.unsqueeze(2)?;
    let sum_mask = attention_mask.sum(1)?;
    let pooled = (hidden_states.broadcast_mul(&attention_mask)?).sum(1)?;
    Ok(pooled.broadcast_div(&sum_mask)?)
}

fn normalize_l2(embeddings: &Tensor) -> Result<Tensor> {
    Ok(embeddings.broadcast_div(&embeddings.sqr()?.sum_keepdim(1)?.sqrt()?)?)
}

#[cfg(test)]
mod model {
    use std::path::PathBuf;

    use tempfile::tempdir;

    use super::{DEFAULT_MAX_LENGTH, DEFAULT_MODEL_ID, DEFAULT_MODEL_REVISION, DenseModelSpec};

    #[test]
    fn local_asset_plan_is_file_based_and_complete() {
        let cache_dir = tempdir().expect("cache dir");
        let spec = DenseModelSpec::default();
        let assets = spec.local_assets(cache_dir.path()).expect("asset plan");

        assert!(assets.root.starts_with(cache_dir.path()));
        assert!(
            assets
                .root
                .ends_with(PathBuf::from(DEFAULT_MODEL_ID).join(DEFAULT_MODEL_REVISION))
        );
        assert!(assets.config.ends_with("config.json"));
        assert!(assets.tokenizer_config.ends_with("tokenizer_config.json"));
        assert!(assets.weights.ends_with("model.safetensors"));
        assert!(assets.vocab.ends_with("vocab.txt"));
        assert!(
            assets
                .pooling
                .ends_with(PathBuf::from("1_Pooling").join("config.json"))
        );
    }

    #[test]
    fn overrides_allow_explicit_max_length() {
        let spec = DenseModelSpec::with_overrides(
            Some("sentence-transformers/all-MiniLM-L6-v2".to_string()),
            Some("refs/pr/1".to_string()),
            Some(32),
        );

        assert_eq!(spec.model_id, "sentence-transformers/all-MiniLM-L6-v2");
        assert_eq!(spec.revision, "refs/pr/1");
        assert_eq!(spec.max_length, 32);
    }

    #[test]
    fn overrides_preserve_default_max_length_when_unspecified() {
        let spec = DenseModelSpec::with_overrides(None, None, None);

        assert_eq!(spec.max_length, DEFAULT_MAX_LENGTH);
    }
}
