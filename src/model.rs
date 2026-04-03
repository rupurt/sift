use std::fmt;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::str::FromStr;

use anyhow::{Context, Result, anyhow, bail};
use candle_core::{DType, Device};
use candle_nn::VarBuilder;
use metamorph::{
    CompatibilityStatus, ConvertRequest, Format as MetamorphFormat, Source as MetamorphSource,
    Target as MetamorphTarget, compatibility as metamorph_compatibility,
    convert as metamorph_convert,
};
use serde::{Deserialize, Serialize};
use tokenizers::Tokenizer;

use crate::cache::cache_dir;

const DEFAULT_HF_REVISION: &str = "main";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ModelArtifactFormat {
    Gguf,
    HfSafetensors,
    Safetensors,
    Mlx,
}

impl fmt::Display for ModelArtifactFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::Gguf => "gguf",
            Self::HfSafetensors => "hf-safetensors",
            Self::Safetensors => "safetensors",
            Self::Mlx => "mlx",
        };
        f.write_str(label)
    }
}

impl From<MetamorphFormat> for ModelArtifactFormat {
    fn from(value: MetamorphFormat) -> Self {
        match value {
            MetamorphFormat::Gguf => Self::Gguf,
            MetamorphFormat::HfSafetensors => Self::HfSafetensors,
            MetamorphFormat::Safetensors => Self::Safetensors,
            MetamorphFormat::Mlx => Self::Mlx,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelSource {
    LocalPath(PathBuf),
    HuggingFace {
        repo: String,
        revision: Option<String>,
    },
}

impl ModelSource {
    pub fn hugging_face(repo: impl Into<String>) -> Self {
        Self::HuggingFace {
            repo: repo.into(),
            revision: None,
        }
    }

    pub fn hugging_face_revision(repo: impl Into<String>, revision: impl Into<String>) -> Self {
        Self::HuggingFace {
            repo: repo.into(),
            revision: Some(revision.into()),
        }
    }

    pub fn display_name(&self) -> String {
        match self {
            Self::LocalPath(path) => path.display().to_string(),
            Self::HuggingFace { repo, revision } => match revision {
                Some(revision) => format!("hf://{repo}@{revision}"),
                None => format!("hf://{repo}"),
            },
        }
    }
}

impl fmt::Display for ModelSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.display_name())
    }
}

impl FromStr for ModelSource {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self> {
        if let Some(rest) = value.strip_prefix("hf://") {
            if rest.is_empty() {
                bail!("invalid Hugging Face source: {value}");
            }

            let (repo, revision) = match rest.split_once('@') {
                Some((repo, revision)) if !repo.is_empty() && !revision.is_empty() => {
                    (repo.to_owned(), Some(revision.to_owned()))
                }
                Some(_) => bail!("invalid Hugging Face source: {value}"),
                None => (rest.to_owned(), None),
            };

            return Ok(Self::HuggingFace { repo, revision });
        }

        Ok(Self::LocalPath(PathBuf::from(value)))
    }
}

impl From<PathBuf> for ModelSource {
    fn from(value: PathBuf) -> Self {
        Self::LocalPath(value)
    }
}

impl From<&Path> for ModelSource {
    fn from(value: &Path) -> Self {
        Self::LocalPath(value.to_path_buf())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ModelRuntimeContract {
    CandleSafetensorsBundle,
}

impl fmt::Display for ModelRuntimeContract {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::CandleSafetensorsBundle => "candle-safetensors-bundle",
        };
        f.write_str(label)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ModelPreparationMode {
    ReusedCompatible,
    DownloadedCompatible,
    ReusedConverted,
    Converted,
}

impl fmt::Display for ModelPreparationMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::ReusedCompatible => "reused-compatible",
            Self::DownloadedCompatible => "downloaded-compatible",
            Self::ReusedConverted => "reused-converted",
            Self::Converted => "converted",
        };
        f.write_str(label)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreparedModel {
    pub source: ModelSource,
    pub runtime_contract: ModelRuntimeContract,
    pub root: PathBuf,
    pub config: PathBuf,
    pub tokenizer: PathBuf,
    pub weights: PathBuf,
    pub generation_config: Option<PathBuf>,
    pub source_format: Option<ModelArtifactFormat>,
    pub preparation_mode: ModelPreparationMode,
    pub lossy: bool,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone)]
