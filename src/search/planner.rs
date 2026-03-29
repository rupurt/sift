use std::collections::HashSet;
use std::sync::Arc;

use anyhow::{Result, bail};
use serde::Deserialize;

use super::{
    AutonomousGraphBranchState, AutonomousGraphBranchStatus, AutonomousGraphEpisodeState,
    AutonomousGraphNode, AutonomousPlanner, AutonomousPlannerAction, AutonomousPlannerDecision,
    AutonomousPlannerStepCursor, AutonomousPlannerStopReason, AutonomousPlannerStrategyKind,
    AutonomousPlannerTrace, AutonomousPlannerTraceStep, AutonomousSearchMode,
    AutonomousSearchRequest, GenerativeModel, LocalContextSource, RetainedArtifact, tokenize,
};

const DEFAULT_MAX_QUERY_TERMS: usize = 6;
const DEFAULT_MAX_GRAPH_BRANCHES: usize = 3;
const STOPWORDS: &[&str] = &[
    "a", "an", "and", "are", "as", "at", "by", "for", "from", "how", "in", "into", "is", "json",
    "me", "md", "of", "on", "or", "rs", "src", "the", "toml", "to", "txt", "with", "yaml", "yml",
    "cwd",
];

#[derive(Debug, Clone)]
pub struct HeuristicAutonomousPlanner {
    max_query_terms: usize,
}

pub struct ModelDrivenAutonomousPlanner {
    model: Arc<dyn GenerativeModel>,
    max_tokens: usize,
}

#[derive(Debug, Clone)]
struct HeuristicPlanOutcome {
    queries: Vec<String>,
    stop_reason: AutonomousPlannerStopReason,
}

#[derive(Debug, Clone)]
struct HeuristicGraphCandidate {
    branch_id: String,
    node_id: String,
    frontier_id: String,
    edge_id: Option<String>,
    step: AutonomousPlannerStepCursor,
    query: String,
    priority: usize,
}

#[derive(Debug, Deserialize)]
struct ModelDrivenPlannerTracePayload {
    steps: Vec<AutonomousPlannerTraceStep>,
    #[serde(default)]
    completed: bool,
    stop_reason: Option<AutonomousPlannerStopReason>,
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

    fn graph_episode(&self, request: &AutonomousSearchRequest) -> AutonomousGraphEpisodeState {
        request
            .state
            .graph_episode
            .clone()
            .unwrap_or_else(|| Self::seed_graph_episode(request))
    }

    fn seed_graph_episode(request: &AutonomousSearchRequest) -> AutonomousGraphEpisodeState {
        let root_node_id = request.state.current_step.step_id.clone();
        let root_branch_id = "branch-root".to_string();

        AutonomousGraphEpisodeState::new()
            .with_root_node_id(root_node_id.clone())
            .with_active_branch_id(root_branch_id.clone())
            .with_nodes(vec![
                AutonomousGraphNode::new(
                    root_node_id.clone(),
                    root_branch_id.clone(),
                    request.state.current_step.clone(),
                )
                .with_query(request.root_task.clone()),
            ])
            .with_branches(vec![
                AutonomousGraphBranchState::new(root_branch_id, root_node_id)
                    .with_status(AutonomousGraphBranchStatus::Active)
                    .with_retained_artifacts(request.state.retained_artifacts.clone()),
            ])
    }

    fn active_graph_branch_id(
        &self,
        episode: &AutonomousGraphEpisodeState,
        request: &AutonomousSearchRequest,
    ) -> String {
        episode
            .active_branch_id
            .clone()
            .or_else(|| {
                episode
                    .branches
                    .first()
                    .map(|branch| branch.branch_id.clone())
            })
            .unwrap_or_else(|| {
                if request.state.current_step.step_id == "step-1" {
                    "branch-root".to_string()
                } else {
                    format!("branch-{}", request.state.current_step.sequence)
                }
            })
    }

