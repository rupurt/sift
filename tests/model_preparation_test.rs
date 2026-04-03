use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;
use std::str::FromStr;
use std::sync::Mutex;

use candle_core::Tensor;
use candle_core::quantized::gguf_file;
use candle_core::quantized::{GgmlDType, QTensor};
use serde_json::Value as JsonValue;
use serde_json::json;
use sift::{
    ModelArtifactFormat, ModelPreparationMode, ModelRuntimeContract, ModelSource, prepare_model,
};
use tempfile::tempdir;

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn prepares_mocked_remote_gguf_source_through_metamorph() {
    let _env_guard = ENV_LOCK.lock().expect("env lock");
    let temp = tempdir().expect("tempdir");
    let sift_cache = temp.path().join("sift-cache");
    let metamorph_cache = temp.path().join("metamorph-cache");
    let mock_root = temp.path().join("mock");

    write_mock_remote_gguf_repo(
        &mock_root,
        "prism-ml/Bonsai-8B-gguf",
        "main",
        "Bonsai-8B-Q4.gguf",
        Some("sha-main-001"),
    );

    unsafe {
        std::env::set_var("SIFT_CACHE", &sift_cache);
        std::env::set_var("METAMORPH_CACHE_DIR", &metamorph_cache);
        std::env::set_var("METAMORPH_HF_MOCK_ROOT", &mock_root);
    }

    let prepared = prepare_model(
        ModelSource::from_str("hf://prism-ml/Bonsai-8B-gguf@main").expect("parse HF source"),
        ModelRuntimeContract::CandleSafetensorsBundle,
    )
    .expect("prepare mocked GGUF source");

    unsafe {
        std::env::remove_var("SIFT_CACHE");
        std::env::remove_var("METAMORPH_CACHE_DIR");
        std::env::remove_var("METAMORPH_HF_MOCK_ROOT");
    }

    assert_eq!(prepared.source_format, Some(ModelArtifactFormat::Gguf));
    assert_eq!(prepared.preparation_mode, ModelPreparationMode::Converted);
    assert!(prepared.lossy);
    assert!(prepared.root.is_dir());
    assert!(prepared.config.is_file());
    assert!(prepared.tokenizer.is_file());
    assert!(prepared.weights.is_file());
    assert!(
        prepared
            .notes
            .iter()
            .any(|note| note.contains("compatibility path"))
    );

    let config: JsonValue =
        serde_json::from_slice(&fs::read(&prepared.config).expect("read config")).expect("config");
    assert_eq!(config["model_type"], "llama");
}

#[test]
fn reuses_compatible_local_bundle_after_initial_conversion() {
    let _env_guard = ENV_LOCK.lock().expect("env lock");
    let temp = tempdir().expect("tempdir");
    let sift_cache = temp.path().join("sift-cache");
    let metamorph_cache = temp.path().join("metamorph-cache");
    let source_path = temp.path().join("fixture.gguf");
    write_fixture_gguf(&source_path);

    unsafe {
        std::env::set_var("SIFT_CACHE", &sift_cache);
        std::env::set_var("METAMORPH_CACHE_DIR", &metamorph_cache);
    }

    let converted = prepare_model(
        source_path.clone(),
        ModelRuntimeContract::CandleSafetensorsBundle,
    )
    .expect("convert local gguf source");

    let reused = prepare_model(
        converted.root.clone(),
        ModelRuntimeContract::CandleSafetensorsBundle,
    )
    .expect("reuse compatible local bundle");

    unsafe {
        std::env::remove_var("SIFT_CACHE");
        std::env::remove_var("METAMORPH_CACHE_DIR");
    }

    assert_eq!(reused.root, converted.root);
    assert_eq!(
        reused.source_format,
        Some(ModelArtifactFormat::HfSafetensors)
    );
    assert_eq!(
        reused.preparation_mode,
        ModelPreparationMode::ReusedCompatible
    );
    assert!(!reused.lossy);
    assert!(reused.generation_config.is_some());
}

fn write_fixture_gguf(path: &Path) {
    let device = candle_core::Device::Cpu;
    let tensor = Tensor::from_vec(vec![0f32, 1.0, 2.0, 3.0], (2, 2), &device).expect("tensor");
    let qtensor = QTensor::quantize(&tensor, GgmlDType::F32).expect("qtensor");

    let metadata = vec![
        (
            "general.architecture",
            gguf_file::Value::String("llama".to_owned()),
        ),
        ("llama.context_length", gguf_file::Value::U32(64)),
        ("llama.embedding_length", gguf_file::Value::U32(32)),
        ("llama.block_count", gguf_file::Value::U32(1)),
        ("llama.feed_forward_length", gguf_file::Value::U32(64)),
        ("llama.attention.head_count", gguf_file::Value::U32(2)),
        ("llama.attention.head_count_kv", gguf_file::Value::U32(2)),
        ("llama.rope.freq_base", gguf_file::Value::F32(10000.0)),
        (
            "llama.attention.layer_norm_rms_epsilon",
            gguf_file::Value::F32(0.00001),
        ),
        (
            "tokenizer.ggml.model",
            gguf_file::Value::String("gpt2".to_owned()),
        ),
        (
            "tokenizer.ggml.pre",
            gguf_file::Value::String("gpt2".to_owned()),
        ),
        (
            "tokenizer.ggml.tokens",
            gguf_file::Value::Array(vec![
                gguf_file::Value::String("<unk>".to_owned()),
                gguf_file::Value::String("a".to_owned()),
                gguf_file::Value::String("b".to_owned()),
                gguf_file::Value::String("ab".to_owned()),
            ]),
        ),
        (
            "tokenizer.ggml.merges",
            gguf_file::Value::Array(vec![gguf_file::Value::String("a b".to_owned())]),
        ),
        ("tokenizer.ggml.unk_token_id", gguf_file::Value::U32(0)),
        ("tokenizer.ggml.bos_token_id", gguf_file::Value::U32(1)),
        ("tokenizer.ggml.eos_token_id", gguf_file::Value::U32(2)),
        (
            "tokenizer.ggml.add_bos_token",
            gguf_file::Value::Bool(false),
        ),
        (
            "tokenizer.ggml.add_eos_token",
            gguf_file::Value::Bool(false),
        ),
    ];
    let metadata_refs = metadata
        .iter()
        .map(|(name, value)| (*name, value))
        .collect::<Vec<_>>();

    let tensors = [("tok_embeddings.weight", qtensor)];
    let tensor_refs = tensors
        .iter()
        .map(|(name, tensor)| (*name, tensor))
        .collect::<Vec<_>>();

    let mut writer = BufWriter::new(File::create(path).expect("create gguf"));
    gguf_file::write(&mut writer, &metadata_refs, &tensor_refs).expect("write gguf");
    writer.flush().expect("flush gguf");
}

fn write_mock_remote_gguf_repo(
    root: &Path,
    repo: &str,
    revision: &str,
    artifact_name: &str,
    resolved_revision: Option<&str>,
) {
    let repo_root = root.join(repo).join(revision);
    fs::create_dir_all(&repo_root).expect("create repo root");
    write_fixture_gguf(&repo_root.join(artifact_name));

    if let Some(resolved_revision) = resolved_revision {
        fs::write(
            repo_root.join(".metamorph-hf.json"),
            serde_json::to_vec_pretty(&json!({ "resolved_revision": resolved_revision }))
                .expect("serialize mock hf config"),
        )
        .expect("write mock hf config");
    }
}
