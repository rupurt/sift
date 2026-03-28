use std::collections::HashSet;

use anyhow::{Result, bail};

use super::{
    AutonomousPlanner, AutonomousPlannerAction, AutonomousPlannerDecision,
    AutonomousPlannerStepCursor, AutonomousPlannerStopReason, AutonomousPlannerStrategyKind,
    AutonomousPlannerTrace, AutonomousPlannerTraceStep, AutonomousSearchRequest,
    LocalContextSource, RetainedArtifact, tokenize,
};

const DEFAULT_MAX_QUERY_TERMS: usize = 6;
const STOPWORDS: &[&str] = &[
    "a", "an", "and", "are", "as", "at", "by", "for", "from", "how", "in", "into", "is", "json",
    "me", "md", "of", "on", "or", "rs", "src", "the", "toml", "to", "txt", "with", "yaml", "yml",
    "cwd",
];

#[derive(Debug, Clone)]
pub struct HeuristicAutonomousPlanner {
    max_query_terms: usize,
}

#[derive(Debug, Clone)]
struct HeuristicPlanOutcome {
    queries: Vec<String>,
    stop_reason: AutonomousPlannerStopReason,
}

impl HeuristicAutonomousPlanner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_max_query_terms(mut self, max_query_terms: usize) -> Self {
        self.max_query_terms = max_query_terms.max(1);
        self
    }

    fn build_plan_outcome(&self, request: &AutonomousSearchRequest) -> HeuristicPlanOutcome {
        let mut all_queries = Vec::new();
        let mut used_queries = HashSet::new();
        let mut used_terms = HashSet::new();

        if self.should_seed_from_root_task(request) {
            let initial_query = self.build_initial_query(request);
            if !initial_query.is_empty() && used_queries.insert(initial_query.clone()) {
                used_terms.extend(initial_query.split_whitespace().map(str::to_string));
                all_queries.push(initial_query);
            }
        }

        for artifact in &request.state.retained_artifacts {
            let follow_up = self.build_follow_up_query(artifact, &used_terms);
            if follow_up.is_empty() || !used_queries.insert(follow_up.clone()) {
                continue;
            }

            used_terms.extend(follow_up.split_whitespace().map(str::to_string));
            all_queries.push(follow_up);
        }

        let stop_reason = self.stop_reason(request, all_queries.len());
        let queries = all_queries
            .into_iter()
            .take(request.state.step_limit)
            .collect::<Vec<_>>();

        HeuristicPlanOutcome {
            queries,
            stop_reason,
        }
    }

    fn should_seed_from_root_task(&self, request: &AutonomousSearchRequest) -> bool {
        request.state.current_step.sequence <= 1
    }

    fn build_initial_query(&self, request: &AutonomousSearchRequest) -> String {
        let root_terms = significant_terms(&request.root_task);
        let local_context_terms = request
            .local_context
            .iter()
            .flat_map(local_context_terms)
            .collect::<Vec<_>>();

        let mut query_terms = Vec::new();
        let mut seen = HashSet::new();

        append_terms(
            &mut query_terms,
            &mut seen,
            &root_terms,
            self.max_query_terms,
        );
        append_terms(
            &mut query_terms,
            &mut seen,
            &local_context_terms,
            self.max_query_terms,
        );

        if query_terms.is_empty() {
            append_terms(
                &mut query_terms,
                &mut seen,
                &fallback_terms(&request.root_task),
                self.max_query_terms,
            );
            append_terms(
                &mut query_terms,
                &mut seen,
                &request
                    .local_context
                    .iter()
                    .flat_map(fallback_local_context_terms)
                    .collect::<Vec<_>>(),
                self.max_query_terms,
            );
        }

        query_terms.join(" ")
    }

    fn build_follow_up_query(
        &self,
        artifact: &RetainedArtifact,
        used_terms: &HashSet<String>,
    ) -> String {
        let mut terms = Vec::new();
        let mut seen = HashSet::new();
        let candidate_terms = retained_artifact_terms(artifact);
        for token in &candidate_terms {
            if used_terms.contains(token.as_str()) || !seen.insert(token.clone()) {
                continue;
            }
            terms.push(token.clone());
            if terms.len() >= self.max_query_terms {
                break;
            }
        }

        if terms.is_empty() && candidate_terms.is_empty() {
            for token in fallback_retained_artifact_terms(artifact) {
                if used_terms.contains(token.as_str()) || !seen.insert(token.clone()) {
                    continue;
                }
                terms.push(token);
                if terms.len() >= self.max_query_terms {
                    break;
                }
            }
        }

        terms.join(" ")
    }

    fn stop_reason(
        &self,
        request: &AutonomousSearchRequest,
        productive_query_count: usize,
    ) -> AutonomousPlannerStopReason {
        if request.state.step_limit == 0 || productive_query_count > request.state.step_limit {
            AutonomousPlannerStopReason::StepLimitReached
        } else if request.state.retained_artifacts.is_empty() {
            AutonomousPlannerStopReason::NoFurtherQueries
        } else {
            AutonomousPlannerStopReason::NoAdditionalEvidence
        }
    }

    fn stop_rationale(reason: AutonomousPlannerStopReason) -> &'static str {
        match reason {
            AutonomousPlannerStopReason::GoalSatisfied => {
                "terminate because the heuristic planner judges the goal satisfied"
            }
            AutonomousPlannerStopReason::StepLimitReached => {
                "terminate because the bounded autonomous step budget was exhausted"
            }
            AutonomousPlannerStopReason::NoFurtherQueries => {
                "terminate because the planner could not derive another productive query"
            }
            AutonomousPlannerStopReason::NoAdditionalEvidence => {
                "terminate because retained evidence no longer yields a novel follow-up query"
            }
        }
    }
}