struct PreparedBundlePaths {
    root: PathBuf,
    config: PathBuf,
    tokenizer: PathBuf,
    weights: PathBuf,
    generation_config: Option<PathBuf>,
}

pub fn prepare_model(
    source: impl Into<ModelSource>,
    runtime_contract: ModelRuntimeContract,
) -> Result<PreparedModel> {
    let source = source.into();

    match runtime_contract {
        ModelRuntimeContract::CandleSafetensorsBundle => prepare_candle_bundle(source),
    }
}

pub(crate) fn download_hf_asset(
    model_id: &str,
    revision: &str,
    path: &Path,
    relative_path: &str,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create Hugging Face asset dir {}", parent.display()))?;
    }

    let temp_path = temp_download_path(path)?;
    let url = format!(
        "https://huggingface.co/{}/resolve/{}/{}",
        model_id, revision, relative_path
    );

    let mut request = ureq::get(&url);
    if let Ok(token) = std::env::var("HF_TOKEN") {
        request = request.header("Authorization", &format!("Bearer {}", token));
    }

    let mut response = request
        .call()
        .with_context(|| format!("download Hugging Face asset {relative_path} from {url}"))?;
    let mut file = fs::File::create(&temp_path)
        .with_context(|| format!("create temp asset {}", temp_path.display()))?;
    let mut reader = response.body_mut().as_reader();
    std::io::copy(&mut reader, &mut file)
        .with_context(|| format!("stream response body for {url}"))?;
    file.sync_all()
        .with_context(|| format!("flush temp asset {}", temp_path.display()))?;
    fs::rename(&temp_path, path)
        .with_context(|| format!("install Hugging Face asset {}", path.display()))?;

    Ok(())
}

fn prepare_candle_bundle(source: ModelSource) -> Result<PreparedModel> {
    let inferred_format = infer_source_format(&source)?;

    match source.clone() {
        ModelSource::LocalPath(path) => prepare_local_candle_bundle(source, &path, inferred_format),
        ModelSource::HuggingFace { repo, revision } => {
            prepare_hf_candle_bundle(source, &repo, revision.as_deref(), inferred_format)
        }
    }
}

fn prepare_local_candle_bundle(
    source: ModelSource,
    path: &Path,
    inferred_format: Option<ModelArtifactFormat>,
) -> Result<PreparedModel> {
    if inferred_format == Some(ModelArtifactFormat::HfSafetensors) {
        let bundle = validate_candle_bundle(path)?;
        return Ok(build_prepared_model(
            source,
            bundle,
            inferred_format,
            ModelPreparationMode::ReusedCompatible,
            false,
            vec!["reused an existing compatible local bundle".to_owned()],
        ));
    }

    prepare_with_metamorph(source, inferred_format)
}

fn prepare_hf_candle_bundle(
    source: ModelSource,
    repo: &str,
    revision: Option<&str>,
    inferred_format: Option<ModelArtifactFormat>,
) -> Result<PreparedModel> {
    if inferred_format == Some(ModelArtifactFormat::Gguf) {
        return prepare_with_metamorph(source, inferred_format);
    }

    match prepare_compatible_hf_bundle(&source, repo, revision, inferred_format) {
        Ok(prepared) => Ok(prepared),
        Err(direct_error) => {
            if can_convert_with_metamorph(&source, inferred_format)? {
                return prepare_with_metamorph(source, inferred_format);
            }

            Err(direct_error)
        }
    }
}

