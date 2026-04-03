# Library Guide

`sift` exposes its supported embedding surface at the crate root. This guide
covers the public modes that are intended for embedders today.

## Supported Public Surface

The stable crate-root contract includes:

- `Sift`, `SiftBuilder`
- `SearchInput`, `SearchOptions`
- `SearchProgress`, `SearchTelemetry`
- `ContextAssemblyRequest`, `ContextAssemblyResponse`
- `SearchTurnRequest`, `SearchTurnResponse`
- `SearchControllerRequest`, `SearchControllerResponse`
- `AutonomousSearchRequest`, `AutonomousSearchResponse`
- `AutonomousSearchMode`, `AutonomousPlannerState`, `AutonomousPlannerStepCursor`, `AutonomousPlannerStrategy`, `AutonomousPlannerStrategyKind`
- `AutonomousPlannerTrace`, `AutonomousPlannerTraceStep`, `AutonomousPlannerDecision`, `AutonomousPlannerAction`, `AutonomousPlannerStopReason`
- `AutonomousPlanner`, `HeuristicAutonomousPlanner`, `ModelDrivenAutonomousPlanner`
- graph episode DTOs and replay helpers (`AutonomousGraphEpisodeState`, `AutonomousGraphNode`, `AutonomousGraphEdge`, `AutonomousGraphFrontierEntry`, `replay_graph_trace`)
- `SearchEmission`, `SearchEmissionMode`
- `SearchPlan`, `QueryExpansionPolicy`, `RetrieverPolicy`, `FusionPolicy`, `RerankingPolicy`
- `Retriever`, `Fusion`, `Reranking`
- `SearchResponse`, `SearchHit`, `ContextArtifact`, `ContextArtifactKind`, `ScoreConfidence`
- `LocalContextSource`, `EnvironmentFactInput`, `ToolOutputInput`, `AgentTurnInput`
- `GenerativeModel`, `Conversation`
- `ModelSource`, `ModelRuntimeContract`, `PreparedModel`, `ModelArtifactFormat`, `ModelPreparationMode`, `prepare_model`

Everything under `sift::internal` is executable support code or repository
plumbing and should be treated as unstable.

## Model Preparation

Use `prepare_model` when your application needs a stable sift-owned boundary for
local model preparation before runtime loading:

```rust
use sift::{ModelRuntimeContract, ModelSource, prepare_model};

let prepared = prepare_model(
    ModelSource::hugging_face_revision("prism-ml/Bonsai-8B-gguf", "main"),
    ModelRuntimeContract::CandleSafetensorsBundle,
)?;

assert!(prepared.weights.is_file());
```

This seam is for compatibility preparation, not native 1-bit execution. Sift
reuses already-compatible bundles when it can and may invoke metamorph to
translate GGUF sources into the current Candle-loadable safetensors contract.
That conversion is lossy and should be treated as a runtime-compatibility
fallback rather than proof of native GGUF or 1-bit runtime support.

## Mode 1: Direct Search

Use `Sift::search` for standard one-shot retrieval, or
`Sift::search_with_progress` when you want synchronous progress callbacks during
corpus preparation and ranking.

```rust
use sift::{Fusion, Retriever, Reranking, SearchInput, SearchOptions, Sift};

fn main() -> anyhow::Result<()> {
    let engine = Sift::builder().build();

    let response = engine.search(
        SearchInput::new("./docs", "architecture decision").with_options(
            SearchOptions::default()
                .with_strategy("hybrid")
                .with_retrievers(vec![Retriever::Bm25, Retriever::Vector])
                .with_fusion(Fusion::Rrf)
                .with_reranking(Reranking::None)
                .with_limit(5)
                .with_shortlist(8),
        ),
    )?;

    for hit in response.hits {
        println!("{} {}", hit.rank, hit.path);
    }

    Ok(())
}
```

For visible progress:

```rust
use std::sync::{Arc, Mutex};

use sift::{SearchInput, SearchOptions, SearchProgress, Sift};

fn main() -> anyhow::Result<()> {
    let engine = Sift::builder().build();
    let progress = Arc::new(Mutex::new(Vec::new()));
    let captured = progress.clone();

    let response = engine.search_with_progress(
        SearchInput::new("./docs", "cache invalidation").with_options(
            SearchOptions::default().with_strategy("bm25"),
        ),
        Some(move |event: &SearchProgress| {
            captured.lock().expect("progress lock").push(event.clone());
        }),
    )?;

    let telemetry = engine.telemetry_snapshot();
    println!("hits {}", response.hits.len());
    println!("blob hits {}", telemetry.blob_hits);
    Ok(())
}
```

`telemetry_snapshot()` returns cumulative metrics for the current search run,
including blob reuse, fresh artifact builds, skipped artifacts, and BM25
cache/build counts.

### Direct Search Knobs

`SearchOptions` supports:

- `with_strategy`
- `with_intent`
- `with_limit`
- `with_shortlist`
- `with_retrievers`
- `with_fusion`
- `with_reranking`
- `with_verbose`
- `with_cache_dir`
- `with_local_context`

There are also advanced model override setters such as `with_dense_model`,
`with_rerank_model`, and `with_gemma_model`, but those currently depend on
model spec types under `sift::internal`. Use them only if you accept tighter
coupling to internal APIs.

## Mode 2: Context Assembly

Use `Sift::assemble_context` when you want retrieval plus a bounded retained
evidence set for downstream tooling or controller logic.

```rust
use sift::{
    ContextAssemblyBudget, ContextAssemblyRequest, SearchEmissionMode, Sift, ToolOutputInput,
};

fn main() -> anyhow::Result<()> {
    let engine = Sift::builder().build();

    let assembled = engine.assemble_context(
        ContextAssemblyRequest::new("./docs", "telemetry")
            .with_strategy("bm25")
            .with_budget(ContextAssemblyBudget::new(2))
            .with_emission_mode(SearchEmissionMode::Protocol)
            .with_local_context(vec![sift::LocalContextSource::ToolOutput(
                ToolOutputInput::new("rg", "call-1", "telemetry span waterfall"),
            )]),
    )?;

    println!("hits: {}", assembled.response.hits.len());
    println!("retained: {}", assembled.retained_artifacts.len());
    Ok(())
}
```

Use this mode when you need:

- retrieved hits
- a bounded retained-artifact list
- explicit pruning counts
- a protocol or latent emission derived from the same search

## Mode 3: Single Turn with Explicit Emission

Use `Sift::search_turn` when you need turn IDs, session IDs, traces, and an
explicit emission mode.

```rust
use sift::{SearchEmissionMode, SearchTurnRequest, Sift};

fn main() -> anyhow::Result<()> {
    let engine = Sift::builder().build();

    let turn = engine.search_turn(
        SearchTurnRequest::new("./docs", "controller state")
            .with_session_id("session-1")
            .with_turn_id("turn-1")
            .with_strategy("bm25")
            .with_emission_mode(SearchEmissionMode::Protocol),
    )?;

    println!("turn {}", turn.turn.turn_id);
    println!("results {}", turn.turn.result_count);
    Ok(())
}
```

### Emission Modes

- `SearchEmissionMode::View`: Emits a standard `SearchResponse`.
- `SearchEmissionMode::Protocol`: Emits `ProtocolSearchEmission` with turn and session metadata.
- `SearchEmissionMode::Latent`: Emits `LatentSearchEmission` with ranking-oriented hit features.

## Mode 4: Deterministic Multi-turn Controller

Use `Sift::search_controller` when your application already knows the planned
turns and wants Sift to execute them while managing retained evidence and
producing inspectable traces.

