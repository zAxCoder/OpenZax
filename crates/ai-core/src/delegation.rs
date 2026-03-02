use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum DelegationError {
    #[error("Budget exhausted: {0}")]
    BudgetExhausted(String),
    #[error("Agent not found: {0}")]
    AgentNotFound(Uuid),
    #[error("Agent already completed")]
    AlreadyCompleted,
    #[error("Task timeout")]
    Timeout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentBudget {
    pub max_tokens: u64,
    pub max_wall_time_secs: u64,
    pub max_tool_calls: u32,
    pub max_memory_mb: u32,
    pub max_fs_bytes: u64,
    pub current_tokens: u64,
    pub current_tool_calls: u32,
    #[serde(skip, default = "Instant::now")]
    pub start_time: Instant,
}

impl AgentBudget {
    pub fn new(
        max_tokens: u64,
        max_wall_time_secs: u64,
        max_tool_calls: u32,
        max_memory_mb: u32,
        max_fs_bytes: u64,
    ) -> Self {
        Self {
            max_tokens,
            max_wall_time_secs,
            max_tool_calls,
            max_memory_mb,
            max_fs_bytes,
            current_tokens: 0,
            current_tool_calls: 0,
            start_time: Instant::now(),
        }
    }

    pub fn fraction_of(other: &AgentBudget, fraction: f32) -> Self {
        let f = fraction.clamp(0.0, 1.0);
        let remaining_tokens = other.max_tokens.saturating_sub(other.current_tokens);
        let elapsed = other.start_time.elapsed().as_secs();
        let remaining_wall = other.max_wall_time_secs.saturating_sub(elapsed);
        let remaining_tools = other
            .max_tool_calls
            .saturating_sub(other.current_tool_calls);
        Self {
            max_tokens: (remaining_tokens as f32 * f) as u64,
            max_wall_time_secs: (remaining_wall as f32 * f) as u64,
            max_tool_calls: (remaining_tools as f32 * f) as u32,
            max_memory_mb: (other.max_memory_mb as f32 * f) as u32,
            max_fs_bytes: (other.max_fs_bytes as f32 * f) as u64,
            current_tokens: 0,
            current_tool_calls: 0,
            start_time: Instant::now(),
        }
    }

    pub fn time_elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    pub fn time_remaining(&self) -> Duration {
        let elapsed = self.start_time.elapsed().as_secs();
        let remaining = self.max_wall_time_secs.saturating_sub(elapsed);
        Duration::from_secs(remaining)
    }
}

pub struct BudgetEnforcer;

impl BudgetEnforcer {
    pub fn check_token_budget(
        budget: &AgentBudget,
        tokens_to_consume: u64,
    ) -> Result<(), DelegationError> {
        if budget.current_tokens + tokens_to_consume > budget.max_tokens {
            return Err(DelegationError::BudgetExhausted(format!(
                "Token budget exceeded: {}/{} (need {} more)",
                budget.current_tokens, budget.max_tokens, tokens_to_consume
            )));
        }
        Ok(())
    }

    pub fn check_tool_call(budget: &AgentBudget) -> Result<(), DelegationError> {
        if budget.current_tool_calls >= budget.max_tool_calls {
            return Err(DelegationError::BudgetExhausted(format!(
                "Tool call budget exceeded: {}/{}",
                budget.current_tool_calls, budget.max_tool_calls
            )));
        }
        Ok(())
    }

    pub fn check_time_remaining(budget: &AgentBudget) -> Result<Duration, DelegationError> {
        let remaining = budget.time_remaining();
        if remaining.is_zero() {
            return Err(DelegationError::Timeout);
        }
        Ok(remaining)
    }

    pub fn is_exhausted(budget: &AgentBudget) -> bool {
        budget.current_tokens >= budget.max_tokens
            || budget.current_tool_calls >= budget.max_tool_calls
            || budget.time_remaining().is_zero()
    }

    pub fn consume_tokens(budget: &mut AgentBudget, tokens: u64) -> Result<(), DelegationError> {
        Self::check_token_budget(budget, tokens)?;
        budget.current_tokens += tokens;
        Ok(())
    }

    pub fn consume_tool_call(budget: &mut AgentBudget) -> Result<(), DelegationError> {
        Self::check_tool_call(budget)?;
        budget.current_tool_calls += 1;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "state")]
pub enum AgentStatus {
    Spawning,
    Running,
    WaitingForApproval,
    Completed { result: serde_json::Value },
    Failed { error: String },
    Killed,
}

impl AgentStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            AgentStatus::Completed { .. } | AgentStatus::Failed { .. } | AgentStatus::Killed
        )
    }
}

#[derive(Debug, Clone)]
pub struct AgentHandle {
    pub agent_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub task: String,
    pub budget: AgentBudget,
    pub status: AgentStatus,
    pub capabilities: Vec<String>,
}

impl AgentHandle {
    pub fn new(
        task: impl Into<String>,
        parent_id: Option<Uuid>,
        budget: AgentBudget,
        capabilities: Vec<String>,
    ) -> Self {
        Self {
            agent_id: Uuid::new_v4(),
            parent_id,
            task: task.into(),
            budget,
            status: AgentStatus::Spawning,
            capabilities,
        }
    }
}

#[derive(Debug)]
pub struct AgentTree {
    pub agent_id: Uuid,
    pub task: String,
    pub status: String,
    pub children: Vec<AgentTree>,
}

pub struct AgentSpawner {
    agents: Arc<Mutex<HashMap<Uuid, AgentHandle>>>,
}

