use crate::{Error, Result};
use crate::event::{Event, EventBus};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;
use futures_util::StreamExt;
use std::sync::Mutex;
use std::collections::HashMap;

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

    // ── File system tool definitions ──────────────────────────────────────────

    fn builtin_tools() -> serde_json::Value {
        serde_json::json!([
            {
                "type": "function",
                "function": {
                    "name": "read_file",
                    "description": "Read the contents of a file from the filesystem",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "path": { "type": "string", "description": "File path to read" }
                        },
                        "required": ["path"]
                    }
                }
            },
            {
                "type": "function",
                "function": {
                    "name": "write_file",
                    "description": "Write or create a file with the given content",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "path": { "type": "string", "description": "File path to write" },
                            "content": { "type": "string", "description": "Content to write to the file" }
                        },
                        "required": ["path", "content"]
                    }
                }
            },
            {
                "type": "function",
                "function": {
                    "name": "list_directory",
                    "description": "List files and directories at the given path",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "path": { "type": "string", "description": "Directory path to list (default: current directory)" }
                        },
                        "required": []
                    }
                }
            },
            {
                "type": "function",
                "function": {
                    "name": "execute_command",
                    "description": "Execute a shell command and return its output. Use this to run programs, install packages, run tests, etc.",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "command": { "type": "string", "description": "Shell command to execute" }
                        },
                        "required": ["command"]
                    }
                }
            },
            {
                "type": "function",
                "function": {
                    "name": "delete_file",
                    "description": "Delete a file or directory",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "path": { "type": "string", "description": "Path to delete" }
                        },
                        "required": ["path"]
                    }
                }
            },
            {
                "type": "function",
                "function": {
                    "name": "create_directory",
                    "description": "Create a directory (including parent directories)",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "path": { "type": "string", "description": "Directory path to create" }
                        },
                        "required": ["path"]
                    }
                }
            },
            {
                "type": "function",
                "function": {
                    "name": "move_file",
                    "description": "Move or rename a file or directory",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "source": { "type": "string", "description": "Source path" },
                            "destination": { "type": "string", "description": "Destination path" }
                        },
                        "required": ["source", "destination"]
                    }
                }
            }
        ])
    }

    async fn execute_tool(name: &str, args: &serde_json::Value) -> String {
        match name {
            "read_file" => {
                let path = args["path"].as_str().unwrap_or(".");
                match std::fs::read_to_string(path) {
                    Ok(content) => {
                        if content.len() > 20000 {
                            format!("[File truncated to 20000 chars]\n{}", &content[..20000])
                        } else {
                            content
                        }
                    }
                    Err(e) => format!("Error reading file '{}': {}", path, e),
                }
            }
            "write_file" => {
                let path = args["path"].as_str().unwrap_or("");
                let content = args["content"].as_str().unwrap_or("");
                if let Some(parent) = std::path::Path::new(path).parent() {
                    if !parent.as_os_str().is_empty() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                }
                match std::fs::write(path, content) {
                    Ok(_) => format!("Successfully wrote {} bytes to '{}'", content.len(), path),
                    Err(e) => format!("Error writing file '{}': {}", path, e),
                }
            }
            "list_directory" => {
                let path = args["path"].as_str().unwrap_or(".");
                match std::fs::read_dir(path) {
                    Ok(entries) => {
                        let mut items: Vec<String> = entries
                            .flatten()
                            .map(|e| {
                                let name = e.file_name().to_string_lossy().into_owned();
                                let is_dir = e.file_type().map(|t| t.is_dir()).unwrap_or(false);
                                if is_dir { format!("{}/", name) } else { name }
                            })
                            .collect();
                        items.sort();
                        if items.is_empty() {
                            format!("Directory '{}' is empty", path)
                        } else {
                            items.join("\n")
                        }
                    }
                    Err(e) => format!("Error listing directory '{}': {}", path, e),
                }
            }
            "execute_command" => {
                let cmd = args["command"].as_str().unwrap_or("");
                #[cfg(windows)]
                let output = std::process::Command::new("powershell")
                    .args(["-NoProfile", "-Command", cmd])
                    .output();
                #[cfg(not(windows))]
                let output = std::process::Command::new("sh")
                    .args(["-c", cmd])
                    .output();

                match output {
                    Ok(o) => {
                        let stdout = String::from_utf8_lossy(&o.stdout).to_string();
                        let stderr = String::from_utf8_lossy(&o.stderr).to_string();
                        let exit_code = o.status.code().unwrap_or(-1);
                        let mut result = String::new();
                        if !stdout.is_empty() { result.push_str(&stdout); }
                        if !stderr.is_empty() {
                            if !result.is_empty() { result.push('\n'); }
                            result.push_str(&format!("STDERR: {}", stderr));
                        }
                        if result.is_empty() {
                            format!("Command completed with exit code {}", exit_code)
                        } else {
                            result
                        }
                    }
                    Err(e) => format!("Error executing command: {}", e),
                }
            }
            "delete_file" => {
                let path = args["path"].as_str().unwrap_or("");
                let p = std::path::Path::new(path);
                let result = if p.is_dir() {
                    std::fs::remove_dir_all(path)
                } else {
                    std::fs::remove_file(path)
                };
                match result {
                    Ok(_) => format!("Successfully deleted '{}'", path),
                    Err(e) => format!("Error deleting '{}': {}", path, e),
                }
            }
            "create_directory" => {
                let path = args["path"].as_str().unwrap_or("");
                match std::fs::create_dir_all(path) {
                    Ok(_) => format!("Successfully created directory '{}'", path),
                    Err(e) => format!("Error creating directory '{}': {}", path, e),
                }
            }
            "move_file" => {
                let src = args["source"].as_str().unwrap_or("");
                let dst = args["destination"].as_str().unwrap_or("");
                match std::fs::rename(src, dst) {
                    Ok(_) => format!("Moved '{}' → '{}'", src, dst),
                    Err(e) => format!("Error moving '{}' to '{}': {}", src, dst, e),
                }
            }
            _ => format!("Unknown tool: {}", name),
        }
    }

    // ── Streaming with tool call support ─────────────────────────────────────

    async fn stream_with_tools(
        &self,
        messages: &[serde_json::Value],
        api_url: &str,
        api_key: &str,
        model: &str,
        temp: f32,
        max_tok: usize,
    ) -> Result<(String, Vec<serde_json::Value>)> {
        let request_body = serde_json::json!({
            "model": model,
            "messages": messages,
            "temperature": temp,
            "max_tokens": max_tok,
            "stream": true,
            "tools": Self::builtin_tools(),
            "tool_choice": "auto",
        });

        let response = self.client
            .post(api_url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .header("HTTP-Referer", "https://openzax.dev")
            .header("X-Title", "OpenZax")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let err = response.text().await?;
            return Err(Error::Agent(format!("LLM API error: {}", err)));
        }

        use tokio::io::AsyncBufReadExt;
        let mut lines = tokio::io::BufReader::new(
            tokio_util::io::StreamReader::new(
                response.bytes_stream().map(|r: std::result::Result<bytes::Bytes, reqwest::Error>| {
                    r.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
                })
            )
        ).lines();

        let mut full_content = String::new();
        // index → (id, name, args_buf)
        let mut tc_map: HashMap<usize, (String, String, String)> = HashMap::new();

        while let Some(line) = lines.next_line().await? {
            if !line.starts_with("data: ") { continue; }
            let data = &line[6..];
            if data == "[DONE]" { break; }

            let json: serde_json::Value = match serde_json::from_str(data) {
                Ok(v) => v,
                Err(_) => continue,
            };

            let delta = &json["choices"][0]["delta"];

            // Stream content tokens
            if let Some(content) = delta["content"].as_str() {
                if !content.is_empty() {
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

            // Accumulate tool call deltas
            if let Some(tc_deltas) = delta["tool_calls"].as_array() {
                for tc_delta in tc_deltas {
                    let idx = tc_delta["index"].as_u64().unwrap_or(0) as usize;
                    let entry = tc_map.entry(idx).or_insert_with(|| (String::new(), String::new(), String::new()));
                    if let Some(id) = tc_delta["id"].as_str() { entry.0 = id.to_string(); }
                    if let Some(name) = tc_delta["function"]["name"].as_str() { entry.1.push_str(name); }
                    if let Some(args) = tc_delta["function"]["arguments"].as_str() { entry.2.push_str(args); }
                }
            }
        }

        // Build tool_calls list from accumulated map
        let mut tc_indices: Vec<usize> = tc_map.keys().copied().collect();
        tc_indices.sort();
        let tool_calls: Vec<serde_json::Value> = tc_indices
            .iter()
            .filter_map(|idx| tc_map.get(idx))
            .map(|(id, name, args)| serde_json::json!({
                "id": id,
                "type": "function",
                "function": { "name": name, "arguments": args }
            }))
            .collect();

        Ok((full_content, tool_calls))
    }

    pub async fn process_streaming(&self, prompt: &str) -> Result<()> {
        self.event_bus.publish(Event::AgentThinking {
            agent_id: self.id,
            thought_text: "Thinking...".to_string(),
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

        let api_key = api_key
            .filter(|k| !k.is_empty())
            .ok_or_else(|| Error::Agent(
                "No API key set. Get a FREE key at https://openrouter.ai/keys then set OPENROUTER_API_KEY".to_string()
            ))?;

        let mut messages = self.build_messages(prompt);

        // Tool calling loop (max 8 rounds)
        for _round in 0..8 {
            let (content, tool_calls) = self.stream_with_tools(
                &messages, &api_url, &api_key, &model, temp, max_tok,
            ).await?;

            if tool_calls.is_empty() {
                // No tool calls - done
                break;
            }

            // Add assistant message with tool calls
            messages.push(serde_json::json!({
                "role": "assistant",
                "content": if content.is_empty() { serde_json::Value::Null } else { serde_json::json!(content) },
                "tool_calls": tool_calls,
            }));

            // Execute each tool and add results
            for tc in &tool_calls {
                let tool_name = tc["function"]["name"].as_str().unwrap_or("unknown");
                let args_str = tc["function"]["arguments"].as_str().unwrap_or("{}");
                let tc_id = tc["id"].as_str().unwrap_or("call_0");
                let args: serde_json::Value = serde_json::from_str(args_str)
                    .unwrap_or(serde_json::json!({}));

                // Notify UI about tool execution
                let preview = if args_str.len() > 80 {
                    format!("{}...", &args_str[..80])
                } else {
                    args_str.to_string()
                };
                self.event_bus.publish(Event::AgentTokenStream {
                    agent_id: self.id,
                    token: format!("\n[tool] {} {}\n", tool_name, preview),
                    finish_reason: None,
                    timestamp: Utc::now(),
                })?;

                let result = Self::execute_tool(tool_name, &args).await;

                // Show truncated result
                let result_preview = if result.len() > 500 {
                    format!("{}...\n[{} chars total]", &result[..500], result.len())
                } else {
                    result.clone()
                };
                self.event_bus.publish(Event::AgentTokenStream {
                    agent_id: self.id,
                    token: format!("```\n{}\n```\n\n", result_preview),
                    finish_reason: None,
                    timestamp: Utc::now(),
                })?;

                messages.push(serde_json::json!({
                    "role": "tool",
                    "tool_call_id": tc_id,
                    "content": result,
                }));
            }
        }

        self.event_bus.publish(Event::AgentOutput {
            agent_id: self.id,
            content: String::new(),
            timestamp: Utc::now(),
        })?;

        Ok(())
    }
}
