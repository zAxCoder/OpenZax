use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum PlannerError {
    #[error("Node not found: {0}")]
    NodeNotFound(Uuid),
    #[error("Cycle detected in plan DAG")]
    CycleDetected,
    #[error("Max depth exceeded")]
    MaxDepthExceeded,
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Execution error in node {0}: {1}")]
    ExecutionError(Uuid, String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub tool_name: String,
    pub parameters: serde_json::Value,
    pub requires_approval: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "state")]
pub enum PlanNodeStatus {
    Pending,
    Executing,
    Completed { result: serde_json::Value },
    Failed { error: String },
    Pruned,
}

impl PlanNodeStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            PlanNodeStatus::Completed { .. } | PlanNodeStatus::Failed { .. } | PlanNodeStatus::Pruned
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanNode {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub thought: String,
    pub action: Option<Action>,
    pub estimated_cost: f32,
    pub expected_outcome: String,
    pub children: Vec<Uuid>,
    pub status: PlanNodeStatus,
    /// Estimated probability of success [0.0, 1.0]
    pub score: f32,
}

impl PlanNode {
    pub fn new(
        parent_id: Option<Uuid>,
        thought: impl Into<String>,
        expected_outcome: impl Into<String>,
        estimated_cost: f32,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            parent_id,
            thought: thought.into(),
            action: None,
            estimated_cost,
            expected_outcome: expected_outcome.into(),
            children: vec![],
            status: PlanNodeStatus::Pending,
            score: 0.5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanDAG {
    pub nodes: HashMap<Uuid, PlanNode>,
    pub root_id: Uuid,
    pub max_depth: u8,
    pub branching_factor: u8,
    pub goal: String,
}

impl PlanDAG {
    pub fn depth_of(&self, node_id: &Uuid) -> u8 {
        let mut depth = 0u8;
        let mut current = *node_id;
        while let Some(node) = self.nodes.get(&current) {
            if let Some(parent_id) = node.parent_id {
                depth += 1;
                current = parent_id;
            } else {
                break;
            }
        }
        depth
    }

    pub fn ancestors(&self, node_id: &Uuid) -> Vec<Uuid> {
        let mut result = vec![];
        let mut current = *node_id;
        while let Some(node) = self.nodes.get(&current) {
            if let Some(pid) = node.parent_id {
                result.push(pid);
                current = pid;
            } else {
                break;
            }
        }
        result
    }

    /// Topological order using iterative DFS (leaf-first = execution order)
    pub fn topological_order(&self) -> Result<Vec<Uuid>, PlannerError> {
        let mut visited = std::collections::HashSet::new();
        let mut stack = vec![];
        let mut order = vec![];

        fn dfs(
            dag: &PlanDAG,
            id: Uuid,
            visited: &mut std::collections::HashSet<Uuid>,
            stack: &mut Vec<Uuid>,
            order: &mut Vec<Uuid>,
        ) -> Result<(), PlannerError> {
            if stack.contains(&id) {
                return Err(PlannerError::CycleDetected);
            }
            if visited.contains(&id) {
                return Ok(());
            }
            stack.push(id);
            let node =
                dag.nodes.get(&id).ok_or(PlannerError::NodeNotFound(id))?;
            for &child_id in &node.children {
                dfs(dag, child_id, visited, stack, order)?;
            }
            stack.pop();
            visited.insert(id);
            order.push(id);
            Ok(())
        }

        dfs(self, self.root_id, &mut visited, &mut stack, &mut order)?;
        order.reverse();
        Ok(order)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanApprovalPolicy {
    pub auto_approve_if_score_above: f32,
    pub require_approval_for_file_writes: bool,
    pub require_approval_for_network: bool,
}

impl Default for PlanApprovalPolicy {
    fn default() -> Self {
        Self {
            auto_approve_if_score_above: 0.85,
            require_approval_for_file_writes: true,
            require_approval_for_network: false,
        }
    }
}

pub struct PlanningEngine {
    pub approval_policy: PlanApprovalPolicy,
}

impl PlanningEngine {
    pub fn new(approval_policy: PlanApprovalPolicy) -> Self {
        Self { approval_policy }
    }

    pub fn create_plan(
        &self,
        goal: &str,
        context: &str,
        constraints: &[&str],
        max_depth: u8,
        branching_factor: u8,
    ) -> Result<PlanDAG, PlannerError> {
        let root = PlanNode {
            id: Uuid::new_v4(),
            parent_id: None,
            thought: format!("Achieve: {} | Context: {} | Constraints: {}", goal, context, constraints.join(", ")),
            action: None,
            estimated_cost: 0.0,
            expected_outcome: goal.to_string(),
            children: vec![],
            status: PlanNodeStatus::Pending,
            score: 0.8,
        };

        let root_id = root.id;
        let mut dag = PlanDAG {
            nodes: HashMap::from([(root_id, root)]),
            root_id,
            max_depth,
            branching_factor,
            goal: goal.to_string(),
        };

        // Generate initial plan branches from the goal
        let initial_thoughts = self.decompose_goal(goal, context, constraints, branching_factor);
        for (thought, outcome, cost, action) in initial_thoughts {
            let mut node = PlanNode::new(Some(root_id), thought, outcome, cost);
            node.action = action;
            node.score = self.estimate_node_score(&node, &dag);
            let node_id = node.id;
            dag.nodes.get_mut(&root_id).unwrap().children.push(node_id);
            dag.nodes.insert(node_id, node);
        }

        Ok(dag)
    }

    pub fn expand_node(
        &self,
        dag: &mut PlanDAG,
        node_id: Uuid,
        alternatives: u8,
    ) -> Result<Vec<Uuid>, PlannerError> {
        let depth = dag.depth_of(&node_id);
        if depth >= dag.max_depth {
            return Err(PlannerError::MaxDepthExceeded);
        }

        let parent_thought = dag
            .nodes
            .get(&node_id)
            .ok_or(PlannerError::NodeNotFound(node_id))?
            .thought
            .clone();

        let mut new_ids = vec![];
        for i in 0..alternatives {
            let thought = format!("Alternative {} for: {}", i + 1, parent_thought);
            let outcome = format!("Expected outcome of alternative {}", i + 1);
            let cost = 0.1 * (i as f32 + 1.0);
            let mut node = PlanNode::new(Some(node_id), thought, outcome, cost);
            node.score = self.estimate_node_score(&node, dag);
            let child_id = node.id;
            dag.nodes.insert(child_id, node);
            dag.nodes
                .get_mut(&node_id)
                .ok_or(PlannerError::NodeNotFound(node_id))?
                .children
                .push(child_id);
            new_ids.push(child_id);
        }
        Ok(new_ids)
    }

    pub async fn execute_dag<F, Fut>(
        &self,
        dag: &mut PlanDAG,
        executor: F,
    ) -> Result<serde_json::Value, PlannerError>
    where
        F: Fn(Action) -> Fut,
        Fut: std::future::Future<Output = Result<serde_json::Value, String>>,
    {
        let order = dag.topological_order()?;
        let mut last_result = serde_json::Value::Null;

        for node_id in order {
            let node = dag.nodes.get(&node_id).ok_or(PlannerError::NodeNotFound(node_id))?;
            if matches!(node.status, PlanNodeStatus::Pruned) {
                continue;
            }

            if let Some(action) = node.action.clone() {
                dag.nodes.get_mut(&node_id).unwrap().status = PlanNodeStatus::Executing;
                let start = std::time::Instant::now();
                match executor(action).await {
                    Ok(result) => {
                        tracing::debug!(
                            "Node {} completed in {:?}",
                            node_id,
                            start.elapsed()
                        );
                        last_result = result.clone();
                        dag.nodes.get_mut(&node_id).unwrap().status =
                            PlanNodeStatus::Completed { result };
                    }
                    Err(error) => {
                        tracing::warn!("Node {} failed: {}", node_id, error);
                        dag.nodes.get_mut(&node_id).unwrap().status =
                            PlanNodeStatus::Failed { error: error.clone() };
                        return Err(PlannerError::ExecutionError(node_id, error));
                    }
                }
            } else {
                dag.nodes.get_mut(&node_id).unwrap().status =
                    PlanNodeStatus::Completed { result: serde_json::Value::Null };
            }
        }

        Ok(last_result)
    }

    pub fn replan_on_failure(
        &self,
        dag: &mut PlanDAG,
        failed_node_id: Uuid,
        error: &str,
    ) -> Result<Vec<Uuid>, PlannerError> {
        // Prune the failed node's subtree
        let failed_subtree = self.collect_subtree(dag, failed_node_id);
        for node_id in &failed_subtree {
            if let Some(node) = dag.nodes.get_mut(node_id) {
                node.status = PlanNodeStatus::Pruned;
            }
        }

        // Find the parent of the failed node to attach the alternative plan
        let parent_id = dag
            .nodes
            .get(&failed_node_id)
            .ok_or(PlannerError::NodeNotFound(failed_node_id))?
            .parent_id;

        let attach_to = parent_id.unwrap_or(dag.root_id);
        let recovery_thought = format!("Recovery from failure '{}': retry with fallback approach", error);
        let mut recovery_node = PlanNode::new(
            Some(attach_to),
            recovery_thought,
            "Recover from previous error and complete the task",
            0.5,
        );
        recovery_node.score = 0.6;
        let recovery_id = recovery_node.id;

        dag.nodes.insert(recovery_id, recovery_node);
        dag.nodes
            .get_mut(&attach_to)
            .ok_or(PlannerError::NodeNotFound(attach_to))?
            .children
            .push(recovery_id);

        Ok(vec![recovery_id])
    }

    pub fn prune_unlikely(&self, dag: &mut PlanDAG, threshold: f32) -> usize {
        let to_prune: Vec<Uuid> = dag
            .nodes
            .iter()
            .filter(|(id, node)| {
                **id != dag.root_id
                    && node.score < threshold
                    && matches!(node.status, PlanNodeStatus::Pending)
            })
            .map(|(id, _)| *id)
            .collect();

        let count = to_prune.len();
        for id in to_prune {
            if let Some(node) = dag.nodes.get_mut(&id) {
                node.status = PlanNodeStatus::Pruned;
            }
        }
        tracing::debug!("Pruned {} unlikely nodes (threshold={})", count, threshold);
        count
    }

    pub fn approve_plan(&self, dag: &PlanDAG) -> bool {
        let needs_approval = dag.nodes.values().any(|node| {
            if let Some(action) = &node.action {
                if action.requires_approval {
                    return true;
                }
                if self.approval_policy.require_approval_for_file_writes
                    && (action.tool_name.contains("write") || action.tool_name.contains("delete"))
                {
                    return true;
                }
                if self.approval_policy.require_approval_for_network
                    && (action.tool_name.contains("http") || action.tool_name.contains("request"))
                {
                    return true;
                }
            }
            false
        });

        let avg_score: f32 = if dag.nodes.is_empty() {
            0.0
        } else {
            dag.nodes.values().map(|n| n.score).sum::<f32>() / dag.nodes.len() as f32
        };

        needs_approval || avg_score < self.approval_policy.auto_approve_if_score_above
    }

    fn collect_subtree(&self, dag: &PlanDAG, root: Uuid) -> Vec<Uuid> {
        let mut result = vec![root];
        let mut queue = vec![root];
        while let Some(id) = queue.pop() {
            if let Some(node) = dag.nodes.get(&id) {
                for &child in &node.children {
                    result.push(child);
                    queue.push(child);
                }
            }
        }
        result
    }

    fn decompose_goal(
        &self,
        goal: &str,
        _context: &str,
        _constraints: &[&str],
        count: u8,
    ) -> Vec<(String, String, f32, Option<Action>)> {
        (0..count)
            .map(|i| {
                (
                    format!("Step {}: approach {} for '{}'", i + 1, i + 1, goal),
                    format!("Partial completion of '{}'", goal),
                    0.1 * (i as f32 + 1.0),
                    None,
                )
            })
            .collect()
    }

    fn estimate_node_score(&self, node: &PlanNode, _dag: &PlanDAG) -> f32 {
        // Simple heuristic: nodes with lower cost and clear outcomes score higher
        let cost_factor = (1.0 - node.estimated_cost.min(1.0)).max(0.0);
        let has_outcome = if node.expected_outcome.is_empty() { 0.3 } else { 0.7 };
        (cost_factor * 0.4 + has_outcome * 0.6).min(1.0)
    }
}