fn prepare_compatible_hf_bundle(
    source: &ModelSource,
    repo: &str,
    revision: Option<&str>,
    inferred_format: Option<ModelArtifactFormat>,
) -> Result<PreparedModel> {
    let revision = revision.unwrap_or(DEFAULT_HF_REVISION);
    validate_repo_path("repo", repo)?;
    validate_repo_path("revision", revision)?;

    let root = cache_dir("models")?
        .join(Path::new(repo))
        .join(Path::new(revision));
    let bundle = match validate_candle_bundle(&root) {
        Ok(bundle) => {
            let mut notes = vec![format!(
                "reused cached compatible Hugging Face bundle at `{}`",
                bundle.root.display()
            )];
            if revision == DEFAULT_HF_REVISION {
                notes.push("using default Hugging Face revision `main`".to_owned());
            }
            return Ok(build_prepared_model(
                source.clone(),
                bundle,
                inferred_format.or(Some(ModelArtifactFormat::HfSafetensors)),
                ModelPreparationMode::ReusedCompatible,
                false,
                notes,
            ));
        }
        Err(_) => {
            remove_path_if_exists(&root).with_context(|| {
                format!("clear invalid compatible bundle cache {}", root.display())
            })?;
            fs::create_dir_all(&root)
                .with_context(|| format!("create compatible bundle cache {}", root.display()))?;

            download_hf_asset(repo, revision, &root.join("config.json"), "config.json")?;
            download_hf_asset(
                repo,
                revision,
                &root.join("tokenizer.json"),
                "tokenizer.json",
            )?;
            download_hf_asset(
                repo,
                revision,
                &root.join("model.safetensors"),
                "model.safetensors",
            )?;
            let _ = download_hf_asset(
                repo,
                revision,
                &root.join("generation_config.json"),
                "generation_config.json",
            );
            validate_candle_bundle(&root)?
        }
    };

    let mut notes = vec![format!(
        "downloaded a compatible Hugging Face bundle into `{}`",
        bundle.root.display()
    )];
    if revision == DEFAULT_HF_REVISION {
        notes.push("using default Hugging Face revision `main`".to_owned());
    }
    if bundle.generation_config.is_none() {
        notes.push(
            "compatible bundle does not include `generation_config.json`; current sift runtime contract does not require it".to_owned(),
        );
    }

    Ok(build_prepared_model(
        source.clone(),
        bundle,
        inferred_format.or(Some(ModelArtifactFormat::HfSafetensors)),
        ModelPreparationMode::DownloadedCompatible,
        false,
        notes,
    ))
}

fn prepare_with_metamorph(
    source: ModelSource,
    inferred_format: Option<ModelArtifactFormat>,
) -> Result<PreparedModel> {
    let output_root =
        converted_bundle_root(&source, ModelRuntimeContract::CandleSafetensorsBundle)?;
    if let Ok(bundle) = validate_candle_bundle(&output_root) {
        return Ok(build_prepared_model(
            source,
            bundle,
            inferred_format,
            ModelPreparationMode::ReusedConverted,
            true,
            vec![
                format!(
                    "reused a previously converted compatible bundle at `{}`",
                    output_root.display()
                ),
                compatibility_tradeoff_note(),
            ],
        ));
    }

    remove_path_if_exists(&output_root)
        .with_context(|| format!("clear invalid converted bundle {}", output_root.display()))?;

    let request = ConvertRequest {
        source: to_metamorph_source(&source),
        target: MetamorphTarget::LocalDir(output_root.clone()),
        from: inferred_format.and_then(to_metamorph_format),
        to: MetamorphFormat::HfSafetensors,
        allow_lossy: true,
        refresh_remote: false,
    };

    let compatibility = metamorph_compatibility(&request).with_context(|| {
        format!(
            "check metamorph compatibility for {}",
            source.display_name()
        )
    })?;
    if compatibility.status != CompatibilityStatus::Executable || !compatibility.blockers.is_empty()
    {
        let blockers = if compatibility.blockers.is_empty() {
            "none".to_owned()
        } else {
            compatibility.blockers.join("; ")
        };
        bail!(
            "metamorph cannot prepare {} for {}: status={}, blockers={}",
            source.display_name(),
            ModelRuntimeContract::CandleSafetensorsBundle,
            compatibility.status,
            blockers,
        );
    }

    let conversion = metamorph_convert(&request)
        .with_context(|| format!("convert {} through metamorph", source.display_name()))?;
    let bundle = validate_candle_bundle(&conversion.output)?;

    let mut notes = compatibility.notes;
    notes.extend(conversion.acquisition.notes);
    if let Some(backend) = compatibility.backend {
        notes.push(format!("converted via metamorph backend `{backend}`"));
    }
    notes.push(compatibility_tradeoff_note());

    Ok(build_prepared_model(
        source,
        bundle,
        compatibility
            .source_format
            .map(Into::into)
            .or(inferred_format),
        ModelPreparationMode::Converted,
        true,
        notes,
    ))
}