    fn active_graph_node_id(
        &self,
        episode: &AutonomousGraphEpisodeState,
        branch_id: &str,
        request: &AutonomousSearchRequest,
    ) -> String {
        episode
            .branches
            .iter()
            .find(|branch| branch.branch_id == branch_id)
            .map(|branch| branch.head_node_id.clone())
            .unwrap_or_else(|| request.state.current_step.step_id.clone())
    }

    fn graph_query_from_branch(
        &self,
        request: &AutonomousSearchRequest,
        branch_id: &str,
        used_queries: &HashSet<String>,
        used_terms: &HashSet<String>,
        episode: &AutonomousGraphEpisodeState,
    ) -> Option<String> {
        let branch = episode
            .branches
            .iter()
            .find(|branch| branch.branch_id == branch_id)?;

        let node_query = episode
            .nodes
            .iter()
            .find(|node| node.node_id == branch.head_node_id)
            .and_then(|node| node.query.clone())
            .filter(|query| !used_queries.contains(query));
        if node_query.is_some() {
            return node_query;
        }

        for artifact in &branch.retained_artifacts {
            let query = self.build_follow_up_query(artifact, used_terms);
            if !query.is_empty() && !used_queries.contains(&query) {
                return Some(query);
            }
        }

        let seed_query = self.build_initial_query(request);
        if seed_query.is_empty() || used_queries.contains(&seed_query) {
            None
        } else {
            Some(seed_query)
        }
    }

    fn graph_frontier_candidates(
        &self,
        request: &AutonomousSearchRequest,
        episode: &AutonomousGraphEpisodeState,
    ) -> Vec<HeuristicGraphCandidate> {
        let mut used_queries = HashSet::new();
        let mut used_terms = HashSet::new();
        let mut frontier = episode.frontier.clone();
        frontier.sort_by(|left, right| {
            left.priority
                .cmp(&right.priority)
                .then_with(|| left.frontier_id.cmp(&right.frontier_id))
        });

        frontier
            .into_iter()
            .filter_map(|entry| {
                let query = self.graph_query_from_branch(
                    request,
                    &entry.branch_id,
                    &used_queries,
                    &used_terms,
                    episode,
                )?;
                used_queries.insert(query.clone());
                used_terms.extend(query.split_whitespace().map(str::to_string));
                let step = episode
                    .nodes
                    .iter()
                    .find(|node| node.node_id == entry.node_id)
                    .map(|node| node.step.clone())
                    .unwrap_or_else(|| request.state.current_step.clone());

                Some(HeuristicGraphCandidate {
                    branch_id: entry.branch_id,
                    node_id: entry.node_id,
                    frontier_id: entry.frontier_id,
                    edge_id: None,
                    step,
                    query,
                    priority: entry.priority,
                })
            })
            .collect()
    }

