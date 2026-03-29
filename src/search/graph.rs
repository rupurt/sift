use std::collections::HashSet;
use std::fmt;

use super::{
    AutonomousGraphBranchState, AutonomousGraphBranchStatus, AutonomousGraphEdge,
    AutonomousGraphEpisodeState, AutonomousGraphFrontierEntry, AutonomousGraphNode,
    AutonomousPlannerAction, AutonomousPlannerDecision, AutonomousPlannerStepCursor,
    AutonomousPlannerTrace,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutonomousGraphTraceContractErrorKind {
    MissingBranchReference,
    MissingNodeReference,
    MissingEdgeReference,
    MissingFrontierReference,
    DuplicateBranchReference,
    DuplicateNodeReference,
    DuplicateEdgeReference,
    DuplicateFrontierReference,
    InvalidBranchNodePair,
    InvalidEdgeReference,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutonomousGraphTraceContractError {
    pub kind: AutonomousGraphTraceContractErrorKind,
    pub step_id: Option<String>,
    pub detail: String,
}

impl AutonomousGraphTraceContractError {
    fn new(
        kind: AutonomousGraphTraceContractErrorKind,
        step: Option<&AutonomousPlannerStepCursor>,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            step_id: step.map(|step| step.step_id.clone()),
            detail: detail.into(),
        }
    }
}

impl fmt::Display for AutonomousGraphTraceContractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(step_id) = &self.step_id {
            write!(
                f,
                "graph trace contract error at {step_id}: {}",
                self.detail
            )
        } else {
            write!(f, "graph trace contract error: {}", self.detail)
        }
    }
}

impl std::error::Error for AutonomousGraphTraceContractError {}

pub fn replay_graph_trace(
    initial_state: &AutonomousGraphEpisodeState,
    trace: &AutonomousPlannerTrace,
) -> Result<AutonomousGraphEpisodeState, AutonomousGraphTraceContractError> {
    validate_graph_state(initial_state, None)?;

    let mut state = initial_state.clone();
    for step in &trace.steps {
        for decision in &step.decisions {
            replay_graph_decision(&mut state, &step.step, decision)?;
        }
    }

    if trace.completed || trace.stop_reason.is_some() {
        state.completed = true;
    }

    validate_graph_state(&state, None)?;
    Ok(state)
}