impl AgentSpawner {
    pub fn new() -> Self {
        Self {
            agents: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn spawn(
        &self,
        task: impl Into<String>,
        parent_budget: &AgentBudget,
        capability_subset: Vec<String>,
        parent_id: Option<Uuid>,
        budget_fraction: f32,
    ) -> Result<AgentHandle, DelegationError> {
        let child_budget = self.inherit_budget(parent_budget, budget_fraction);
        let mut handle = AgentHandle::new(task, parent_id, child_budget, capability_subset);
        handle.status = AgentStatus::Running;

        let id = handle.agent_id;
        self.agents.lock().unwrap().insert(id, handle.clone());

        tracing::info!(
            "Spawned agent {} (parent={:?}) with budget fraction {:.2}",
            id,
            parent_id,
            budget_fraction
        );
        Ok(handle)
    }

    pub fn complete_agent(
        &self,
        agent_id: Uuid,
        result: serde_json::Value,
    ) -> Result<(), DelegationError> {
        let mut agents = self.agents.lock().unwrap();
        let agent = agents
            .get_mut(&agent_id)
            .ok_or(DelegationError::AgentNotFound(agent_id))?;
        if agent.status.is_terminal() {
            return Err(DelegationError::AlreadyCompleted);
        }
        agent.status = AgentStatus::Completed { result };
        Ok(())
    }

    pub fn fail_agent(
        &self,
        agent_id: Uuid,
        error: impl Into<String>,
    ) -> Result<(), DelegationError> {
        let mut agents = self.agents.lock().unwrap();
        let agent = agents
            .get_mut(&agent_id)
            .ok_or(DelegationError::AgentNotFound(agent_id))?;
        agent.status = AgentStatus::Failed {
            error: error.into(),
        };
        Ok(())
    }

    pub async fn join(&self, agent_id: Uuid) -> Result<serde_json::Value, DelegationError> {
        // Poll with exponential backoff until terminal state
        let mut wait_ms = 50u64;
        loop {
            let status = {
                let agents = self.agents.lock().unwrap();
                agents.get(&agent_id).map(|a| a.status.clone())
            };
            match status {
                Some(AgentStatus::Completed { result }) => return Ok(result),
                Some(AgentStatus::Failed { error }) => {
                    return Err(DelegationError::BudgetExhausted(error))
                }
                Some(AgentStatus::Killed) => {
                    return Err(DelegationError::BudgetExhausted("Agent was killed".into()))
                }
                None => return Err(DelegationError::AgentNotFound(agent_id)),
                _ => {
                    tokio::time::sleep(Duration::from_millis(wait_ms)).await;
                    wait_ms = (wait_ms * 2).min(2000);
                }
            }
        }
    }

    pub fn kill(&self, agent_id: Uuid, reason: &str) -> Result<(), DelegationError> {
        let mut agents = self.agents.lock().unwrap();
        let agent = agents
            .get_mut(&agent_id)
            .ok_or(DelegationError::AgentNotFound(agent_id))?;
        tracing::warn!("Killing agent {}: {}", agent_id, reason);
        agent.status = AgentStatus::Killed;
        Ok(())
    }

    pub fn get_agent_tree(&self) -> AgentTree {
        let agents = self.agents.lock().unwrap();
        // Find root agents (no parent)
        let roots: Vec<Uuid> = agents
            .values()
            .filter(|a| a.parent_id.is_none())
            .map(|a| a.agent_id)
            .collect();

        fn build_tree(agents: &HashMap<Uuid, AgentHandle>, agent_id: Uuid) -> AgentTree {
            let agent = match agents.get(&agent_id) {
                Some(a) => a,
                None => {
                    return AgentTree {
                        agent_id,
                        task: "unknown".to_string(),
                        status: "unknown".to_string(),
                        children: vec![],
                    }
                }
            };
            let children_ids: Vec<Uuid> = agents
                .values()
                .filter(|a| a.parent_id == Some(agent_id))
                .map(|a| a.agent_id)
                .collect();
            let children = children_ids
                .into_iter()
                .map(|id| build_tree(agents, id))
                .collect();
            let status = match &agent.status {
                AgentStatus::Spawning => "spawning",
                AgentStatus::Running => "running",
                AgentStatus::WaitingForApproval => "waiting_approval",
                AgentStatus::Completed { .. } => "completed",
                AgentStatus::Failed { .. } => "failed",
                AgentStatus::Killed => "killed",
            };
            AgentTree {
                agent_id,
                task: agent.task.clone(),
                status: status.to_string(),
                children,
            }
        }

        let root_id = roots.into_iter().next().unwrap_or_else(Uuid::new_v4);
        build_tree(&agents, root_id)
    }

    pub fn inherit_budget(&self, parent: &AgentBudget, fraction: f32) -> AgentBudget {
        AgentBudget::fraction_of(parent, fraction)
    }

    pub fn update_budget_tokens(
        &self,
        agent_id: Uuid,
        tokens_used: u64,
    ) -> Result<(), DelegationError> {
        let mut agents = self.agents.lock().unwrap();
        let agent = agents
            .get_mut(&agent_id)
            .ok_or(DelegationError::AgentNotFound(agent_id))?;
        BudgetEnforcer::consume_tokens(&mut agent.budget, tokens_used)
    }

    pub fn update_budget_tool_call(&self, agent_id: Uuid) -> Result<(), DelegationError> {
        let mut agents = self.agents.lock().unwrap();
        let agent = agents
            .get_mut(&agent_id)
            .ok_or(DelegationError::AgentNotFound(agent_id))?;
        BudgetEnforcer::consume_tool_call(&mut agent.budget)
    }
}

impl Default for AgentSpawner {
    fn default() -> Self {
        Self::new()
    }
}