    fn graph_seed_candidates(
        &self,
        request: &AutonomousSearchRequest,
        episode: &AutonomousGraphEpisodeState,
    ) -> Vec<HeuristicGraphCandidate> {
        let mut queries = Vec::new();
        let initial_query = self.build_initial_query(request);
        if !initial_query.is_empty() {
            queries.push(initial_query);
        }

        let active_branch_id = self.active_graph_branch_id(episode, request);
        if let Some(branch) = episode
            .branches
            .iter()
            .find(|branch| branch.branch_id == active_branch_id)
        {
            let mut used_terms = queries
                .iter()
                .flat_map(|query| query.split_whitespace().map(str::to_string))
                .collect::<HashSet<_>>();
            for artifact in &branch.retained_artifacts {
                let follow_up = self.build_follow_up_query(artifact, &used_terms);
                if follow_up.is_empty() || queries.contains(&follow_up) {
                    continue;
                }
                used_terms.extend(follow_up.split_whitespace().map(str::to_string));
                queries.push(follow_up);
                if queries.len() >= DEFAULT_MAX_GRAPH_BRANCHES {
                    break;
                }
            }
        }

        if queries.is_empty() {
            return Vec::new();
        }

        let source_branch_id = self.active_graph_branch_id(episode, request);
        let source_node_id = self.active_graph_node_id(episode, &source_branch_id, request);
        let base_branch = episode.branches.len() + 1;
        let base_node = episode.nodes.len() + 1;
        let base_edge = episode.edges.len() + 1;

        queries
            .into_iter()
            .enumerate()
            .map(|(index, query)| {
                let candidate_index = index + 1;
                let branch_id = format!("branch-{}", base_branch + index);
                let node_id = format!("node-{}", base_node + index);
                let frontier_id = format!("frontier-{}", base_branch + index);
                let step = AutonomousPlannerStepCursor::new(
                    node_id.clone(),
                    request.state.current_step.sequence + candidate_index,
                )
                .with_parent_step_id(source_node_id.clone());

                HeuristicGraphCandidate {
                    branch_id,
                    node_id,
                    frontier_id,
                    edge_id: Some(format!("edge-{}", base_edge + index)),
                    step,
                    query,
                    priority: index,
                }
            })
            .collect()
    }

    fn graph_stop_reason(
        &self,
        request: &AutonomousSearchRequest,
        candidate_count: usize,
        executed_count: usize,
        episode: &AutonomousGraphEpisodeState,
    ) -> AutonomousPlannerStopReason {
        if request.state.step_limit == 0 || candidate_count > executed_count {
            AutonomousPlannerStopReason::StepLimitReached
        } else if executed_count == 0 {
            if episode.frontier.is_empty() && request.state.retained_artifacts.is_empty() {
                AutonomousPlannerStopReason::NoFurtherQueries
            } else {
                AutonomousPlannerStopReason::NoAdditionalEvidence
            }
        } else if episode.frontier.is_empty() && request.state.retained_artifacts.is_empty() {
            AutonomousPlannerStopReason::NoFurtherQueries
        } else {
            AutonomousPlannerStopReason::NoAdditionalEvidence
        }
    }

