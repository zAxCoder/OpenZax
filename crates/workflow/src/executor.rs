use chrono::Utc;
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{
    graph::{ErrorStrategy, NodeType, WorkflowGraph, WorkflowNode},
    registry::WorkflowRegistry,
    triggers::TriggerEvent,
};

#[derive(Debug, Error)]
pub enum ExecutionError {
    #[error("Workflow not found: {0}")]
    WorkflowNotFound(Uuid),

    #[error("Node execution failed at {node_id}: {reason}")]
    NodeFailed { node_id: Uuid, reason: String },

    #[error("Graph build error: {0}")]
    GraphError(#[from] crate::graph::GraphError),

    #[error("HTTP request failed: {0}")]
    HttpError(String),

    #[error("Sub-workflow error: {0}")]
    SubWorkflowError(String),

    #[error("Max retries exceeded for node {0}")]
    MaxRetriesExceeded(Uuid),

    #[error("Execution timed out after {0}ms")]
    Timeout(u64),

    #[error("Registry error: {0}")]
    Registry(#[from] crate::registry::RegistryError),
}

pub type ExecResult<T> = std::result::Result<T, ExecutionError>;

#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub run_id: Uuid,
    pub workflow_id: Uuid,
    pub trigger_event: TriggerEvent,
    pub node_outputs: HashMap<Uuid, serde_json::Value>,
    pub error_count: u32,
    pub started_at: chrono::DateTime<Utc>,
}

impl ExecutionContext {
    pub fn new(workflow_id: Uuid, trigger_event: TriggerEvent) -> Self {
        Self {
            run_id: Uuid::new_v4(),
            workflow_id,
            trigger_event,
            node_outputs: HashMap::new(),
            error_count: 0,
            started_at: Utc::now(),
        }
    }

