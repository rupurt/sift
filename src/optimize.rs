use std::path::PathBuf;

use anyhow::{Result, Context};
use std::fs;

use crate::config::{Config, Ignore};
use crate::eval::{run_quality_evaluation, QualityEvaluationRequest};
use crate::search::adapters::qwen::{QwenModelSpec, QwenReranker};
use crate::search::GenerativeModel;

pub struct OptimizeRequest {
    pub corpus_dir: PathBuf,
    pub queries_path: PathBuf,
    pub qrels_path: PathBuf,
    pub shortlist: usize,
    pub iterations: usize,
    pub command: String,
    pub verbose: u8,
    pub query_limit: Option<usize>,
}

pub fn run_optimization(request: &OptimizeRequest, ignore: Option<&Ignore>, config: &Config) -> Result<()> {
    tracing::info!("Starting Sift Prompt Optimizer...");

    let mut current_config = config.clone();

    // Setup the LLM for generating new prompts
    tracing::info!("Loading LLM for prompt mutation...");
    let qwen_spec = QwenModelSpec::default();
    let reranker = QwenReranker::load(qwen_spec)?;

    let strategies = vec![
        ("page-index-splade", "splade", "Provide 5 synonymous technical keywords, API names, or standard library functions."),
        ("page-index-classified", "classified", "Identify the core domain. Write a concise, 2-sentence summary describing the underlying subject matter, methodology, and expected document type."),
        ("page-index-llm", "hyde", "Generate a concise, hypothetical technical document snippet that would satisfy the user's intent. Focus on code structures, API names, and implementation logic."),
    ];

    for (strategy_name, config_key, intent_desc) in strategies {
        tracing::info!("Optimizing {} prompt...", strategy_name);

        // Baseline eval
        let mut eval_req = QualityEvaluationRequest {
            strategy: strategy_name.to_string(),
            baseline: Some("bm25".to_string()),
            command: request.command.clone(),
            corpus_dir: request.corpus_dir.clone(),
            queries_path: Some(request.queries_path.clone()),
            qrels_path: request.qrels_path.clone(),
            shortlist: request.shortlist,
            dense_model: crate::dense::DenseModelSpec::default(),
            verbose: request.verbose,
            query_limit: request.query_limit,
            prompts: Some(current_config.prompts.clone()),
        };

        let mut best_gain = run_quality_evaluation(&eval_req, ignore)?
            .reactor_metrics
            .map(|m| m.signal_gain)
            .unwrap_or(0.0);

        let mut best_prompt = match config_key {
            "splade" => current_config.prompts.splade.clone(),
            "classified" => current_config.prompts.classified.clone(),
            "hyde" => current_config.prompts.hyde.clone(),
            _ => None,
        };

        tracing::info!("Baseline Signal Gain for {}: {:.4}", strategy_name, best_gain);

        for i in 0..request.iterations {
            tracing::info!("Iteration {}/{}", i + 1, request.iterations);

            let current_prompt_text = best_prompt.clone().unwrap_or_else(|| "You are an expert.".to_string());
            let mutation_prompt = format!(
                "<|im_start|>system\nYou are an AI prompt engineer. Your goal is to improve a system prompt used for semantic search query expansion. The prompt should instruct an LLM to: {}\nOutput ONLY the raw new prompt text. Do not include any conversational filler, markdown formatting, or quotes around the prompt.<|im_end|>\n<|im_start|>user\nCurrent Prompt: {}\nGenerate a variation of this prompt that is more concise and focuses heavily on technical software engineering terms.<|im_end|>\n<|im_start|>assistant\n",
                intent_desc, current_prompt_text
            );

            let new_prompt = match reranker.generate(&mutation_prompt, 150) {
                Ok(p) => p.trim().to_string(),
                Err(e) => {
                    tracing::warn!("Failed to mutate prompt: {}. Skipping iteration.", e);
                    continue;
                }
            };

            tracing::info!("Testing new prompt: {:?}", new_prompt);

            match config_key {
                "splade" => eval_req.prompts.as_mut().unwrap().splade = Some(new_prompt.clone()),
                "classified" => eval_req.prompts.as_mut().unwrap().classified = Some(new_prompt.clone()),
                "hyde" => eval_req.prompts.as_mut().unwrap().hyde = Some(new_prompt.clone()),
                _ => {}
            }

            let new_gain = match run_quality_evaluation(&eval_req, ignore) {
                Ok(report) => report.reactor_metrics.map(|m| m.signal_gain).unwrap_or(0.0),
                Err(e) => {
                    tracing::warn!("Evaluation failed during optimization: {}. Skipping.", e);
                    continue;
                }
            };

            if new_gain > best_gain {
                tracing::info!("✅ Improvement found! {:.4} -> {:.4}", best_gain, new_gain);
                best_gain = new_gain;
                best_prompt = Some(new_prompt);
            } else {
                tracing::info!("❌ No improvement. {:.4} <= {:.4}", new_gain, best_gain);
            }
        }

        match config_key {
            "splade" => current_config.prompts.splade = best_prompt,
            "classified" => current_config.prompts.classified = best_prompt,
            "hyde" => current_config.prompts.hyde = best_prompt,
            _ => {}
        }
    }

    // Save to local sift.toml
    tracing::info!("Saving optimized prompts to ./sift.toml");
    
    // We only serialize the prompts section to avoid wiping out other config if it wasn't there
    let toml_string = toml::to_string(&current_config.prompts)?;
    let mut final_toml = String::new();
    final_toml.push_str("[prompts]\n");
    final_toml.push_str(&toml_string);

    fs::write("./sift.toml", final_toml).context("Failed to write ./sift.toml")?;
    tracing::info!("Optimization complete.");

    Ok(())
}