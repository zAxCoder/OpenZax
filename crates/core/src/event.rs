use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    UserInput {
        session_id: Uuid,
        content: String,
        attachments: Vec<String>,
        timestamp: DateTime<Utc>,
    },
    AgentThinking {
        agent_id: Uuid,
        thought_text: String,
        timestamp: DateTime<Utc>,
    },
    AgentTokenStream {
        agent_id: Uuid,
        token: String,
        finish_reason: Option<String>,
        timestamp: DateTime<Utc>,
    },
    AgentOutput {
        agent_id: Uuid,
        content: String,
        timestamp: DateTime<Utc>,
    },
    SystemEvent {
        message: String,
        level: EventLevel,
        timestamp: DateTime<Utc>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EventLevel {
    Info,
    Warning,
    Error,
}

pub struct EventBus {
    sender: tokio::sync::broadcast::Sender<Event>,
}

impl EventBus {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = tokio::sync::broadcast::channel(capacity);
        Self { sender }
    }

    pub fn publish(&self, event: Event) -> crate::Result<()> {
        self.sender
            .send(event)
            .map_err(|e| crate::Error::EventBus(format!("Failed to publish event: {}", e)))?;
        Ok(())
    }

    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<Event> {
        self.sender.subscribe()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new(1000)
    }
}