    fn build_graph_trace(&self, request: &AutonomousSearchRequest) -> AutonomousPlannerTrace {
        let episode = self.graph_episode(request);
        let active_branch_id = self.active_graph_branch_id(&episode, request);
        let active_node_id = self.active_graph_node_id(&episode, &active_branch_id, request);
        let mut candidates = self.graph_frontier_candidates(request, &episode);
        let seeded_from_root = candidates.is_empty();
        if seeded_from_root {
            candidates = self.graph_seed_candidates(request, &episode);
        }

        let mut trace =
            AutonomousPlannerTrace::new(request.planner_strategy.clone()).with_completed(true);
        if let Some(session_id) = request.session_id.clone() {
            trace = trace.with_session_id(session_id);
        }
        if request.state.step_limit == 0 {
            return trace.with_stop_reason(AutonomousPlannerStopReason::StepLimitReached);
        }
        if candidates.is_empty() {
            return trace.with_stop_reason(self.graph_stop_reason(request, 0, 0, &episode));
        }

        candidates.sort_by(|left, right| {
            left.priority
                .cmp(&right.priority)
                .then_with(|| left.branch_id.cmp(&right.branch_id))
        });

        let candidate_count = candidates.len();
        let selected = candidates
            .into_iter()
            .take(request.state.step_limit)
            .collect::<Vec<_>>();
        let stop_reason = self.graph_stop_reason(request, candidate_count, selected.len(), &episode);
        let mut steps = Vec::new();

        let first = &selected[0];
        let mut first_decisions = Vec::new();
        if seeded_from_root {
            for candidate in &selected {
                first_decisions.push(
                    AutonomousPlannerDecision::new(AutonomousPlannerAction::Fork)
                        .with_branch_id(active_branch_id.clone())
                        .with_node_id(active_node_id.clone())
                        .with_target_branch_id(candidate.branch_id.clone())
                        .with_target_node_id(candidate.node_id.clone())
                        .with_edge_id(
                            candidate
                                .edge_id
                                .clone()
                                .unwrap_or_else(|| format!("edge-{}", candidate.step.sequence)),
                        )
                        .with_edge_kind(super::AutonomousGraphEdgeKind::Child)
                        .with_frontier_id(candidate.frontier_id.clone())
                        .with_query(candidate.query.clone())
                        .with_next_step(candidate.step.clone())
                        .with_rationale(if candidate.priority == 0 {
                            "fork the highest-priority graph branch from the root task"
                        } else {
                            "fork an additional bounded branch to preserve graph optionality"
                        }),
                );
            }
        }
        first_decisions.push(
            AutonomousPlannerDecision::new(AutonomousPlannerAction::Select)
                .with_branch_id(first.branch_id.clone())
                .with_node_id(first.node_id.clone())
                .with_frontier_id(first.frontier_id.clone())
                .with_rationale("select the highest-priority graph frontier branch"),
        );
        first_decisions.push(
            AutonomousPlannerDecision::new(AutonomousPlannerAction::Search)
                .with_branch_id(first.branch_id.clone())
                .with_node_id(first.node_id.clone())
                .with_query(first.query.clone())
                .with_turn_id("turn-1")
                .with_rationale("execute the highest-priority graph branch query"),
        );
        if selected.len() > 1 {
            first_decisions.push(
                AutonomousPlannerDecision::new(AutonomousPlannerAction::Continue)
                    .with_next_step(selected[1].step.clone())
                    .with_rationale("continue to the next prioritized graph branch"),
            );
        } else {
            first_decisions.push(
                AutonomousPlannerDecision::new(AutonomousPlannerAction::Terminate)
                    .with_branch_id(first.branch_id.clone())
                    .with_stop_reason(stop_reason)
                    .with_rationale(Self::stop_rationale(stop_reason)),
            );
        }
        steps.push(
            AutonomousPlannerTraceStep::new(request.state.current_step.clone())
                .with_decisions(first_decisions),
        );

        for (index, candidate) in selected.iter().enumerate().skip(1) {
            let mut decisions = vec![
                AutonomousPlannerDecision::new(AutonomousPlannerAction::Select)
                    .with_branch_id(candidate.branch_id.clone())
                    .with_node_id(candidate.node_id.clone())
                    .with_frontier_id(candidate.frontier_id.clone())
                    .with_rationale("select the next bounded graph frontier branch"),
                AutonomousPlannerDecision::new(AutonomousPlannerAction::Search)
                    .with_branch_id(candidate.branch_id.clone())
                    .with_node_id(candidate.node_id.clone())
                    .with_query(candidate.query.clone())
                    .with_turn_id(format!("turn-{}", index + 1))
                    .with_rationale("execute the selected graph branch query"),
            ];

            if index + 1 < selected.len() {
                decisions.push(
                    AutonomousPlannerDecision::new(AutonomousPlannerAction::Continue)
                        .with_next_step(selected[index + 1].step.clone())
                        .with_rationale("continue to the next prioritized graph branch"),
                );
            } else {
                decisions.push(
                    AutonomousPlannerDecision::new(AutonomousPlannerAction::Terminate)
                        .with_branch_id(candidate.branch_id.clone())
                        .with_stop_reason(stop_reason)
                        .with_rationale(Self::stop_rationale(stop_reason)),
                );
            }

            steps.push(
                AutonomousPlannerTraceStep::new(candidate.step.clone()).with_decisions(decisions),
            );
        }

        trace.with_steps(steps).with_stop_reason(stop_reason)
    }
}

impl ModelDrivenAutonomousPlanner {
    pub fn new(model: Arc<dyn GenerativeModel>) -> Self {
        Self {
            model,
            max_tokens: 512,
        }
    }