```rust
use sift::{
    FusionPolicy, QueryExpansionPolicy, RerankingPolicy, RetrieverPolicy, SearchControllerRequest,
    SearchPlan, SearchTurnRequest, Sift,
};

fn main() -> anyhow::Result<()> {
    let engine = Sift::builder().build();

    let plan = SearchPlan {
        name: "controller-lexical".to_string(),
        query_expansion: QueryExpansionPolicy::None,
        retrievers: vec![RetrieverPolicy::Bm25],
        fusion: FusionPolicy::Rrf,
        reranking: RerankingPolicy::None,
    };

    let response = engine.search_controller(
        SearchControllerRequest::new(
            plan,
            vec![
                SearchTurnRequest::new("./docs", "alpha").with_turn_id("turn-a"),
                SearchTurnRequest::new("./docs", "beta").with_turn_id("turn-b"),
            ],
        )
        .with_session_id("session-1")
        .with_retained_artifact_limit(2),
    )?;

    println!("completed: {}", response.state.completed);
    println!("turns: {}", response.turns.len());
    Ok(())
}
```

This mode gives you:

- explicit turn sequencing
- retained-artifact carryover between turns
- pruning when the retained-artifact budget is exceeded
- per-turn controller decisions in `SearchTrace`

It is deterministic and plan-driven. It does not currently invent turns or do
autonomous decomposition by itself.

## Mode 5: Supported Autonomous Search

Use `Sift::search_autonomous` when you want the supported built-in autonomous
runtime. It selects the shipped planner from
`AutonomousSearchRequest::planner_strategy`, chooses linear or graph mode from
`AutonomousSearchRequest::mode`, and does not require custom planner
injection.

```rust
use anyhow::Result;
use sift::{
    AutonomousPlannerStrategy, AutonomousSearchMode, AutonomousSearchRequest, Sift,
};

fn main() -> Result<()> {
    let engine = Sift::builder().build();
    let response = engine.search_autonomous(
        AutonomousSearchRequest::new("./docs", "find the cache invalidation path")
            .with_strategy("hybrid")
            .with_mode(AutonomousSearchMode::Graph)
            .with_planner_strategy(AutonomousPlannerStrategy::heuristic())
            .with_step_limit(3),
    )?;

    assert_eq!(response.planner_strategy, AutonomousPlannerStrategy::heuristic());
    assert_eq!(response.mode, AutonomousSearchMode::Graph);
    assert_eq!(
        response.planner_trace.planner_strategy,
        AutonomousPlannerStrategy::heuristic()
    );
    Ok(())
}
```

Use this mode when you need:

- built-in planner selection through `AutonomousPlannerStrategy`
- additive linear or graph runtime selection through `AutonomousSearchMode`
- the default heuristic planner when `planner_strategy` is omitted, or explicit
  model-driven selection through `AutonomousPlannerStrategy::model_driven()`
- a stable `AutonomousSearchResponse` carrying both `planner_trace` and the
  lowered `SearchTrace`
- the same autonomous runtime contract that the shipped CLI agent flow reuses
- a public crate-root surface that still coexists with direct search, context
  assembly, and explicit controller modes

For model-driven planning, configure the engine with
`SiftBuilder::with_generative_model` and request a model-driven strategy:

```rust
use std::sync::Arc;

use sift::{AutonomousPlannerStrategy, Conversation, GenerativeModel, Sift};

struct MyPlannerModel;

impl GenerativeModel for MyPlannerModel {
    fn generate(&self, _prompt: &str, _max_tokens: usize) -> anyhow::Result<String> {
        unimplemented!()
    }

    fn start_conversation(&self) -> anyhow::Result<Box<dyn Conversation>> {
        unimplemented!()
    }
}

fn main() -> anyhow::Result<()> {
    let engine = Sift::builder()
        .with_generative_model(Arc::new(MyPlannerModel))
        .build();

    let _response = engine.search_autonomous(
        sift::AutonomousSearchRequest::new("./docs", "plan a bounded search")
            .with_strategy("page-index-llm")
            .with_planner_strategy(
                AutonomousPlannerStrategy::model_driven().with_profile("local-planner-v1"),
            ),
    )?;

    Ok(())
}
```