pub fn replay_graph_decision(
    state: &mut AutonomousGraphEpisodeState,
    step: &AutonomousPlannerStepCursor,
    decision: &AutonomousPlannerDecision,
) -> Result<(), AutonomousGraphTraceContractError> {
    match decision.action {
        AutonomousPlannerAction::Search => {
            let branch_id = require_field(
                decision.branch_id.as_deref(),
                AutonomousGraphTraceContractErrorKind::MissingBranchReference,
                step,
                "search decisions must reference an existing branch_id",
            )?;
            let node_id = require_field(
                decision.node_id.as_deref(),
                AutonomousGraphTraceContractErrorKind::MissingNodeReference,
                step,
                "search decisions must reference an existing node_id",
            )?;

            let branch_index = branch_index(state, branch_id).ok_or_else(|| {
                AutonomousGraphTraceContractError::new(
                    AutonomousGraphTraceContractErrorKind::MissingBranchReference,
                    Some(step),
                    format!("search decision references missing branch '{branch_id}'"),
                )
            })?;
            let node_index = node_index(state, node_id).ok_or_else(|| {
                AutonomousGraphTraceContractError::new(
                    AutonomousGraphTraceContractErrorKind::MissingNodeReference,
                    Some(step),
                    format!("search decision references missing node '{node_id}'"),
                )
            })?;

            if state.nodes[node_index].branch_id != branch_id {
                return Err(AutonomousGraphTraceContractError::new(
                    AutonomousGraphTraceContractErrorKind::InvalidBranchNodePair,
                    Some(step),
                    format!(
                        "search decision references node '{}' that does not belong to branch '{}'",
                        node_id, branch_id
                    ),
                ));
            }

            state.active_branch_id = Some(branch_id.to_string());
            state.branches[branch_index].status = AutonomousGraphBranchStatus::Active;
            state.branches[branch_index].head_node_id = node_id.to_string();
            if state.root_node_id.is_none() {
                state.root_node_id = Some(node_id.to_string());
            }
            if state.nodes[node_index].query.is_none() {
                state.nodes[node_index].query = decision.query.clone();
            }
            if state.nodes[node_index].turn_id.is_none() {
                state.nodes[node_index].turn_id = decision.turn_id.clone();
            }
        }
        AutonomousPlannerAction::Select => {
            let branch_id = require_field(
                decision.branch_id.as_deref(),
                AutonomousGraphTraceContractErrorKind::MissingBranchReference,
                step,
                "select decisions must reference branch_id",
            )?;
            let node_id = require_field(
                decision.node_id.as_deref(),
                AutonomousGraphTraceContractErrorKind::MissingNodeReference,
                step,
                "select decisions must reference node_id",
            )?;
            let frontier_id = require_field(
                decision.frontier_id.as_deref(),
                AutonomousGraphTraceContractErrorKind::MissingFrontierReference,
                step,
                "select decisions must reference frontier_id",
            )?;
            let frontier_index = frontier_index(state, frontier_id).ok_or_else(|| {
                AutonomousGraphTraceContractError::new(
                    AutonomousGraphTraceContractErrorKind::MissingFrontierReference,
                    Some(step),
                    format!("select decision references missing frontier '{frontier_id}'"),
                )
            })?;
            let frontier = &state.frontier[frontier_index];
            if frontier.branch_id != branch_id || frontier.node_id != node_id {
                return Err(AutonomousGraphTraceContractError::new(
                    AutonomousGraphTraceContractErrorKind::InvalidBranchNodePair,
                    Some(step),
                    format!(
                        "frontier '{}' points at branch '{}' node '{}' rather than branch '{}' node '{}'",
                        frontier_id, frontier.branch_id, frontier.node_id, branch_id, node_id
                    ),
                ));
            }
            let branch_index = branch_index(state, branch_id).ok_or_else(|| {
                AutonomousGraphTraceContractError::new(
                    AutonomousGraphTraceContractErrorKind::MissingBranchReference,
                    Some(step),
                    format!("select decision references missing branch '{branch_id}'"),
                )
            })?;

            state.frontier.remove(frontier_index);
            state.active_branch_id = Some(branch_id.to_string());
            state.branches[branch_index].status = AutonomousGraphBranchStatus::Active;
            state.branches[branch_index].head_node_id = node_id.to_string();
        }
        AutonomousPlannerAction::Fork => {
            let branch_id = require_field(
                decision.branch_id.as_deref(),
                AutonomousGraphTraceContractErrorKind::MissingBranchReference,
                step,
                "fork decisions must reference a parent branch_id",
            )?;
            let node_id = require_field(
                decision.node_id.as_deref(),
                AutonomousGraphTraceContractErrorKind::MissingNodeReference,
                step,
                "fork decisions must reference a parent node_id",
            )?;
            let target_branch_id = require_field(
                decision.target_branch_id.as_deref(),
                AutonomousGraphTraceContractErrorKind::MissingBranchReference,
                step,
                "fork decisions must reference a target branch_id",
            )?;
            let target_node_id = require_field(
                decision.target_node_id.as_deref(),
                AutonomousGraphTraceContractErrorKind::MissingNodeReference,
                step,
                "fork decisions must reference a target node_id",
            )?;
            let edge_id = require_field(
                decision.edge_id.as_deref(),
                AutonomousGraphTraceContractErrorKind::MissingEdgeReference,
                step,
                "fork decisions must reference an edge_id",
            )?;
            let frontier_id = require_field(
                decision.frontier_id.as_deref(),
                AutonomousGraphTraceContractErrorKind::MissingFrontierReference,
                step,
                "fork decisions must reference a frontier_id",
            )?;
            let edge_kind = decision.edge_kind.ok_or_else(|| {
                AutonomousGraphTraceContractError::new(
                    AutonomousGraphTraceContractErrorKind::MissingEdgeReference,
                    Some(step),
                    "fork decisions must declare an explicit edge kind",
                )
            })?;

            ensure_branch_exists(state, branch_id, step, "fork")?;
            ensure_node_exists(state, node_id, step, "fork")?;
            let parent_branch_index =
                branch_index(state, branch_id).expect("validated parent branch");

            if branch_index(state, target_branch_id).is_some() {
                return Err(AutonomousGraphTraceContractError::new(
                    AutonomousGraphTraceContractErrorKind::DuplicateBranchReference,
                    Some(step),
                    format!("fork decision reuses existing branch '{target_branch_id}'"),
                ));
            }
            if node_index(state, target_node_id).is_some() {
                return Err(AutonomousGraphTraceContractError::new(
                    AutonomousGraphTraceContractErrorKind::DuplicateNodeReference,
                    Some(step),
                    format!("fork decision reuses existing node '{target_node_id}'"),
                ));
            }
            if edge_index(state, edge_id).is_some() {
                return Err(AutonomousGraphTraceContractError::new(
                    AutonomousGraphTraceContractErrorKind::DuplicateEdgeReference,
                    Some(step),
                    format!("fork decision reuses existing edge '{edge_id}'"),
                ));
            }
            if frontier_index(state, frontier_id).is_some() {
                return Err(AutonomousGraphTraceContractError::new(
                    AutonomousGraphTraceContractErrorKind::DuplicateFrontierReference,
                    Some(step),
                    format!("fork decision reuses existing frontier entry '{frontier_id}'"),
                ));
            }

            let next_step = decision.next_step.clone().unwrap_or_else(|| {
                AutonomousPlannerStepCursor::new(target_node_id, step.sequence + 1)
                    .with_parent_step_id(step.step_id.clone())
            });
            state.nodes.push(
                AutonomousGraphNode::new(target_node_id, target_branch_id, next_step).with_query(
                    decision
                        .query
                        .clone()
                        .unwrap_or_else(|| target_node_id.to_string()),
                ),
            );
            state.edges.push(AutonomousGraphEdge::new(
                edge_id,
                node_id,
                target_node_id,
                edge_kind,
            ));
            state.branches.push(
                AutonomousGraphBranchState::new(target_branch_id, target_node_id)
                    .with_status(AutonomousGraphBranchStatus::Pending)
                    .with_retained_artifacts(
                        state.branches[parent_branch_index]
                            .retained_artifacts
                            .clone(),
                    ),
            );
            state.frontier.push(AutonomousGraphFrontierEntry::new(
                frontier_id,
                target_branch_id,
                target_node_id,
            ));
        }
        AutonomousPlannerAction::Merge => {
            let branch_id = require_field(
                decision.branch_id.as_deref(),
                AutonomousGraphTraceContractErrorKind::MissingBranchReference,
                step,
                "merge decisions must reference a source branch_id",
            )?;
            let target_branch_id = require_field(
                decision.target_branch_id.as_deref(),
                AutonomousGraphTraceContractErrorKind::MissingBranchReference,
                step,
                "merge decisions must reference a target branch_id",
            )?;
            let branch_index = ensure_branch_exists(state, branch_id, step, "merge")?;
            let target_index = ensure_branch_exists(state, target_branch_id, step, "merge")?;

            if branch_id == target_branch_id {
                return Err(AutonomousGraphTraceContractError::new(
                    AutonomousGraphTraceContractErrorKind::InvalidBranchNodePair,
                    Some(step),
                    "merge decisions must target a different branch",
                ));
            }

            let merged_artifacts = state.branches[target_index].retained_artifacts.clone();
            state.branches[branch_index]
                .retained_artifacts
                .extend(merged_artifacts);
            state.branches[target_index].status = AutonomousGraphBranchStatus::Merged;
            state
                .frontier
                .retain(|entry| entry.branch_id != target_branch_id);

            if let (Some(edge_id), Some(node_id), Some(target_node_id), Some(edge_kind)) = (
                decision.edge_id.as_deref(),
                decision.node_id.as_deref(),
                decision.target_node_id.as_deref(),
                decision.edge_kind,
            ) {
                if edge_index(state, edge_id).is_some() {
                    return Err(AutonomousGraphTraceContractError::new(
                        AutonomousGraphTraceContractErrorKind::DuplicateEdgeReference,
                        Some(step),
                        format!("merge decision reuses existing edge '{edge_id}'"),
                    ));
                }
                ensure_node_exists(state, node_id, step, "merge")?;
                ensure_node_exists(state, target_node_id, step, "merge")?;
                state.edges.push(AutonomousGraphEdge::new(
                    edge_id,
                    node_id,
                    target_node_id,
                    edge_kind,
                ));
            }
        }
        AutonomousPlannerAction::Prune => {
            let target_branch_id = decision
                .target_branch_id
                .as_deref()
                .or(decision.branch_id.as_deref())
                .ok_or_else(|| {
                    AutonomousGraphTraceContractError::new(
                        AutonomousGraphTraceContractErrorKind::MissingBranchReference,
                        Some(step),
                        "prune decisions must reference a branch_id or target_branch_id",
                    )
                })?;
            let branch_index = ensure_branch_exists(state, target_branch_id, step, "prune")?;
            state.branches[branch_index].status = AutonomousGraphBranchStatus::Pruned;
            state
                .frontier
                .retain(|entry| entry.branch_id != target_branch_id);
            if state.active_branch_id.as_deref() == Some(target_branch_id) {
                state.active_branch_id = None;
            }
        }
        AutonomousPlannerAction::Continue => {}
        AutonomousPlannerAction::Terminate => {
            if let Some(branch_id) = decision.branch_id.as_deref()
                && let Some(branch_index) = branch_index(state, branch_id)
            {
                state.branches[branch_index].status = AutonomousGraphBranchStatus::Completed;
            }
            state.completed = true;
        }
    }

    Ok(())
}

