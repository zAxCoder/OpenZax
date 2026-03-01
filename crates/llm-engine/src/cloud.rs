use crate::{LlmError, LlmResult};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

#[derive(Debug, Clone)]
pub struct CloudProvider {
    client: Client,
    api_url: String,
    api_key: String,
    model: String,
}

impl CloudProvider {
    pub fn new(api_url: String, api_key: String, model: String) -> Self {
        Self {
            client: Client::new(),
            api_url,
            api_key,
            model,
        }
    }

    pub async fn generate(&self, prompt: &str, max_tokens: usize) -> LlmResult<String> {
        info!("Calling cloud API: {}", self.api_url);
        
        let request = CloudRequest {
            model: self.model.clone(),
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            max_tokens,
            temperature: 0.7,
            stream: false,
        };

        let response = self.client
            .post(&self.api_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(LlmError::Inference(format!("API error: {}", error_text)));
        }

        let cloud_response: CloudResponse = response.json().await?;
        
        let content = cloud_response.choices
            .first()
            .and_then(|c| c.message.content.clone())
            .ok_or_else(|| LlmError::Inference("No content in response".to_string()))?;

        debug!("Received response: {} tokens", content.len());
        Ok(content)
    }

    pub async fn generate_stream(
        &self,
        prompt: &str,
        max_tokens: usize,
    ) -> LlmResult<impl futures::Stream<Item = LlmResult<String>>> {
        info!("Calling cloud API (streaming): {}", self.api_url);
        
        let request = CloudRequest {
            model: self.model.clone(),
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            max_tokens,
            temperature: 0.7,
            stream: true,
        };

        let response = self.client
            .post(&self.api_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(LlmError::Inference(format!("API error: {}", error_text)));
        }

        // TODO: Implement actual streaming
        // For now, return a placeholder stream
        use futures::stream;
        Ok(stream::once(async { Ok("Streaming not yet implemented".to_string()) }))
    }
}

#[derive(Debug, Serialize)]
struct CloudRequest {
    model: String,
    messages: Vec<Message>,
    max_tokens: usize,
    temperature: f64,
    stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct CloudResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: MessageResponse,
}

#[derive(Debug, Deserialize)]
struct MessageResponse {
    content: Option<String>,
}