`AutonomousSearchRequest`, `AutonomousSearchResponse`, and
`AutonomousPlannerState` remain the stable records embedders can persist or
exchange when they need to save root-task planning state, retained evidence,
graph episode state, and planner selection.

## Mode 6: Advanced Autonomous Execution Seam

Use `Sift::search_autonomous_with` only when your application already owns the
planning policy and wants Sift to lower planner-issued search decisions into
the existing controller runtime. Most embedders should prefer
`Sift::search_autonomous`.

```rust
use anyhow::Result;
use sift::{
    AutonomousPlanner, AutonomousPlannerAction, AutonomousPlannerDecision,
    AutonomousPlannerStopReason, AutonomousPlannerTrace, AutonomousPlannerTraceStep,
    AutonomousSearchRequest, Sift,
};

struct SingleStepPlanner;

impl AutonomousPlanner for SingleStepPlanner {
    fn plan(&self, request: &AutonomousSearchRequest) -> Result<AutonomousPlannerTrace> {
        Ok(AutonomousPlannerTrace::new(request.planner_strategy.clone())
            .with_steps(vec![
                AutonomousPlannerTraceStep::new(sift::AutonomousPlannerStepCursor::new(
                    "step-1", 1,
                ))
                .with_decisions(vec![
                    AutonomousPlannerDecision::new(AutonomousPlannerAction::Search)
                        .with_query("alpha")
                        .with_turn_id("turn-a"),
                    AutonomousPlannerDecision::new(AutonomousPlannerAction::Terminate)
                        .with_stop_reason(AutonomousPlannerStopReason::GoalSatisfied),
                ]),
            ])
            .with_completed(true)
            .with_stop_reason(AutonomousPlannerStopReason::GoalSatisfied))
    }
}

fn main() -> Result<()> {
    let engine = Sift::builder().build();
    let response = engine.search_autonomous_with(
        AutonomousSearchRequest::new("./docs", "find the alpha details")
            .with_mode(sift::AutonomousSearchMode::Graph)
            .with_strategy("bm25"),
        &SingleStepPlanner,
    )?;

    assert_eq!(response.turns.len(), 1);
    assert!(response.state.completed);
    Ok(())
}
```

## Local Context Injection

All request modes can inject synthetic context artifacts alongside filesystem
documents:

- `LocalContextSource::EnvironmentFact`
- `LocalContextSource::ToolOutput`
- `LocalContextSource::AgentTurn`

That context is searchable and shows up with synthetic provenance in returned
hits and traces.

## Generative Model Access

`Sift::generative` resolves the current strategy's generative model and returns
the crate-root `GenerativeModel` trait:

```rust
use sift::{SearchOptions, Sift};

fn main() -> anyhow::Result<()> {
    let engine = Sift::builder().build();
    let model = engine.generative(SearchOptions::default().with_strategy("page-index-llm"))?;
    let reply = model.generate("Summarize the query intent: telemetry cache", 64)?;
    println!("{reply}");
    Ok(())
}
```

The returned model also supports `start_conversation()` through the
`Conversation` trait.

## Builder Notes

`Sift::builder()` gives you a reusable engine instance with shared telemetry and
query-cache state.

Useful stable builder usage:

- `Sift::builder().build()`
- `Sift::builder().with_generative_model(...)` for built-in model-driven
  autonomous planning

Advanced builder methods such as `with_config`, `with_ignore`, and
`with_embedder` exist, but some of them depend on types outside the stable
crate-root surface. Prefer request-local configuration unless you need that
tighter integration.

`with_generative_model` is the stable builder hook for supplying the local
planner model used by built-in model-driven autonomous search.

## Status Boundary

Supported now:

- direct search
- context assembly
- single-turn traced search
- deterministic multi-turn controller execution
- built-in heuristic and model-driven autonomous runtime selection through
  `search_autonomous`
- additive linear/graph mode selection through `AutonomousSearchMode`
- custom planner injection through `search_autonomous_with`
- protocol and latent emissions
- synthetic local context artifacts

Not shipped yet:

- a stable crate-root config type
- a general-purpose interactive agentic CLI command