impl Default for HeuristicAutonomousPlanner {
    fn default() -> Self {
        Self {
            max_query_terms: DEFAULT_MAX_QUERY_TERMS,
        }
    }
}

impl AutonomousPlanner for HeuristicAutonomousPlanner {
    fn plan(&self, request: &AutonomousSearchRequest) -> Result<AutonomousPlannerTrace> {
        if request.planner_strategy.kind != AutonomousPlannerStrategyKind::Heuristic {
            bail!("heuristic planner requires the heuristic planner strategy");
        }

        let plan_outcome = self.build_plan_outcome(request);
        let mut trace = AutonomousPlannerTrace::new(request.planner_strategy.clone())
            .with_completed(true)
            .with_stop_reason(plan_outcome.stop_reason);
        if let Some(session_id) = request.session_id.clone() {
            trace = trace.with_session_id(session_id);
        }

        if plan_outcome.queries.is_empty() {
            return Ok(trace);
        }

        let mut steps = Vec::with_capacity(plan_outcome.queries.len());
        let mut previous_step_id = None;

        for (index, query) in plan_outcome.queries.iter().enumerate() {
            let sequence = request.state.current_step.sequence + index;
            let step = if index == 0 {
                request.state.current_step.clone()
            } else {
                let mut next_step =
                    AutonomousPlannerStepCursor::new(format!("step-{sequence}"), sequence);
                if let Some(parent_step_id) = previous_step_id.clone() {
                    next_step = next_step.with_parent_step_id(parent_step_id);
                }
                next_step
            };

            let mut decisions = vec![
                AutonomousPlannerDecision::new(AutonomousPlannerAction::Search)
                    .with_query(query.clone())
                    .with_turn_id(format!("turn-{sequence}"))
                    .with_rationale(if index == 0 {
                        "seed the first autonomous search from the root task and local context"
                    } else {
                        "follow retained evidence with a deduplicated heuristic query"
                    }),
            ];

            if index + 1 < plan_outcome.queries.len() {
                let next_sequence = sequence + 1;
                decisions.push(
                    AutonomousPlannerDecision::new(AutonomousPlannerAction::Continue)
                        .with_next_step(
                            AutonomousPlannerStepCursor::new(
                                format!("step-{next_sequence}"),
                                next_sequence,
                            )
                            .with_parent_step_id(step.step_id.clone()),
                        )
                        .with_rationale("advance the bounded linear planner to the next step"),
                );
            } else {
                decisions.push(
                    AutonomousPlannerDecision::new(AutonomousPlannerAction::Terminate)
                        .with_stop_reason(plan_outcome.stop_reason)
                        .with_rationale(Self::stop_rationale(plan_outcome.stop_reason)),
                );
            }

            previous_step_id = Some(step.step_id.clone());
            steps.push(AutonomousPlannerTraceStep::new(step).with_decisions(decisions));
        }

        trace.steps = steps;
        Ok(trace)
    }
}

fn append_terms(
    output: &mut Vec<String>,
    seen: &mut HashSet<String>,
    candidates: &[String],
    max_terms: usize,
) {
    for candidate in candidates {
        if output.len() >= max_terms || !seen.insert(candidate.clone()) {
            continue;
        }
        output.push(candidate.clone());
    }
}

fn significant_terms(text: &str) -> Vec<String> {
    tokenize(text)
        .into_iter()
        .filter(|token| is_significant(token))
        .collect()
}

fn fallback_terms(text: &str) -> Vec<String> {
    tokenize(text)
}

fn local_context_terms(source: &LocalContextSource) -> Vec<String> {
    match source {
        LocalContextSource::EnvironmentFact(input) => significant_terms(&input.value),
        LocalContextSource::ToolOutput(input) => significant_terms(&input.content),
        LocalContextSource::AgentTurn(input) => significant_terms(&input.content),
    }
}