fn validate_graph_state(
    state: &AutonomousGraphEpisodeState,
    step: Option<&AutonomousPlannerStepCursor>,
) -> Result<(), AutonomousGraphTraceContractError> {
    let mut branch_ids = HashSet::new();
    for branch in &state.branches {
        if !branch_ids.insert(branch.branch_id.as_str()) {
            return Err(AutonomousGraphTraceContractError::new(
                AutonomousGraphTraceContractErrorKind::DuplicateBranchReference,
                step,
                format!("graph episode reuses branch '{}'", branch.branch_id),
            ));
        }
    }

    let mut node_ids = HashSet::new();
    for node in &state.nodes {
        if !node_ids.insert(node.node_id.as_str()) {
            return Err(AutonomousGraphTraceContractError::new(
                AutonomousGraphTraceContractErrorKind::DuplicateNodeReference,
                step,
                format!("graph episode reuses node '{}'", node.node_id),
            ));
        }
        if !branch_ids.contains(node.branch_id.as_str()) {
            return Err(AutonomousGraphTraceContractError::new(
                AutonomousGraphTraceContractErrorKind::MissingBranchReference,
                step,
                format!(
                    "graph node '{}' references missing branch '{}'",
                    node.node_id, node.branch_id
                ),
            ));
        }
    }

    let mut edge_ids = HashSet::new();
    for edge in &state.edges {
        if !edge_ids.insert(edge.edge_id.as_str()) {
            return Err(AutonomousGraphTraceContractError::new(
                AutonomousGraphTraceContractErrorKind::DuplicateEdgeReference,
                step,
                format!("graph episode reuses edge '{}'", edge.edge_id),
            ));
        }
        if !node_ids.contains(edge.from_node_id.as_str())
            || !node_ids.contains(edge.to_node_id.as_str())
        {
            return Err(AutonomousGraphTraceContractError::new(
                AutonomousGraphTraceContractErrorKind::InvalidEdgeReference,
                step,
                format!(
                    "graph edge '{}' references missing node endpoints '{}' -> '{}'",
                    edge.edge_id, edge.from_node_id, edge.to_node_id
                ),
            ));
        }
    }

    let mut frontier_ids = HashSet::new();
    for frontier in &state.frontier {
        if !frontier_ids.insert(frontier.frontier_id.as_str()) {
            return Err(AutonomousGraphTraceContractError::new(
                AutonomousGraphTraceContractErrorKind::DuplicateFrontierReference,
                step,
                format!(
                    "graph episode reuses frontier entry '{}'",
                    frontier.frontier_id
                ),
            ));
        }
        if !branch_ids.contains(frontier.branch_id.as_str()) {
            return Err(AutonomousGraphTraceContractError::new(
                AutonomousGraphTraceContractErrorKind::MissingBranchReference,
                step,
                format!(
                    "frontier entry '{}' references missing branch '{}'",
                    frontier.frontier_id, frontier.branch_id
                ),
            ));
        }
        if !node_ids.contains(frontier.node_id.as_str()) {
            return Err(AutonomousGraphTraceContractError::new(
                AutonomousGraphTraceContractErrorKind::MissingNodeReference,
                step,
                format!(
                    "frontier entry '{}' references missing node '{}'",
                    frontier.frontier_id, frontier.node_id
                ),
            ));
        }
    }

    if let Some(root_node_id) = state.root_node_id.as_deref()
        && !node_ids.contains(root_node_id)
    {
        return Err(AutonomousGraphTraceContractError::new(
            AutonomousGraphTraceContractErrorKind::MissingNodeReference,
            step,
            format!("graph episode root references missing node '{root_node_id}'"),
        ));
    }

    if let Some(active_branch_id) = state.active_branch_id.as_deref()
        && !branch_ids.contains(active_branch_id)
    {
        return Err(AutonomousGraphTraceContractError::new(
            AutonomousGraphTraceContractErrorKind::MissingBranchReference,
            step,
            format!("graph episode active branch references missing '{active_branch_id}'"),
        ));
    }

    Ok(())
}

