use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum GraphError {
    #[error("Cycle detected in workflow graph involving node {0}")]
    CycleDetected(Uuid),

    #[error("Node not found: {0}")]
    NodeNotFound(Uuid),

    #[error("Invalid edge: from_node {from} port '{from_port}' does not exist")]
    InvalidFromPort { from: Uuid, from_port: String },

    #[error("Invalid edge: to_node {to} port '{to_port}' does not exist")]
    InvalidToPort { to: Uuid, to_port: String },

    #[error("Multiple triggers found - workflow must have exactly one trigger node")]
    MultipleTriggers,

    #[error("No trigger node found in workflow")]
    NoTrigger,

    #[error("Disconnected subgraph detected - all nodes must be reachable from trigger")]
    DisconnectedSubgraph,

    #[error("Duplicate edge: {from} -> {to} already connected on these ports")]
    DuplicateEdge { from: Uuid, to: Uuid },
}

pub type GraphResult<T> = std::result::Result<T, GraphError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowNode {
    pub id: Uuid,
    pub node_type: NodeType,
    pub label: String,
    pub position: NodePosition,
    pub config: serde_json::Value,
    pub inputs: Vec<PortDef>,
    pub outputs: Vec<PortDef>,
}

impl WorkflowNode {
    pub fn new(node_type: NodeType, label: impl Into<String>) -> Self {
        let (inputs, outputs) = node_type.default_ports();
        Self {
            id: Uuid::new_v4(),
            node_type,
            label: label.into(),
            position: NodePosition::default(),
            config: serde_json::Value::Object(Default::default()),
            inputs,
            outputs,
        }
    }

    pub fn has_input_port(&self, name: &str) -> bool {
        self.inputs.iter().any(|p| p.name == name)
    }