fn fallback_local_context_terms(source: &LocalContextSource) -> Vec<String> {
    match source {
        LocalContextSource::EnvironmentFact(input) => fallback_terms(&input.value),
        LocalContextSource::ToolOutput(input) => fallback_terms(&input.content),
        LocalContextSource::AgentTurn(input) => fallback_terms(&input.content),
    }
}

fn retained_artifact_terms(artifact: &RetainedArtifact) -> Vec<String> {
    let mut terms = Vec::new();
    if let Some(snippet) = &artifact.snippet {
        terms.extend(significant_terms(snippet));
    }
    if let Some(location) = &artifact.location {
        terms.extend(significant_terms(location));
    }
    terms.extend(significant_terms(&artifact.path));
    terms
}

fn fallback_retained_artifact_terms(artifact: &RetainedArtifact) -> Vec<String> {
    let mut terms = Vec::new();
    if let Some(snippet) = &artifact.snippet {
        terms.extend(fallback_terms(snippet));
    }
    if let Some(location) = &artifact.location {
        terms.extend(fallback_terms(location));
    }
    terms.extend(fallback_terms(&artifact.path));
    terms
}

fn is_significant(token: &str) -> bool {
    token.len() > 1 && !token.chars().all(|ch| ch.is_ascii_digit()) && !STOPWORDS.contains(&token)
}

#[cfg(test)]
mod tests {
    use super::HeuristicAutonomousPlanner;
    use crate::search::{
        AcquisitionAdapterKind, ArtifactBudget, ArtifactFreshness, ArtifactProvenance,
        AutonomousPlanner, AutonomousPlannerAction, AutonomousPlannerStopReason,
        AutonomousSearchRequest, ContextArtifactKind, EnvironmentFactInput, LocalContextSource,
        RetainedArtifact, ToolOutputInput,
    };

    fn retained_artifact(
        artifact_id: &str,
        path: &str,
        snippet: &str,
        rationale: &str,
    ) -> RetainedArtifact {
        RetainedArtifact::new(
            artifact_id,
            ContextArtifactKind::File,
            path,
            ArtifactProvenance {
                adapter: AcquisitionAdapterKind::FileSystem,
                source: "test".to_string(),
                synthetic: false,
            },
            ArtifactFreshness {
                observed_unix_secs: 1,
                modified_unix_secs: Some(1),
            },
            ArtifactBudget::from_text(snippet, 1),
        )
        .with_snippet(snippet)
        .with_rationale(rationale)
    }

    #[test]
    fn heuristic_planner_seeds_initial_query_from_root_task_and_local_context() {
        let planner = HeuristicAutonomousPlanner::default();
        let request = AutonomousSearchRequest::new("./docs", "find cache invalidation")
            .with_local_context(vec![
                LocalContextSource::EnvironmentFact(EnvironmentFactInput::new("cwd", "src/cache")),
                LocalContextSource::ToolOutput(ToolOutputInput::new(
                    "rg",
                    "call-1",
                    "retry loop adapter",
                )),
            ]);

        let trace = planner.plan(&request).expect("heuristic planner trace");

        assert_eq!(trace.steps.len(), 1);
        assert_eq!(trace.steps[0].step.step_id, "step-1");
        assert_eq!(trace.steps[0].decisions.len(), 2);
        assert_eq!(
            trace.steps[0].decisions[0].action,
            AutonomousPlannerAction::Search
        );
        assert_eq!(
            trace.steps[0].decisions[1].action,
            AutonomousPlannerAction::Terminate
        );
        assert_eq!(
            trace.steps[0].decisions[0].query.as_deref(),
            Some("find cache invalidation retry loop adapter")
        );
        assert_eq!(
            trace.steps[0].decisions[1].stop_reason,
            Some(AutonomousPlannerStopReason::NoFurtherQueries)
        );
        assert!(trace.completed);
        assert_eq!(
            trace.stop_reason,
            Some(AutonomousPlannerStopReason::NoFurtherQueries)
        );
    }