fn require_field<'a>(
    value: Option<&'a str>,
    kind: AutonomousGraphTraceContractErrorKind,
    step: &AutonomousPlannerStepCursor,
    detail: &str,
) -> Result<&'a str, AutonomousGraphTraceContractError> {
    value.ok_or_else(|| AutonomousGraphTraceContractError::new(kind, Some(step), detail))
}

fn ensure_branch_exists(
    state: &AutonomousGraphEpisodeState,
    branch_id: &str,
    step: &AutonomousPlannerStepCursor,
    action: &str,
) -> Result<usize, AutonomousGraphTraceContractError> {
    branch_index(state, branch_id).ok_or_else(|| {
        AutonomousGraphTraceContractError::new(
            AutonomousGraphTraceContractErrorKind::MissingBranchReference,
            Some(step),
            format!("{action} decision references missing branch '{branch_id}'"),
        )
    })
}

fn ensure_node_exists(
    state: &AutonomousGraphEpisodeState,
    node_id: &str,
    step: &AutonomousPlannerStepCursor,
    action: &str,
) -> Result<usize, AutonomousGraphTraceContractError> {
    node_index(state, node_id).ok_or_else(|| {
        AutonomousGraphTraceContractError::new(
            AutonomousGraphTraceContractErrorKind::MissingNodeReference,
            Some(step),
            format!("{action} decision references missing node '{node_id}'"),
        )
    })
}