    pub fn get_input_for_node(&self, graph: &WorkflowGraph, node_id: Uuid) -> serde_json::Value {
        // Gather all outputs from predecessor nodes via edges
        let mut inputs = serde_json::json!({});
        for edge in graph.edges_from(node_id) {
            if let Some(output) = self.node_outputs.get(&edge.from_node) {
                if let serde_json::Value::Object(map) = &mut inputs {
                    map.insert(edge.from_port.clone(), output.clone());
                }
            }
        }

        if inputs == serde_json::json!({}) {
            // No predecessors - use trigger payload as input
            self.trigger_event.payload.clone()
        } else {
            inputs
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub run_id: Uuid,
    pub workflow_id: Uuid,
    pub success: bool,
    pub output: serde_json::Value,
    pub error_message: Option<String>,
    pub duration_ms: u64,
    pub nodes_executed: u32,
}

/// Core workflow execution engine
pub struct WorkflowExecutor {
    registry: Arc<RwLock<WorkflowRegistry>>,
    http_client: reqwest::Client,
}

impl WorkflowExecutor {
    pub fn new(registry: Arc<RwLock<WorkflowRegistry>>) -> Self {
        Self {
            registry,
            http_client: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("failed to build HTTP client"),
        }
    }

    /// Execute a workflow triggered by a TriggerEvent
    pub async fn execute_workflow(
        &self,
        trigger_event: TriggerEvent,
    ) -> ExecResult<ExecutionResult> {
        let workflow_id = trigger_event.workflow_id;
        let start = Instant::now();

        info!(
            "Starting execution of workflow {workflow_id} (trigger: {})",
            trigger_event.trigger_config.kind_name()
        );

        // Load workflow from registry
        let workflow = {
            let reg = self.registry.read().await;
            reg.get(workflow_id)?
                .ok_or(ExecutionError::WorkflowNotFound(workflow_id))?
        };

        // Build and validate the DAG
        let graph = WorkflowGraph::build(&workflow)?;
        let levels = graph.execution_levels()?;

        let mut ctx = ExecutionContext::new(workflow_id, trigger_event);
        let mut nodes_executed = 0u32;
        let mut last_output = serde_json::Value::Null;

        for level in levels {
            // Execute all nodes at the same depth concurrently
            let mut tasks = Vec::new();

            for node_id in level {
                if let Some(node) = graph.node(node_id) {
                    let input = ctx.get_input_for_node(&graph, node_id);
                    tasks.push((node_id, node.clone(), input));
                }
            }

            // Run this level concurrently
            let mut handles = Vec::new();
            for (node_id, node, input) in tasks {
                let http_client = self.http_client.clone();
                let registry = self.registry.clone();
                handles.push(tokio::spawn(async move {
                    (
                        node_id,
                        execute_node_with_retry(&node, input, &http_client, &registry).await,
                    )
                }));
            }

            for handle in handles {
                let (node_id, result) = handle.await.map_err(|e| ExecutionError::NodeFailed {
                    node_id: Uuid::nil(),
                    reason: e.to_string(),
                })?;

                match result {
                    Ok(output) => {
                        ctx.node_outputs.insert(node_id, output.clone());
                        last_output = output;
                        nodes_executed += 1;
                    }
                    Err(e) => {
                        ctx.error_count += 1;
                        error!("Node {node_id} failed: {e}");
                        let duration_ms = start.elapsed().as_millis() as u64;
                        return Ok(ExecutionResult {
                            run_id: ctx.run_id,
                            workflow_id,
                            success: false,
                            output: serde_json::Value::Null,
                            error_message: Some(e.to_string()),
                            duration_ms,
                            nodes_executed,
                        });
                    }
                }
            }
        }

        let duration_ms = start.elapsed().as_millis() as u64;
        info!("Workflow {workflow_id} completed in {duration_ms}ms ({nodes_executed} nodes)");

        Ok(ExecutionResult {
            run_id: ctx.run_id,
            workflow_id,
            success: true,
            output: last_output,
            error_message: None,
            duration_ms,
            nodes_executed,
        })
    }
}

async fn execute_node_with_retry(
    node: &WorkflowNode,
    input: serde_json::Value,
    http_client: &reqwest::Client,
    registry: &Arc<RwLock<WorkflowRegistry>>,
) -> ExecResult<serde_json::Value> {
    let strategy = extract_error_strategy(node);

    match strategy {
        ErrorStrategy::StopOnError => execute_node(node, input, http_client, registry).await,
        ErrorStrategy::SkipAndContinue => {
            match execute_node(node, input, http_client, registry).await {
                Ok(v) => Ok(v),
                Err(e) => {
                    warn!("Node {} failed (skipping): {e}", node.id);
                    Ok(serde_json::json!({ "skipped": true, "error": e.to_string() }))
                }
            }
        }
        ErrorStrategy::RetryWithBackoff {
            max_retries,
            base_delay_ms,
        } => {
            let mut last_err = None;
            for attempt in 0..=max_retries {
                if attempt > 0 {
                    let delay = base_delay_ms * 2u64.pow(attempt - 1);
                    debug!(
                        "Retry {attempt}/{max_retries} for node {} after {delay}ms",
                        node.id
                    );
                    tokio::time::sleep(Duration::from_millis(delay)).await;
                }
                match execute_node(node, input.clone(), http_client, registry).await {
                    Ok(v) => return Ok(v),
                    Err(e) => last_err = Some(e),
                }
            }
            Err(last_err.unwrap_or(ExecutionError::MaxRetriesExceeded(node.id)))
        }
    }
}

fn extract_error_strategy(node: &WorkflowNode) -> ErrorStrategy {
    if let NodeType::ErrorHandler { strategy } = &node.node_type {
        return strategy.clone();
    }
    // Default from config
    node.config
        .get("error_strategy")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or(ErrorStrategy::StopOnError)
}

pub async fn execute_node(
    node: &WorkflowNode,
    input: serde_json::Value,
    http_client: &reqwest::Client,
    registry: &Arc<RwLock<WorkflowRegistry>>,
) -> ExecResult<serde_json::Value> {
    debug!("Executing node {} ({})", node.id, node.label);

    match &node.node_type {
        NodeType::Trigger(_) => {
            // Trigger nodes pass through the trigger payload
            Ok(input)
        }

        NodeType::SkillCall { skill_id, params } => {
            // In production: load WASM skill from marketplace, execute via wasm-runtime crate
            debug!("Skill call: {skill_id} with params {params}");
            Ok(serde_json::json!({
                "skill_id": skill_id.to_string(),
                "input": input,
                "output": null,
                "note": "skill execution delegated to wasm-runtime"
            }))
        }

        NodeType::Transform { expression } => {
            // Simple JSONPath-like transform expressions
            // e.g. "$.field" extracts a field, "$.field1 + $.field2" concatenates
            let result = apply_transform(expression, &input);
            Ok(result)
        }

        NodeType::Condition { predicate } => {
            let truthy = evaluate_predicate(predicate, &input);
            Ok(serde_json::json!({
                "condition_result": truthy,
                "branch": if truthy { "true" } else { "false" },
                "input": input,
            }))
        }

        NodeType::Loop { count_expr } => {
            let count = evaluate_count_expr(count_expr, &input).min(10_000);
            let mut results = Vec::with_capacity(count as usize);
            for i in 0..count {
                results.push(serde_json::json!({ "iteration": i, "input": input }));
            }
            Ok(serde_json::json!({ "iterations": count, "results": results }))
        }

        NodeType::SubWorkflow { workflow_id } => {
            // Load and execute sub-workflow
            let sub_workflow = {
                let reg = registry.read().await;
                reg.get(*workflow_id)?
                    .ok_or(ExecutionError::SubWorkflowError(format!(
                        "sub-workflow {workflow_id} not found"
                    )))?
            };

            debug!("Executing sub-workflow: {}", sub_workflow.name);
            // Full recursive execution would happen here; returning stub output
            Ok(serde_json::json!({
                "sub_workflow_id": workflow_id.to_string(),
                "sub_workflow_name": sub_workflow.name,
                "input": input,
            }))
        }

        NodeType::HttpRequest { url, method } => {
            let response = match method.to_string().as_str() {
                "GET" => http_client.get(url).send().await,
                "POST" => http_client.post(url).json(&input).send().await,
                "PUT" => http_client.put(url).json(&input).send().await,
                "PATCH" => http_client.patch(url).json(&input).send().await,
                "DELETE" => http_client.delete(url).send().await,
                m => {
                    return Err(ExecutionError::HttpError(format!(
                        "unsupported method: {m}"
                    )))
                }
            };

            match response {
                Ok(resp) => {
                    let status = resp.status().as_u16();
                    let body: serde_json::Value =
                        resp.json().await.unwrap_or(serde_json::Value::Null);
                    Ok(serde_json::json!({ "status": status, "body": body }))
                }
                Err(e) => Err(ExecutionError::HttpError(e.to_string())),
            }
        }

        NodeType::Delay { duration_ms } => {
            tokio::time::sleep(Duration::from_millis(*duration_ms)).await;
            Ok(serde_json::json!({ "delayed_ms": duration_ms, "input": input }))
        }

        NodeType::Merge => {
            // Merge combines inputs from all predecessor ports
            Ok(serde_json::json!({ "merged": input }))
        }

        NodeType::Split => {
            // Split fans out to multiple successor nodes with the same data
            Ok(input)
        }

        NodeType::ErrorHandler { strategy } => {
            debug!("Error handler node: {:?}", strategy);
            Ok(serde_json::json!({ "handled": true, "strategy": format!("{:?}", strategy) }))
        }
    }
}

fn apply_transform(expression: &str, input: &serde_json::Value) -> serde_json::Value {
    let expr = expression.trim();

    // Simple field extraction: "$.fieldname"
    if let Some(field) = expr.strip_prefix("$.") {
        if let Some(val) = input.get(field) {
            return val.clone();
        }
        return serde_json::Value::Null;
    }

    // Literal passthrough
    if expr == "$" {
        return input.clone();
    }

    // JSON literal
    if let Ok(v) = serde_json::from_str(expr) {
        return v;
    }

    serde_json::json!({ "transform": expr, "input": input })
}

fn evaluate_predicate(predicate: &str, input: &serde_json::Value) -> bool {
    let pred = predicate.trim();

    // "$.field == value" style
    if let Some(rest) = pred.strip_prefix("$.") {
        if let Some((field, cmp_rest)) = rest.split_once(" == ") {
            if let Some(val) = input.get(field.trim()) {
                let expected: serde_json::Value = serde_json::from_str(cmp_rest.trim())
                    .unwrap_or_else(|_| serde_json::Value::String(cmp_rest.trim().to_string()));
                return val == &expected;
            }
        }
        if let Some((field, cmp_rest)) = rest.split_once(" != ") {
            if let Some(val) = input.get(field.trim()) {
                let expected: serde_json::Value = serde_json::from_str(cmp_rest.trim())
                    .unwrap_or_else(|_| serde_json::Value::String(cmp_rest.trim().to_string()));
                return val != &expected;
            }
        }
        // Check field existence
        return input.get(rest).is_some();
    }

    // Boolean literal
    match pred {
        "true" => true,
        "false" => false,
        _ => !input.is_null(),
    }
}

fn evaluate_count_expr(expr: &str, input: &serde_json::Value) -> u64 {
    let expr = expr.trim();

    // Direct number literal
    if let Ok(n) = expr.parse::<u64>() {
        return n;
    }

    // Field reference: "$.count"
    if let Some(field) = expr.strip_prefix("$.") {
        if let Some(val) = input.get(field) {
            return val.as_u64().unwrap_or(1);
        }
    }

    1
}