fn can_convert_with_metamorph(
    source: &ModelSource,
    inferred_format: Option<ModelArtifactFormat>,
) -> Result<bool> {
    let request = ConvertRequest {
        source: to_metamorph_source(source),
        target: MetamorphTarget::LocalDir(PathBuf::from(".")),
        from: inferred_format.and_then(to_metamorph_format),
        to: MetamorphFormat::HfSafetensors,
        allow_lossy: true,
        refresh_remote: false,
    };
    let compatibility = metamorph_compatibility(&request)?;
    Ok(
        compatibility.status == CompatibilityStatus::Executable
            && compatibility.blockers.is_empty(),
    )
}

fn infer_source_format(source: &ModelSource) -> Result<Option<ModelArtifactFormat>> {
    match source {
        ModelSource::LocalPath(path) => infer_local_source_format(path),
        ModelSource::HuggingFace { repo, .. } => Ok(infer_hf_source_format(repo)),
    }
}

fn infer_local_source_format(path: &Path) -> Result<Option<ModelArtifactFormat>> {
    if !path.exists() {
        bail!("missing model source path {}", path.display());
    }

    if path.is_file() {
        return Ok(
            match path.extension().and_then(|extension| extension.to_str()) {
                Some("gguf") => Some(ModelArtifactFormat::Gguf),
                Some("safetensors") => Some(ModelArtifactFormat::Safetensors),
                _ => None,
            },
        );
    }

    let config = path.join("config.json");
    let tokenizer = path.join("tokenizer.json");
    let weights = path.join("model.safetensors");
    if config.is_file() && tokenizer.is_file() && weights.is_file() {
        return Ok(Some(ModelArtifactFormat::HfSafetensors));
    }

    let mut has_gguf = false;
    let mut has_safetensors = false;
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        match entry
            .path()
            .extension()
            .and_then(|extension| extension.to_str())
        {
            Some("gguf") => has_gguf = true,
            Some("safetensors") => has_safetensors = true,
            _ => {}
        }
    }

    if has_gguf {
        return Ok(Some(ModelArtifactFormat::Gguf));
    }
    if has_safetensors {
        return Ok(Some(ModelArtifactFormat::Safetensors));
    }

    Ok(None)
}

fn infer_hf_source_format(repo: &str) -> Option<ModelArtifactFormat> {
    let normalized = repo.to_ascii_lowercase();

    if normalized.contains("gguf") {
        return Some(ModelArtifactFormat::Gguf);
    }
    if normalized.contains("mlx") {
        return Some(ModelArtifactFormat::Mlx);
    }
    if normalized.contains("safetensors") {
        return Some(ModelArtifactFormat::HfSafetensors);
    }

    None
}

fn validate_candle_bundle(root: &Path) -> Result<PreparedBundlePaths> {
    if !root.exists() {
        bail!("missing prepared model bundle {}", root.display());
    }
    if !root.is_dir() {
        bail!(
            "prepared model bundle must be a directory, got {}",
            root.display()
        );
    }

    let root = fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
    let config = root.join("config.json");
    let tokenizer = root.join("tokenizer.json");
    let weights = root.join("model.safetensors");
    let generation_config = root.join("generation_config.json");

    ensure_file(&config, "config.json")?;
    ensure_file(&tokenizer, "tokenizer.json")?;
    ensure_file(&weights, "model.safetensors")?;

    parse_json_file(&config, "config.json")?;
    if generation_config.is_file() {
        parse_json_file(&generation_config, "generation_config.json")?;
    }
    Tokenizer::from_file(&tokenizer)
        .map_err(|err| anyhow!("load tokenizer {}: {}", tokenizer.display(), err))?;
    let weights_files = [weights.clone()];
    let device = Device::Cpu;
    let _ = unsafe { VarBuilder::from_mmaped_safetensors(&weights_files, DType::F32, &device) }
        .with_context(|| format!("mmap Candle safetensors {}", weights.display()))?;

    Ok(PreparedBundlePaths {
        root,
        config,
        tokenizer,
        weights,
        generation_config: generation_config.is_file().then_some(generation_config),
    })
}