fn branch_index(state: &AutonomousGraphEpisodeState, branch_id: &str) -> Option<usize> {
    state
        .branches
        .iter()
        .position(|branch| branch.branch_id == branch_id)
}

fn node_index(state: &AutonomousGraphEpisodeState, node_id: &str) -> Option<usize> {
    state.nodes.iter().position(|node| node.node_id == node_id)
}

fn edge_index(state: &AutonomousGraphEpisodeState, edge_id: &str) -> Option<usize> {
    state.edges.iter().position(|edge| edge.edge_id == edge_id)
}

fn frontier_index(state: &AutonomousGraphEpisodeState, frontier_id: &str) -> Option<usize> {
    state
        .frontier
        .iter()
        .position(|frontier| frontier.frontier_id == frontier_id)
}

#[cfg(test)]
mod tests {
    use super::replay_graph_trace;
    use crate::search::{
        AutonomousGraphBranchState, AutonomousGraphBranchStatus, AutonomousGraphEdgeKind,
        AutonomousGraphEpisodeState, AutonomousGraphNode, AutonomousPlannerAction,
        AutonomousPlannerDecision, AutonomousPlannerStepCursor, AutonomousPlannerStrategy,
        AutonomousPlannerTrace, AutonomousPlannerTraceStep,
    };

