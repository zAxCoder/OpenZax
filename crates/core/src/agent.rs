use crate::{Error, Result};
use crate::event::{Event, EventBus};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;
use futures_util::StreamExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub api_url: String,
    pub api_key: Option<String>,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: usize,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            api_url: "https://api.openai.com/v1/chat/completions".to_string(),
            api_key: None,
            model: "gpt-4".to_string(),
            temperature: 0.7,
            max_tokens: 2000,
        }
    }
}

pub struct Agent {
    id: Uuid,
    config: AgentConfig,
    event_bus: EventBus,
    client: reqwest::Client,
}

impl Agent {
    pub fn new(config: AgentConfig, event_bus: EventBus) -> Self {
        Self {
            id: Uuid::new_v4(),
            config,
            event_bus,
            client: reqwest::Client::new(),
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub async fn process(&self, prompt: &str) -> Result<String> {
        self.event_bus.publish(Event::AgentThinking {
            agent_id: self.id,
            thought_text: "Processing request...".to_string(),
            timestamp: Utc::now(),
        })?;

        let response = self.call_llm(prompt).await?;

        self.event_bus.publish(Event::AgentOutput {
            agent_id: self.id,
            content: response.clone(),
            timestamp: Utc::now(),
        })?;

        Ok(response)
    }

    async fn call_llm(&self, prompt: &str) -> Result<String> {
        let api_key = self.config.api_key.as_ref()
            .ok_or_else(|| Error::Agent("API key not configured".to_string()))?;

        let request_body = serde_json::json!({
            "model": self.config.model,
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "temperature": self.config.temperature,
            "max_tokens": self.config.max_tokens,
        });

        let response = self.client
            .post(&self.config.api_url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(Error::Agent(format!("LLM API error: {}", error_text)));
        }

        let response_json: serde_json::Value = response.json().await?;
        
        let content = response_json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| Error::Agent("Invalid response format".to_string()))?
            .to_string();

        Ok(content)
    }

    pub async fn process_streaming(&self, prompt: &str) -> Result<()> {
        self.event_bus.publish(Event::AgentThinking {
            agent_id: self.id,
            thought_text: "Processing request...".to_string(),
            timestamp: Utc::now(),
        })?;

        let api_key = self.config.api_key.as_ref()
            .ok_or_else(|| Error::Agent("API key not configured".to_string()))?;

        let request_body = serde_json::json!({
            "model": self.config.model,
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "temperature": self.config.temperature,
            "max_tokens": self.config.max_tokens,
            "stream": true,
        });

        let response = self.client
            .post(&self.config.api_url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(Error::Agent(format!("LLM API error: {}", error_text)));
        }

        use tokio::io::AsyncBufReadExt;
        let mut lines = tokio::io::BufReader::new(
            tokio_util::io::StreamReader::new(
                response.bytes_stream().map(|result: std::result::Result<bytes::Bytes, reqwest::Error>| {
                    result.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
                })
            )
        ).lines();

        while let Some(line) = lines.next_line().await? {
            let line: String = line;
            if line.starts_with("data: ") {
                let data = &line[6..];
                if data == "[DONE]" {
                    break;
                }

                if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                    if let Some(content) = json["choices"][0]["delta"]["content"].as_str() {
                        self.event_bus.publish(Event::AgentTokenStream {
                            agent_id: self.id,
                            token: content.to_string(),
                            finish_reason: json["choices"][0]["finish_reason"]
                                .as_str()
                                .map(|s| s.to_string()),
                            timestamp: Utc::now(),
                        })?;
                    }
                }
            }
        }

        Ok(())
    }
}
