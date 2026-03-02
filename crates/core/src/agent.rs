use crate::{Error, Result};
use crate::event::{Event, EventBus};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;
use futures_util::StreamExt;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub api_url: String,
    pub api_key: Option<String>,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: usize,
    pub system_prompt: Option<String>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            api_url: "https://openrouter.ai/api/v1/chat/completions".to_string(),
            api_key: None,
            model: "deepseek/deepseek-r1-0528:free".to_string(),
            temperature: 0.7,
            max_tokens: 4096,
            system_prompt: None,
        }
    }
}

pub struct Agent {
    id: Uuid,
    config: Mutex<AgentConfig>,
    event_bus: EventBus,
    client: reqwest::Client,
}

impl Agent {
    pub fn new(config: AgentConfig, event_bus: EventBus) -> Self {
        Self {
            id: Uuid::new_v4(),
            config: Mutex::new(config),
            event_bus,
            client: reqwest::Client::new(),
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn set_system_prompt(&self, prompt: String) {
        self.config.lock().unwrap().system_prompt = Some(prompt);
    }

    pub fn set_model(&self, model: String) {
        self.config.lock().unwrap().model = model;
    }

    pub fn set_api_url(&self, url: String) {
        self.config.lock().unwrap().api_url = url;
    }

    pub fn set_api_key(&self, key: String) {
        self.config.lock().unwrap().api_key = Some(key);
    }

    pub fn model_name(&self) -> String {
        self.config.lock().unwrap().model.clone()
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

    fn build_messages(&self, prompt: &str) -> Vec<serde_json::Value> {
        let cfg = self.config.lock().unwrap();
        let mut messages = Vec::new();
        if let Some(ref sys) = cfg.system_prompt {
            messages.push(serde_json::json!({"role": "system", "content": sys}));
        }
        messages.push(serde_json::json!({"role": "user", "content": prompt}));
        messages
    }

    async fn call_llm(&self, prompt: &str) -> Result<String> {
        let (api_url, api_key, model, temp, max_tok) = {
            let cfg = self.config.lock().unwrap();
            (
                cfg.api_url.clone(),
                cfg.api_key.clone().unwrap_or_default(),
                cfg.model.clone(),
                cfg.temperature,
                cfg.max_tokens,
            )
        };

        let messages = self.build_messages(prompt);

        let request_body = serde_json::json!({
            "model": model,
            "messages": messages,
            "temperature": temp,
            "max_tokens": max_tok,
        });

        let response = self.client
            .post(&api_url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .header("HTTP-Referer", "https://openzax.dev")
            .header("X-Title", "OpenZax")
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

        let (api_url, api_key, model, temp, max_tok) = {
            let cfg = self.config.lock().unwrap();
            (
                cfg.api_url.clone(),
                cfg.api_key.clone(),
                cfg.model.clone(),
                cfg.temperature,
                cfg.max_tokens,
            )
        };

        let api_key = api_key.filter(|k| !k.is_empty())
            .ok_or_else(|| Error::Agent(
                "No API key set. Get a FREE key at https://openrouter.ai/keys then set OPENROUTER_API_KEY".to_string()
            ))?;

        let messages = self.build_messages(prompt);

        let request_body = serde_json::json!({
            "model": model,
            "messages": messages,
            "temperature": temp,
            "max_tokens": max_tok,
            "stream": true,
        });

        let response = self.client
            .post(&api_url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .header("HTTP-Referer", "https://openzax.dev")
            .header("X-Title", "OpenZax")
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

        let mut full_content = String::new();

        while let Some(line) = lines.next_line().await? {
            let line: String = line;
            if line.starts_with("data: ") {
                let data = &line[6..];
                if data == "[DONE]" {
                    break;
                }

                if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                    if let Some(content) = json["choices"][0]["delta"]["content"].as_str() {
                        full_content.push_str(content);
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

        self.event_bus.publish(Event::AgentOutput {
            agent_id: self.id,
            content: full_content,
            timestamp: Utc::now(),
        })?;

        Ok(())
    }
}