    #[test]
    fn replay_graph_trace_advances_frontier_state_deterministically() {
        let initial_state = AutonomousGraphEpisodeState::new()
            .with_root_node_id("node-root")
            .with_active_branch_id("branch-root")
            .with_nodes(vec![
                AutonomousGraphNode::new(
                    "node-root",
                    "branch-root",
                    AutonomousPlannerStepCursor::first(),
                )
                .with_query("root task"),
            ])
            .with_branches(vec![
                AutonomousGraphBranchState::new("branch-root", "node-root")
                    .with_status(AutonomousGraphBranchStatus::Active),
            ]);
        let trace = AutonomousPlannerTrace::new(AutonomousPlannerStrategy::heuristic())
            .with_steps(vec![
                AutonomousPlannerTraceStep::new(AutonomousPlannerStepCursor::first())
                    .with_decisions(vec![
                        AutonomousPlannerDecision::new(AutonomousPlannerAction::Fork)
                            .with_branch_id("branch-root")
                            .with_node_id("node-root")
                            .with_target_branch_id("branch-a")
                            .with_target_node_id("node-a")
                            .with_edge_id("edge-root-a")
                            .with_edge_kind(AutonomousGraphEdgeKind::Child)
                            .with_frontier_id("frontier-a")
                            .with_query("cache invalidation path")
                            .with_next_step(
                                AutonomousPlannerStepCursor::new("node-a", 2)
                                    .with_parent_step_id("step-1"),
                            ),
                        AutonomousPlannerDecision::new(AutonomousPlannerAction::Select)
                            .with_branch_id("branch-a")
                            .with_node_id("node-a")
                            .with_frontier_id("frontier-a"),
                        AutonomousPlannerDecision::new(AutonomousPlannerAction::Search)
                            .with_branch_id("branch-a")
                            .with_node_id("node-a")
                            .with_query("cache invalidation path")
                            .with_turn_id("turn-1"),
                        AutonomousPlannerDecision::new(AutonomousPlannerAction::Terminate)
                            .with_branch_id("branch-a"),
                    ]),
            ])
            .with_completed(true);

        let replayed = replay_graph_trace(&initial_state, &trace).expect("replay graph trace");

        assert_eq!(replayed.root_node_id.as_deref(), Some("node-root"));
        assert_eq!(replayed.active_branch_id.as_deref(), Some("branch-a"));
        assert!(replayed.frontier.is_empty());
        assert_eq!(replayed.nodes.len(), 2);
        assert_eq!(replayed.edges.len(), 1);
        assert!(replayed.completed);
    }

    #[test]
    fn replay_graph_trace_rejects_missing_frontier_references() {
        let initial_state = AutonomousGraphEpisodeState::new()
            .with_root_node_id("node-root")
            .with_nodes(vec![AutonomousGraphNode::new(
                "node-root",
                "branch-root",
                AutonomousPlannerStepCursor::first(),
            )])
            .with_branches(vec![AutonomousGraphBranchState::new(
                "branch-root",
                "node-root",
            )]);
        let trace =
            AutonomousPlannerTrace::new(AutonomousPlannerStrategy::heuristic()).with_steps(vec![
                AutonomousPlannerTraceStep::new(AutonomousPlannerStepCursor::first())
                    .with_decisions(vec![
                        AutonomousPlannerDecision::new(AutonomousPlannerAction::Select)
                            .with_branch_id("branch-root")
                            .with_node_id("node-root")
                            .with_frontier_id("frontier-missing"),
                    ]),
            ]);

        let error = replay_graph_trace(&initial_state, &trace).expect_err("missing frontier");
        assert!(
            error
                .to_string()
                .contains("missing frontier 'frontier-missing'")
        );
    }
}
