use std::{collections::HashMap, sync::Arc};
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{debug, info};
use uuid::Uuid;

use crate::{
    executor::WorkflowExecutor,
    triggers::{TriggerConfig, TriggerEvent},
};

#[derive(Debug, Error)]
pub enum SubWorkflowError {
    #[error("Module not found: {0}")]
    ModuleNotFound(Uuid),

    #[error("Module not found by name: {0}")]
    ModuleNotFoundByName(String),

    #[error("Input validation failed: {field} - {reason}")]
    InputValidationFailed { field: String, reason: String },

    #[error("Output validation failed: {field} - {reason}")]
    OutputValidationFailed { field: String, reason: String },

    #[error("Circular sub-workflow reference: {0}")]
    CircularReference(Uuid),

    #[error("Execution error: {0}")]
    ExecutionError(#[from] crate::executor::ExecutionError),

    #[error("Registry error: {0}")]
    Registry(#[from] crate::registry::RegistryError),
}

pub type SubWorkflowResult<T> = std::result::Result<T, SubWorkflowError>;

/// Describes a sub-workflow exposed as a reusable module
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SubWorkflowModule {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub name: String,
    pub description: String,
    /// JSON Schema describing the expected input object
    pub input_schema: serde_json::Value,
    /// JSON Schema describing the produced output object
    pub output_schema: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl SubWorkflowModule {
    pub fn new(
        workflow_id: Uuid,
        name: impl Into<String>,
        description: impl Into<String>,
        input_schema: serde_json::Value,
        output_schema: serde_json::Value,
    ) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Uuid::new_v4(),
            workflow_id,
            name: name.into(),
            description: description.into(),
            input_schema,
            output_schema,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Manages the registry of sub-workflow modules
pub struct ModuleRegistry {
    modules: HashMap<Uuid, SubWorkflowModule>,
    name_index: HashMap<String, Uuid>,
}

impl ModuleRegistry {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            name_index: HashMap::new(),
        }
    }

    /// Register a sub-workflow module
    pub fn register(&mut self, module: SubWorkflowModule) -> SubWorkflowResult<Uuid> {
        let id = module.id;
        let name = module.name.clone();

        self.name_index.insert(name.clone(), id);
        self.modules.insert(id, module);

        info!("Registered sub-workflow module '{name}' ({id})");
        Ok(id)
    }

    /// Get a module by ID
    pub fn get(&self, id: Uuid) -> SubWorkflowResult<&SubWorkflowModule> {
        self.modules.get(&id).ok_or(SubWorkflowError::ModuleNotFound(id))
    }

    /// Get a module by name
    pub fn get_by_name(&self, name: &str) -> SubWorkflowResult<&SubWorkflowModule> {
        let id = self.name_index.get(name)
            .copied()
            .ok_or_else(|| SubWorkflowError::ModuleNotFoundByName(name.to_string()))?;
        self.get(id)
    }

    /// List all registered modules
    pub fn list(&self) -> Vec<&SubWorkflowModule> {
        self.modules.values().collect()
    }

    /// Remove a module
    pub fn unregister(&mut self, id: Uuid) -> bool {
        if let Some(module) = self.modules.remove(&id) {
            self.name_index.remove(&module.name);
            true
        } else {
            false
        }
    }

    pub fn module_count(&self) -> usize {
        self.modules.len()
    }
}

impl Default for ModuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Executes a sub-workflow with typed inputs and returns typed output
pub struct SubWorkflowInvoker {
    executor: Arc<WorkflowExecutor>,
    module_registry: Arc<RwLock<ModuleRegistry>>,
    /// Tracks in-progress workflow IDs to detect circular references
    execution_stack: Vec<Uuid>,
}

impl SubWorkflowInvoker {
    pub fn new(
        executor: Arc<WorkflowExecutor>,
        module_registry: Arc<RwLock<ModuleRegistry>>,
    ) -> Self {
        Self {
            executor,
            module_registry,
            execution_stack: Vec::new(),
        }
    }

    /// Invoke a sub-workflow by module ID with typed input
    pub async fn invoke_by_id(
        &mut self,
        module_id: Uuid,
        input: serde_json::Value,
        parent_workflow_id: Uuid,
    ) -> SubWorkflowResult<TypedOutput> {
        let module = {
            let reg = self.module_registry.read().await;
            reg.get(module_id)?.clone()
        };

        self.invoke_module(&module, input, parent_workflow_id).await
    }

    /// Invoke a sub-workflow by module name with typed input
    pub async fn invoke_by_name(
        &mut self,
        name: &str,
        input: serde_json::Value,
        parent_workflow_id: Uuid,
    ) -> SubWorkflowResult<TypedOutput> {
        let module = {
            let reg = self.module_registry.read().await;
            reg.get_by_name(name)?.clone()
        };

        self.invoke_module(&module, input, parent_workflow_id).await
    }

    async fn invoke_module(
        &mut self,
        module: &SubWorkflowModule,
        input: serde_json::Value,
        parent_workflow_id: Uuid,
    ) -> SubWorkflowResult<TypedOutput> {
        // Detect circular reference
        if self.execution_stack.contains(&module.workflow_id) {
            return Err(SubWorkflowError::CircularReference(module.workflow_id));
        }

        // Validate input against schema
        validate_against_schema(&input, &module.input_schema, "input")?;

        debug!(
            "Invoking sub-workflow '{}' ({}) from parent {}",
            module.name, module.workflow_id, parent_workflow_id
        );

        self.execution_stack.push(module.workflow_id);

        // Build a synthetic trigger event for the sub-workflow
        let trigger_event = TriggerEvent::new(
            module.workflow_id,
            TriggerConfig::ChainedFrom {
                workflow_id: parent_workflow_id,
                condition: None,
            },
            input.clone(),
        );

        let result = self.executor.execute_workflow(trigger_event).await;
        self.execution_stack.pop();

        let exec_result = result?;

        // Validate output against schema
        validate_against_schema(&exec_result.output, &module.output_schema, "output")?;

        Ok(TypedOutput {
            module_id: module.id,
            module_name: module.name.clone(),
            run_id: exec_result.run_id,
            value: exec_result.output,
            duration_ms: exec_result.duration_ms,
            nodes_executed: exec_result.nodes_executed,
        })
    }
}

/// The typed output returned from a sub-workflow invocation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TypedOutput {
    pub module_id: Uuid,
    pub module_name: String,
    pub run_id: Uuid,
    pub value: serde_json::Value,
    pub duration_ms: u64,
    pub nodes_executed: u32,
}

/// Validates a JSON value against a JSON Schema (subset support)
///
/// Supports: type, required, properties, minimum/maximum, minLength/maxLength
fn validate_against_schema(
    value: &serde_json::Value,
    schema: &serde_json::Value,
    context: &str,
) -> SubWorkflowResult<()> {
    if schema.is_null() || schema == &serde_json::json!({}) {
        return Ok(());
    }

    let schema_obj = match schema.as_object() {
        Some(obj) => obj,
        None => return Ok(()), // Non-object schema - skip
    };

    // Type check
    if let Some(type_val) = schema_obj.get("type").and_then(|v| v.as_str()) {
        let type_ok = match type_val {
            "object" => value.is_object(),
            "array" => value.is_array(),
            "string" => value.is_string(),
            "number" | "integer" => value.is_number(),
            "boolean" => value.is_boolean(),
            "null" => value.is_null(),
            _ => true,
        };
        if !type_ok {
            return Err(SubWorkflowError::InputValidationFailed {
                field: context.to_string(),
                reason: format!("expected type '{type_val}', got {:?}", value_type_name(value)),
            });
        }
    }

    // Required fields check
    if let Some(required) = schema_obj.get("required").and_then(|v| v.as_array()) {
        if let Some(obj) = value.as_object() {
            for req in required {
                if let Some(field) = req.as_str() {
                    if !obj.contains_key(field) {
                        return Err(SubWorkflowError::InputValidationFailed {
                            field: format!("{context}.{field}"),
                            reason: "required field missing".to_string(),
                        });
                    }
                }
            }
        }
    }

    // Properties validation (recurse)
    if let (Some(properties), Some(obj)) = (
        schema_obj.get("properties").and_then(|v| v.as_object()),
        value.as_object(),
    ) {
        for (prop_name, prop_schema) in properties {
            if let Some(prop_value) = obj.get(prop_name) {
                let field_ctx = format!("{context}.{prop_name}");
                validate_against_schema(prop_value, prop_schema, &field_ctx)?;
            }
        }
    }

    // String length constraints
    if let Some(s) = value.as_str() {
        if let Some(min) = schema_obj.get("minLength").and_then(|v| v.as_u64()) {
            if s.len() < min as usize {
                return Err(SubWorkflowError::InputValidationFailed {
                    field: context.to_string(),
                    reason: format!("string too short: {} < {min}", s.len()),
                });
            }
        }
        if let Some(max) = schema_obj.get("maxLength").and_then(|v| v.as_u64()) {
            if s.len() > max as usize {
                return Err(SubWorkflowError::InputValidationFailed {
                    field: context.to_string(),
                    reason: format!("string too long: {} > {max}", s.len()),
                });
            }
        }
    }

    // Numeric range constraints
    if let Some(n) = value.as_f64() {
        if let Some(min) = schema_obj.get("minimum").and_then(|v| v.as_f64()) {
            if n < min {
                return Err(SubWorkflowError::InputValidationFailed {
                    field: context.to_string(),
                    reason: format!("{n} is less than minimum {min}"),
                });
            }
        }
        if let Some(max) = schema_obj.get("maximum").and_then(|v| v.as_f64()) {
            if n > max {
                return Err(SubWorkflowError::InputValidationFailed {
                    field: context.to_string(),
                    reason: format!("{n} exceeds maximum {max}"),
                });
            }
        }
    }

    Ok(())
}

fn value_type_name(v: &serde_json::Value) -> &'static str {
    match v {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "boolean",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}