    pub fn has_output_port(&self, name: &str) -> bool {
        self.outputs.iter().any(|p| p.name == name)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodePosition {
    pub x: f32,
    pub y: f32,
}

impl Default for NodePosition {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NodeType {
    Trigger(TriggerNodeConfig),
    SkillCall {
        skill_id: Uuid,
        params: serde_json::Value,
    },
    Transform {
        expression: String,
    },
    Condition {
        predicate: String,
    },
    Loop {
        count_expr: String,
    },
    SubWorkflow {
        workflow_id: Uuid,
    },
    HttpRequest {
        url: String,
        method: HttpMethod,
    },
    Delay {
        duration_ms: u64,
    },
    Merge,
    Split,
    ErrorHandler {
        strategy: ErrorStrategy,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerNodeConfig {
    pub trigger_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Patch => "PATCH",
            Self::Delete => "DELETE",
        };
        write!(f, "{s}")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "strategy", rename_all = "snake_case")]
pub enum ErrorStrategy {
    StopOnError,
    SkipAndContinue,
    RetryWithBackoff {
        max_retries: u32,
        base_delay_ms: u64,
    },
}

impl NodeType {
    pub fn default_ports(&self) -> (Vec<PortDef>, Vec<PortDef>) {
        let any_in = vec![PortDef::new("input", DataType::Any, false)];
        let any_out = vec![PortDef::new("output", DataType::Any, false)];
        let exec_in = vec![PortDef::new("exec", DataType::Any, false)];
        let exec_out = vec![PortDef::new("exec", DataType::Any, false)];

        match self {
            Self::Trigger(_) => (vec![], any_out),
            Self::SkillCall { .. } => (any_in.clone(), any_out.clone()),
            Self::Transform { .. } => (any_in.clone(), any_out.clone()),
            Self::Condition { .. } => (
                any_in,
                vec![
                    PortDef::new("true", DataType::Any, false),
                    PortDef::new("false", DataType::Any, false),
                ],
            ),
            Self::Loop { .. } => (
                exec_in.clone(),
                vec![
                    PortDef::new("body", DataType::Any, false),
                    PortDef::new("done", DataType::Any, false),
                ],
            ),
            Self::SubWorkflow { .. } => (any_in.clone(), any_out.clone()),
            Self::HttpRequest { .. } => (
                vec![PortDef::new("trigger", DataType::Any, false)],
                vec![
                    PortDef::new("response", DataType::Object, false),
                    PortDef::new("error", DataType::Object, false),
                ],
            ),
            Self::Delay { .. } => (exec_in.clone(), exec_out.clone()),
            Self::Merge => (
                vec![
                    PortDef::new("a", DataType::Any, false),
                    PortDef::new("b", DataType::Any, false),
                ],
                any_out,
            ),
            Self::Split => (
                any_in,
                vec![
                    PortDef::new("a", DataType::Any, false),
                    PortDef::new("b", DataType::Any, false),
                ],
            ),
            Self::ErrorHandler { .. } => (
                vec![PortDef::new("error", DataType::Object, true)],
                exec_out,
            ),
        }
    }

    pub fn is_trigger(&self) -> bool {
        matches!(self, Self::Trigger(_))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortDef {
    pub name: String,
    pub data_type: DataType,
    pub required: bool,
}

impl PortDef {
    pub fn new(name: impl Into<String>, data_type: DataType, required: bool) -> Self {
        Self {
            name: name.into(),
            data_type,
            required,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DataType {
    String,
    Number,
    Boolean,
    Object,
    Array,
    Any,
}

impl DataType {
    pub fn is_compatible_with(&self, other: &DataType) -> bool {
        *self == DataType::Any || *other == DataType::Any || self == other
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowEdge {
    pub id: Uuid,
    pub from_node: Uuid,
    pub from_port: String,
    pub to_node: Uuid,
    pub to_port: String,
    pub label: Option<String>,
}

impl WorkflowEdge {
    pub fn new(
        from_node: Uuid,
        from_port: impl Into<String>,
        to_node: Uuid,
        to_port: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            from_node,
            from_port: from_port.into(),
            to_node,
            to_port: to_port.into(),
            label: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub version: u32,
    pub nodes: Vec<WorkflowNode>,
    pub edges: Vec<WorkflowEdge>,
    pub triggers: Vec<crate::triggers::TriggerConfig>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_active: bool,
}

impl Workflow {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: description.into(),
            version: 1,
            nodes: Vec::new(),
            edges: Vec::new(),
            triggers: Vec::new(),
            created_at: now,
            updated_at: now,
            is_active: false,
        }
    }
}

/// Validated, executable DAG representation of a workflow
pub struct WorkflowGraph {
    nodes: HashMap<Uuid, WorkflowNode>,
    adjacency: HashMap<Uuid, Vec<Uuid>>,
    in_degree: HashMap<Uuid, usize>,
    edges: Vec<WorkflowEdge>,
}

impl WorkflowGraph {
    pub fn build(workflow: &Workflow) -> GraphResult<Self> {
        let nodes: HashMap<Uuid, WorkflowNode> =
            workflow.nodes.iter().map(|n| (n.id, n.clone())).collect();

        // Validate all edge endpoints exist and ports are defined
        let mut adjacency: HashMap<Uuid, Vec<Uuid>> =
            nodes.keys().map(|&id| (id, Vec::new())).collect();
        let mut in_degree: HashMap<Uuid, usize> = nodes.keys().map(|&id| (id, 0)).collect();
        let mut seen_edges: HashSet<(Uuid, String, Uuid, String)> = HashSet::new();

        for edge in &workflow.edges {
            let from_node = nodes
                .get(&edge.from_node)
                .ok_or(GraphError::NodeNotFound(edge.from_node))?;
            let to_node = nodes
                .get(&edge.to_node)
                .ok_or(GraphError::NodeNotFound(edge.to_node))?;

            if !from_node.has_output_port(&edge.from_port) {
                return Err(GraphError::InvalidFromPort {
                    from: edge.from_node,
                    from_port: edge.from_port.clone(),
                });
            }
            if !to_node.has_input_port(&edge.to_port) {
                return Err(GraphError::InvalidToPort {
                    to: edge.to_node,
                    to_port: edge.to_port.clone(),
                });
            }

            let edge_key = (
                edge.from_node,
                edge.from_port.clone(),
                edge.to_node,
                edge.to_port.clone(),
            );
            if seen_edges.contains(&edge_key) {
                return Err(GraphError::DuplicateEdge {
                    from: edge.from_node,
                    to: edge.to_node,
                });
            }
            seen_edges.insert(edge_key);

            adjacency
                .entry(edge.from_node)
                .or_default()
                .push(edge.to_node);
            *in_degree.entry(edge.to_node).or_insert(0) += 1;
        }

        let graph = Self {
            nodes,
            adjacency,
            in_degree,
            edges: workflow.edges.clone(),
        };

        // Detect cycles
        let order = graph.topological_sort()?;

        // Check all nodes are reachable
        if order.len() != workflow.nodes.len() {
            return Err(GraphError::DisconnectedSubgraph);
        }

        Ok(graph)
    }

    /// Kahn's algorithm for topological sort - also detects cycles
    pub fn topological_sort(&self) -> GraphResult<Vec<Uuid>> {
        let mut in_degree = self.in_degree.clone();
        let mut queue: VecDeque<Uuid> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&id, _)| id)
            .collect();

        let mut order = Vec::with_capacity(self.nodes.len());

        while let Some(node_id) = queue.pop_front() {
            order.push(node_id);

            if let Some(neighbors) = self.adjacency.get(&node_id) {
                for &neighbor in neighbors {
                    let deg = in_degree.entry(neighbor).or_insert(0);
                    *deg = deg.saturating_sub(1);
                    if *deg == 0 {
                        queue.push_back(neighbor);
                    }
                }
            }
        }

        if order.len() != self.nodes.len() {
            // Find a node still with positive in-degree to report
            let cycle_node = in_degree
                .iter()
                .find(|(_, &d)| d > 0)
                .map(|(&id, _)| id)
                .unwrap_or_else(Uuid::nil);
            return Err(GraphError::CycleDetected(cycle_node));
        }

        Ok(order)
    }

    /// Returns sets of nodes that can execute concurrently (same topological depth)
    pub fn execution_levels(&self) -> GraphResult<Vec<Vec<Uuid>>> {
        let order = self.topological_sort()?;
        let mut depth: HashMap<Uuid, usize> = HashMap::new();

        for &id in &order {
            let max_pred_depth = self
                .edges
                .iter()
                .filter(|e| e.to_node == id)
                .map(|e| depth.get(&e.from_node).copied().unwrap_or(0) + 1)
                .max()
                .unwrap_or(0);
            depth.insert(id, max_pred_depth);
        }

        let max_depth = depth.values().copied().max().unwrap_or(0);
        let mut levels: Vec<Vec<Uuid>> = vec![Vec::new(); max_depth + 1];
        for (&id, &d) in &depth {
            levels[d].push(id);
        }

        Ok(levels)
    }

    pub fn node(&self, id: Uuid) -> Option<&WorkflowNode> {
        self.nodes.get(&id)
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn successors(&self, id: Uuid) -> &[Uuid] {
        self.adjacency.get(&id).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub fn edges_from(&self, node_id: Uuid) -> impl Iterator<Item = &WorkflowEdge> {
        self.edges.iter().filter(move |e| e.from_node == node_id)
    }
}