    pub fn with_max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_tokens = max_tokens.max(1);
        self
    }

    fn build_prompt(&self, request: &AutonomousSearchRequest) -> String {
        if request.mode == AutonomousSearchMode::Graph {
            return self.build_graph_prompt(request);
        }

        let local_context = request
            .local_context
            .iter()
            .map(render_local_context)
            .collect::<Vec<_>>()
            .join("\n");
        let retained_evidence = request
            .state
            .retained_artifacts
            .iter()
            .map(render_retained_artifact)
            .collect::<Vec<_>>()
            .join("\n");
        let profile = request
            .planner_strategy
            .profile
            .as_deref()
            .unwrap_or("default");

        format!(
            concat!(
                "You are a local autonomous planning adapter for Sift.\n",
                "Return JSON only with this shape:\n",
                "{{\"steps\":[{{\"step\":{{\"step_id\":\"step-1\",\"parent_step_id\":null,\"sequence\":1}},",
                "\"decisions\":[{{\"action\":\"search\",\"query\":\"...\",\"turn_id\":\"turn-1\",",
                "\"rationale\":\"...\",\"next_step\":null,\"stop_reason\":null}},",
                "{{\"action\":\"terminate\",\"rationale\":\"...\",\"query\":null,\"turn_id\":null,",
                "\"next_step\":null,\"stop_reason\":\"no-further-queries\"}}]}}],",
                "\"completed\":true,\"stop_reason\":\"no-further-queries\"}}\n",
                "Keep planning bounded and linear. Use only search, continue, or terminate decisions.\n",
                "Task: {task}\n",
                "Planner profile: {profile}\n",
                "Current step: id={step_id} sequence={sequence}\n",
                "Step limit: {step_limit}\n",
                "Local context:\n{local_context}\n",
                "Retained evidence:\n{retained_evidence}\n"
            ),
            task = request.root_task,
            profile = profile,
            step_id = request.state.current_step.step_id,
            sequence = request.state.current_step.sequence,
            step_limit = request.state.step_limit,
            local_context = if local_context.is_empty() {
                "(none)".to_string()
            } else {
                local_context
            },
            retained_evidence = if retained_evidence.is_empty() {
                "(none)".to_string()
            } else {
                retained_evidence
            },
        )
    }

    fn build_graph_prompt(&self, request: &AutonomousSearchRequest) -> String {
        let profile = request
            .planner_strategy
            .profile
            .as_deref()
            .unwrap_or("default");
        let graph_episode = request
            .state
            .graph_episode
            .as_ref()
            .map(|episode| serde_json::to_string_pretty(episode).unwrap_or_default())
            .unwrap_or_else(|| "null".to_string());

        format!(
            concat!(
                "You are a local graph-planning adapter for Sift.\n",
                "Return JSON only with this shape:\n",
                "{{\"steps\":[{{\"step\":{{\"step_id\":\"step-1\",\"parent_step_id\":null,\"sequence\":1}},",
                "\"decisions\":[{{\"action\":\"fork\",\"rationale\":\"...\",\"query\":\"...\",\"turn_id\":null,",
                "\"branch_id\":\"branch-root\",\"node_id\":\"step-1\",\"target_branch_id\":\"branch-2\",",
                "\"target_node_id\":\"node-2\",\"edge_id\":\"edge-2\",\"edge_kind\":\"child\",",
                "\"frontier_id\":\"frontier-2\",\"next_step\":{{\"step_id\":\"node-2\",\"parent_step_id\":\"step-1\",\"sequence\":2}},",
                "\"stop_reason\":null}},",
                "{{\"action\":\"select\",\"rationale\":\"...\",\"query\":null,\"turn_id\":null,",
                "\"branch_id\":\"branch-2\",\"node_id\":\"node-2\",\"target_branch_id\":null,",
                "\"target_node_id\":null,\"edge_id\":null,\"edge_kind\":null,\"frontier_id\":\"frontier-2\",",
                "\"next_step\":null,\"stop_reason\":null}},",
                "{{\"action\":\"search\",\"rationale\":\"...\",\"query\":\"cache invalidation path\",\"turn_id\":\"turn-1\",",
                "\"branch_id\":\"branch-2\",\"node_id\":\"node-2\",\"target_branch_id\":null,",
                "\"target_node_id\":null,\"edge_id\":null,\"edge_kind\":null,\"frontier_id\":null,",
                "\"next_step\":null,\"stop_reason\":null}},",
                "{{\"action\":\"terminate\",\"rationale\":\"...\",\"query\":null,\"turn_id\":null,",
                "\"branch_id\":\"branch-2\",\"node_id\":null,\"target_branch_id\":null,\"target_node_id\":null,",
                "\"edge_id\":null,\"edge_kind\":null,\"frontier_id\":null,\"next_step\":null,",
                "\"stop_reason\":\"goal-satisfied\"}}]}}],\"completed\":true,\"stop_reason\":\"goal-satisfied\"}}\n",
                "Keep planning bounded and graph-shaped. Use only fork, select, search, merge, prune, continue, or terminate decisions.\n",
                "Task: {task}\n",
                "Planner profile: {profile}\n",
                "Step limit: {step_limit}\n",
                "Graph episode state:\n{graph_episode}\n"
            ),
            task = request.root_task,
            profile = profile,
            step_limit = request.state.step_limit,
            graph_episode = graph_episode,
        )
    }

    fn extract_json(raw: &str) -> &str {
        let trimmed = raw.trim();
        if let Some(stripped) = trimmed.strip_prefix("```json") {
            return stripped
                .strip_suffix("```")
                .map(str::trim)
                .unwrap_or(trimmed);
        }
        if let Some(stripped) = trimmed.strip_prefix("```") {
            return stripped
                .strip_suffix("```")
                .map(str::trim)
                .unwrap_or(trimmed);
        }
        trimmed
    }
}