    #[test]
    fn heuristic_planner_derives_deduplicated_follow_up_queries_from_retained_evidence() {
        let planner = HeuristicAutonomousPlanner::default();
        let request = AutonomousSearchRequest::new("./docs", "cache invalidation path")
            .with_step_limit(3)
            .with_state(
                crate::search::AutonomousPlannerState::new(3).with_retained_artifacts(vec![
                    retained_artifact(
                        "artifact-1",
                        "src/cache.rs",
                        "cache invalidation retry loop adapter layer",
                        "follow the retry loop implementation",
                    ),
                    retained_artifact(
                        "artifact-2",
                        "src/cache.rs",
                        "cache invalidation retry loop adapter layer",
                        "follow the retry loop implementation",
                    ),
                    retained_artifact(
                        "artifact-3",
                        "src/cache_state.rs",
                        "planner cursor evidence persistence",
                        "inspect retained evidence persistence",
                    ),
                ]),
            );

        let trace = planner.plan(&request).expect("heuristic planner trace");

        assert_eq!(trace.steps.len(), 3);
        assert_eq!(
            trace.steps[0].decisions[0].query.as_deref(),
            Some("cache invalidation path")
        );
        assert_eq!(
            trace.steps[1].decisions[0].query.as_deref(),
            Some("retry loop adapter layer")
        );
        assert_eq!(
            trace.steps[2].decisions[0].query.as_deref(),
            Some("planner cursor evidence persistence state")
        );
        assert_eq!(
            trace.steps[0].decisions[1].action,
            AutonomousPlannerAction::Continue
        );
        assert_eq!(
            trace.steps[1].decisions[1].action,
            AutonomousPlannerAction::Continue
        );
        assert_eq!(
            trace.steps[2].decisions[1].action,
            AutonomousPlannerAction::Terminate
        );
        assert_eq!(
            trace.steps[2].decisions[1].stop_reason,
            Some(AutonomousPlannerStopReason::NoAdditionalEvidence)
        );
        assert!(trace.completed);
        assert_eq!(
            trace.stop_reason,
            Some(AutonomousPlannerStopReason::NoAdditionalEvidence)
        );
    }

    #[test]
    fn heuristic_planner_terminates_with_step_limit_when_additional_queries_exist() {
        let planner = HeuristicAutonomousPlanner::default();
        let request = AutonomousSearchRequest::new("./docs", "cache invalidation path")
            .with_step_limit(2)
            .with_state(
                crate::search::AutonomousPlannerState::new(2).with_retained_artifacts(vec![
                    retained_artifact(
                        "artifact-1",
                        "src/cache.rs",
                        "cache invalidation retry loop adapter layer",
                        "follow the retry loop implementation",
                    ),
                    retained_artifact(
                        "artifact-2",
                        "src/cache_state.rs",
                        "planner cursor evidence persistence",
                        "inspect retained evidence persistence",
                    ),
                ]),
            );

        let trace = planner.plan(&request).expect("heuristic planner trace");

        assert_eq!(trace.steps.len(), 2);
        assert_eq!(
            trace.steps[1].decisions[1].action,
            AutonomousPlannerAction::Terminate
        );
        assert_eq!(
            trace.steps[1].decisions[1].stop_reason,
            Some(AutonomousPlannerStopReason::StepLimitReached)
        );
        assert!(trace.completed);
        assert_eq!(
            trace.stop_reason,
            Some(AutonomousPlannerStopReason::StepLimitReached)
        );
    }

    #[test]
    fn heuristic_planner_is_deterministic_for_same_request_and_evidence() {
        let planner = HeuristicAutonomousPlanner::default();
        let request = AutonomousSearchRequest::new("./docs", "cache invalidation path")
            .with_step_limit(3)
            .with_state(
                crate::search::AutonomousPlannerState::new(3).with_retained_artifacts(vec![
                    retained_artifact(
                        "artifact-1",
                        "src/cache.rs",
                        "cache invalidation retry loop adapter layer",
                        "follow the retry loop implementation",
                    ),
                    retained_artifact(
                        "artifact-2",
                        "src/cache_state.rs",
                        "planner cursor evidence persistence",
                        "inspect retained evidence persistence",
                    ),
                ]),
            );

        let first = planner
            .plan(&request)
            .expect("first heuristic planner trace");
        let second = planner
            .plan(&request)
            .expect("second heuristic planner trace");

        assert_eq!(first, second);
    }

    #[test]
    fn heuristic_planner_resumes_from_explicit_step_state_without_reseeding_root_query() {
        let planner = HeuristicAutonomousPlanner::default();
        let request = AutonomousSearchRequest::new("./docs", "find alpha details").with_state(
            crate::search::AutonomousPlannerState::new(3)
                .with_current_step(
                    crate::search::AutonomousPlannerStepCursor::new("step-2", 2)
                        .with_parent_step_id("step-1"),
                )
                .with_retained_artifacts(vec![retained_artifact(
                    "artifact-1",
                    "context/seed.txt",
                    "retry loop adapter layer",
                    "resume from retained evidence",
                )]),
        );

        let trace = planner.plan(&request).expect("heuristic planner trace");

        assert_eq!(trace.steps.len(), 1);
        assert_eq!(trace.steps[0].step.step_id, "step-2");
        assert_eq!(trace.steps[0].step.sequence, 2);
        assert_eq!(
            trace.steps[0].decisions[0].query.as_deref(),
            Some("retry loop adapter layer context seed")
        );
        assert_eq!(
            trace.steps[0].decisions[1].stop_reason,
            Some(AutonomousPlannerStopReason::NoAdditionalEvidence)
        );
    }
}