fn build_prepared_model(
    source: ModelSource,
    bundle: PreparedBundlePaths,
    source_format: Option<ModelArtifactFormat>,
    preparation_mode: ModelPreparationMode,
    lossy: bool,
    notes: Vec<String>,
) -> PreparedModel {
    PreparedModel {
        source,
        runtime_contract: ModelRuntimeContract::CandleSafetensorsBundle,
        root: bundle.root,
        config: bundle.config,
        tokenizer: bundle.tokenizer,
        weights: bundle.weights,
        generation_config: bundle.generation_config,
        source_format,
        preparation_mode,
        lossy,
        notes,
    }
}

fn converted_bundle_root(source: &ModelSource, contract: ModelRuntimeContract) -> Result<PathBuf> {
    let cache_root = cache_dir("prepared_models")?;
    let descriptor = format!("{contract}|{}", source.display_name());
    let digest = blake3::hash(descriptor.as_bytes()).to_hex().to_string();
    let slug = source_cache_slug(source);

    Ok(cache_root
        .join(contract.to_string())
        .join(format!("{}-{}", slug, &digest[..16])))
}

fn source_cache_slug(source: &ModelSource) -> String {
    let raw = match source {
        ModelSource::LocalPath(path) => path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("local-model")
            .to_owned(),
        ModelSource::HuggingFace { repo, .. } => {
            repo.rsplit('/').next().unwrap_or("hf-model").to_owned()
        }
    };

    let mut slug = String::with_capacity(raw.len());
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
        } else if slug.chars().last() != Some('-') {
            slug.push('-');
        }
    }
    slug.trim_matches('-').to_owned().chars().take(48).collect()
}

fn compatibility_tradeoff_note() -> String {
    "GGUF-to-safetensors preparation is a compatibility path and does not preserve native 1-bit runtime efficiency".to_owned()
}

fn to_metamorph_source(source: &ModelSource) -> MetamorphSource {
    match source {
        ModelSource::LocalPath(path) => MetamorphSource::LocalPath(path.clone()),
        ModelSource::HuggingFace { repo, revision } => MetamorphSource::HuggingFace {
            repo: repo.clone(),
            revision: revision.clone(),
        },
    }
}

fn to_metamorph_format(format: ModelArtifactFormat) -> Option<MetamorphFormat> {
    Some(match format {
        ModelArtifactFormat::Gguf => MetamorphFormat::Gguf,
        ModelArtifactFormat::HfSafetensors => MetamorphFormat::HfSafetensors,
        ModelArtifactFormat::Safetensors => MetamorphFormat::Safetensors,
        ModelArtifactFormat::Mlx => MetamorphFormat::Mlx,
    })
}

fn ensure_file(path: &Path, label: &str) -> Result<()> {
    if !path.is_file() {
        bail!(
            "missing required prepared-model file `{label}` at {}",
            path.display()
        );
    }
    Ok(())
}

fn parse_json_file(path: &Path, label: &str) -> Result<()> {
    let json = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let _: serde_json::Value = serde_json::from_str(&json)
        .with_context(|| format!("parse {label} at {}", path.display()))?;
    Ok(())
}

fn remove_path_if_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }

    if path.is_dir() {
        fs::remove_dir_all(path).with_context(|| format!("remove dir {}", path.display()))?;
    } else {
        fs::remove_file(path).with_context(|| format!("remove file {}", path.display()))?;
    }

    Ok(())
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

fn temp_download_path(path: &Path) -> Result<PathBuf> {
    let file_name = path
        .file_name()
        .ok_or_else(|| anyhow!("missing filename for download target {}", path.display()))?;

    Ok(path.with_file_name(format!(".{}.part", file_name.to_string_lossy())))
}