impl Default for HeuristicAutonomousPlanner {
    fn default() -> Self {
        Self {
            max_query_terms: DEFAULT_MAX_QUERY_TERMS,
        }
    }
}

impl AutonomousPlanner for ModelDrivenAutonomousPlanner {
    fn plan(&self, request: &AutonomousSearchRequest) -> Result<AutonomousPlannerTrace> {
        if request.planner_strategy.kind != AutonomousPlannerStrategyKind::ModelDriven {
            bail!("model-driven planner requires the model-driven planner strategy");
        }

        let payload: ModelDrivenPlannerTracePayload = serde_json::from_str(Self::extract_json(
            &self
                .model
                .generate(&self.build_prompt(request), self.max_tokens)?,
        ))?;

        let mut trace = AutonomousPlannerTrace::new(request.planner_strategy.clone())
            .with_steps(payload.steps)
            .with_completed(payload.completed);
        if let Some(stop_reason) = payload.stop_reason {
            trace = trace.with_stop_reason(stop_reason);
        }
        if let Some(session_id) = request.session_id.clone() {
            trace = trace.with_session_id(session_id);
        }
        if trace.steps.len() > request.state.step_limit {
            bail!("model-driven planner trace exceeded the configured autonomous step limit");
        }

        Ok(trace)
    }
}

impl AutonomousPlanner for HeuristicAutonomousPlanner {
    fn plan(&self, request: &AutonomousSearchRequest) -> Result<AutonomousPlannerTrace> {
        if request.planner_strategy.kind != AutonomousPlannerStrategyKind::Heuristic {
            bail!("heuristic planner requires the heuristic planner strategy");
        }

        if request.mode == AutonomousSearchMode::Graph {
            return Ok(self.build_graph_trace(request));
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

fn render_local_context(source: &LocalContextSource) -> String {
    match source {
        LocalContextSource::EnvironmentFact(input) => format!("env:{}={}", input.key, input.value),
        LocalContextSource::ToolOutput(input) => {
            format!("tool:{} {}", input.tool_name, input.content)
        }
        LocalContextSource::AgentTurn(input) => format!("turn:{} {}", input.role, input.content),
    }
}

fn render_retained_artifact(artifact: &RetainedArtifact) -> String {
    format!(
        "artifact:{} path={} snippet={}",
        artifact.artifact_id,
        artifact.path,
        artifact.snippet.as_deref().unwrap_or("")
    )
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use anyhow::Result;

    use super::{HeuristicAutonomousPlanner, ModelDrivenAutonomousPlanner};
    use crate::search::{
        AcquisitionAdapterKind, ArtifactBudget, ArtifactFreshness, ArtifactProvenance,
        AutonomousPlanner, AutonomousPlannerAction, AutonomousPlannerStopReason,
        AutonomousPlannerStrategy, AutonomousSearchRequest, ContextArtifactKind, Conversation,
        EnvironmentFactInput, GenerativeModel, LocalContextSource, RetainedArtifact,
        ToolOutputInput,
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

    struct EmptyConversation;

    impl Conversation for EmptyConversation {
        fn send(&mut self, _message: &str, _max_tokens: usize) -> Result<String> {
            Ok(String::new())
        }

        fn history(&self) -> &[String] {
            &[]
        }
    }

    struct StaticGenerativeModel {
        output: String,
    }

    impl GenerativeModel for StaticGenerativeModel {
        fn generate(&self, _prompt: &str, _max_tokens: usize) -> Result<String> {
            Ok(self.output.clone())
        }

        fn start_conversation(&self) -> Result<Box<dyn Conversation>> {
            Ok(Box::new(EmptyConversation))
        }
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

    #[test]
    fn model_driven_planner_parses_shared_autonomous_trace_shape() {
        let planner = ModelDrivenAutonomousPlanner::new(Arc::new(StaticGenerativeModel {
            output: r#"{
                "steps": [
                    {
                        "step": {
                            "step_id": "step-1",
                            "parent_step_id": null,
                            "sequence": 1
                        },
                        "decisions": [
                            {
                                "action": "search",
                                "rationale": "model-driven planner selected the most salient token",
                                "query": "alpha runtime details",
                                "turn_id": "turn-md-1",
                                "next_step": null,
                                "stop_reason": null
                            },
                            {
                                "action": "terminate",
                                "rationale": "the task is answered after the first turn",
                                "query": null,
                                "turn_id": null,
                                "next_step": null,
                                "stop_reason": "goal-satisfied"
                            }
                        ]
                    }
                ],
                "completed": true,
                "stop_reason": "goal-satisfied"
            }"#
            .to_string(),
        }));
        let request = AutonomousSearchRequest::new("./docs", "find alpha details")
            .with_planner_strategy(
                AutonomousPlannerStrategy::model_driven().with_profile("local-planner-v1"),
            );

        let trace = planner.plan(&request).expect("model-driven planner trace");

        assert_eq!(trace.planner_strategy, request.planner_strategy);
        assert_eq!(trace.steps.len(), 1);
        assert_eq!(
            trace.steps[0].decisions[0].action,
            AutonomousPlannerAction::Search
        );
        assert_eq!(
            trace.steps[0].decisions[1].stop_reason,
            Some(AutonomousPlannerStopReason::GoalSatisfied)
        );
    }

    #[test]
    fn model_driven_planner_requires_model_driven_strategy() {
        let planner = ModelDrivenAutonomousPlanner::new(Arc::new(StaticGenerativeModel {
            output: "{}".to_string(),
        }));
        let request = AutonomousSearchRequest::new("./docs", "find alpha details");

        let error = planner
            .plan(&request)
            .expect_err("heuristic request should not route through model-driven planner");

        assert!(
            error
                .to_string()
                .contains("model-driven planner requires the model-driven planner strategy")
        );
    }
}
