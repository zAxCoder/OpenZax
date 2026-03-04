use crate::event::{Event, EventBus};
use crate::{Error, Result};
use chrono::Utc;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

fn safe_lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}
use uuid::Uuid;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubAgentStatus {
    pub task: String,
    pub done: bool,
}

pub struct Agent {
    id: Uuid,
    config: Mutex<AgentConfig>,
    event_bus: EventBus,
    client: reqwest::Client,
    history: Mutex<Vec<serde_json::Value>>,
    sub_agents: Arc<Mutex<Vec<SubAgentStatus>>>,
    user_memory: Mutex<HashMap<String, String>>,
}

impl Agent {
    pub fn new(config: AgentConfig, event_bus: EventBus) -> Self {
        let mut user_mem = HashMap::new();
        if let Some(home) = dirs::home_dir() {
            let mem_path = home.join(".openzax").join("user_memory.json");
            if let Ok(content) = std::fs::read_to_string(&mem_path) {
                if let Ok(map) = serde_json::from_str::<HashMap<String, String>>(&content) {
                    user_mem = map;
                }
            }
        }
        Self {
            id: Uuid::new_v4(),
            config: Mutex::new(config),
            event_bus,
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .connect_timeout(std::time::Duration::from_secs(15))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
            history: Mutex::new(Vec::new()),
            sub_agents: Arc::new(Mutex::new(Vec::new())),
            user_memory: Mutex::new(user_mem),
        }
    }

    pub fn clear_history(&self) {
        safe_lock(&self.history).clear();
        safe_lock(&self.sub_agents).clear();
    }

    pub fn sub_agent_statuses(&self) -> Vec<SubAgentStatus> {
        safe_lock(&self.sub_agents).clone()
    }

    pub fn save_user_memory(&self, key: &str, value: &str) {
        {
            let mut mem = safe_lock(&self.user_memory);
            mem.insert(key.to_string(), value.to_string());
        }
        self.persist_user_memory();
    }

    pub fn get_user_memory(&self) -> HashMap<String, String> {
        safe_lock(&self.user_memory).clone()
    }

    fn persist_user_memory(&self) {
        if let Some(home) = dirs::home_dir() {
            let dir = home.join(".openzax");
            let _ = std::fs::create_dir_all(&dir);
            let mem = safe_lock(&self.user_memory);
            if let Ok(json) = serde_json::to_string_pretty(&*mem) {
                let _ = std::fs::write(dir.join("user_memory.json"), json);
            }
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn set_system_prompt(&self, prompt: String) {
        safe_lock(&self.config).system_prompt = Some(prompt);
    }

    pub fn set_model(&self, model: String) {
        safe_lock(&self.config).model = model;
    }

    pub fn set_api_url(&self, url: String) {
        safe_lock(&self.config).api_url = url;
    }

    pub fn set_api_key(&self, key: String) {
        safe_lock(&self.config).api_key = Some(key);
    }

    pub fn model_name(&self) -> String {
        safe_lock(&self.config).model.clone()
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
        let cfg = safe_lock(&self.config);
        let history = safe_lock(&self.history);
        let user_mem = safe_lock(&self.user_memory);
        let mut messages = Vec::new();

        let mut system = cfg.system_prompt.clone().unwrap_or_default();

        if !user_mem.is_empty() {
            system.push_str("\n\n## User Profile (remembered across sessions)\n");
            for (k, v) in user_mem.iter() {
                system.push_str(&format!("- {}: {}\n", k, v));
            }
        }

        if !system.is_empty() {
            messages.push(serde_json::json!({"role": "system", "content": system}));
        }

        for msg in history.iter() {
            messages.push(msg.clone());
        }

        messages.push(serde_json::json!({"role": "user", "content": prompt}));
        messages
    }

    fn save_to_history(&self, user_msg: &str, assistant_msg: &str) {
        let mut history = safe_lock(&self.history);
        history.push(serde_json::json!({"role": "user", "content": user_msg}));
        if !assistant_msg.is_empty() {
            history.push(serde_json::json!({"role": "assistant", "content": assistant_msg}));
        }

        let max_history = 50;
        while history.len() > max_history * 2 {
            history.remove(0);
        }
    }

    async fn call_llm(&self, prompt: &str) -> Result<String> {
        let (api_url, api_key, model, temp, max_tok) = {
            let cfg = safe_lock(&self.config);
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

        let response = self
            .client
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
            },
            {
                "type": "function",
                "function": {
                    "name": "search_files",
                    "description": "Search for files matching a glob pattern recursively. Returns matching file paths.",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "pattern": { "type": "string", "description": "Glob pattern (e.g. '*.rs', 'src/**/*.ts', '*.py')" },
                            "directory": { "type": "string", "description": "Starting directory (default: current dir)" }
                        },
                        "required": ["pattern"]
                    }
                }
            },
            {
                "type": "function",
                "function": {
                    "name": "search_text",
                    "description": "Search file contents for a text pattern (regex supported). Returns matching lines with file paths and line numbers.",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "pattern": { "type": "string", "description": "Text or regex pattern to search for" },
                            "directory": { "type": "string", "description": "Directory to search in (default: current dir)" },
                            "file_pattern": { "type": "string", "description": "Only search files matching this glob (e.g. '*.rs')" }
                        },
                        "required": ["pattern"]
                    }
                }
            },
            {
                "type": "function",
                "function": {
                    "name": "spawn_agent",
                    "description": "Spawn a sub-agent to handle a specific task autonomously. The sub-agent has full tool access and will return its result. Use this to parallelize work or delegate focused subtasks.",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "task": { "type": "string", "description": "Detailed task description for the sub-agent" },
                            "model": { "type": "string", "description": "Optional: model to use (default: same as current)" }
                        },
                        "required": ["task"]
                    }
                }
            },
            {
                "type": "function",
                "function": {
                    "name": "remember_user",
                    "description": "Save personal information about the user for future sessions. Use when the user tells you their name, preferences, coding style, or any personal detail they want you to remember permanently.",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "key": { "type": "string", "description": "Category (e.g. 'name', 'language', 'preferred_stack', 'coding_style')" },
                            "value": { "type": "string", "description": "The information to remember" }
                        },
                        "required": ["key", "value"]
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
                                if is_dir {
                                    format!("{}/", name)
                                } else {
                                    name
                                }
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
                let output = std::process::Command::new("sh").args(["-c", cmd]).output();

                match output {
                    Ok(o) => {
                        let stdout = String::from_utf8_lossy(&o.stdout).to_string();
                        let stderr = String::from_utf8_lossy(&o.stderr).to_string();
                        let exit_code = o.status.code().unwrap_or(-1);
                        let mut result = String::new();
                        if !stdout.is_empty() {
                            result.push_str(&stdout);
                        }
                        if !stderr.is_empty() {
                            if !result.is_empty() {
                                result.push('\n');
                            }
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
            "search_files" => {
                let pattern = args["pattern"].as_str().unwrap_or("*");
                let dir = args["directory"].as_str().unwrap_or(".");
                let mut results = Vec::new();
                fn walk_glob(dir: &std::path::Path, pattern: &str, results: &mut Vec<String>, depth: usize) {
                    if depth > 10 { return; }
                    if let Ok(entries) = std::fs::read_dir(dir) {
                        for entry in entries.flatten() {
                            let path = entry.path();
                            let name = entry.file_name().to_string_lossy().into_owned();
                            if path.is_dir() {
                                if !name.starts_with('.') && name != "node_modules" && name != "target" {
                                    walk_glob(&path, pattern, results, depth + 1);
                                }
                            } else {
                                let matches = if pattern.contains('*') {
                                    let pat = pattern.trim_start_matches("**/").replace('*', "");
                                    name.ends_with(&pat) || pat.is_empty()
                                } else {
                                    name.contains(pattern)
                                };
                                if matches {
                                    results.push(path.to_string_lossy().into_owned());
                                }
                            }
                            if results.len() >= 100 { return; }
                        }
                    }
                }
                walk_glob(std::path::Path::new(dir), pattern, &mut results, 0);
                if results.is_empty() {
                    format!("No files matching '{}' found in '{}'", pattern, dir)
                } else {
                    results.join("\n")
                }
            }
            "search_text" => {
                let pattern = args["pattern"].as_str().unwrap_or("");
                let dir = args["directory"].as_str().unwrap_or(".");
                let file_pat = args["file_pattern"].as_str();
                let mut results = Vec::new();
                fn walk_search(dir: &std::path::Path, pattern: &str, file_pat: Option<&str>, results: &mut Vec<String>, depth: usize) {
                    if depth > 10 { return; }
                    if let Ok(entries) = std::fs::read_dir(dir) {
                        for entry in entries.flatten() {
                            let path = entry.path();
                            let name = entry.file_name().to_string_lossy().into_owned();
                            if path.is_dir() {
                                if !name.starts_with('.') && name != "node_modules" && name != "target" {
                                    walk_search(&path, pattern, file_pat, results, depth + 1);
                                }
                            } else {
                                let match_ext = file_pat.map_or(true, |fp| {
                                    let fp = fp.trim_start_matches("**/").trim_start_matches('*');
                                    name.ends_with(fp)
                                });
                                if match_ext {
                                    if let Ok(content) = std::fs::read_to_string(&path) {
                                        for (i, line) in content.lines().enumerate() {
                                            if line.contains(pattern) {
                                                results.push(format!("{}:{}:{}", path.to_string_lossy(), i + 1, line.trim()));
                                                if results.len() >= 50 { return; }
                                            }
                                        }
                                    }
                                }
                            }
                            if results.len() >= 50 { return; }
                        }
                    }
                }
                walk_search(std::path::Path::new(dir), pattern, file_pat, &mut results, 0);
                if results.is_empty() {
                    format!("No matches for '{}' in '{}'", pattern, dir)
                } else {
                    results.join("\n")
                }
            }
            _ => format!("Unknown tool: {}", name),
        }
    }

    async fn spawn_sub_agent(&self, args: &serde_json::Value) -> String {
        let task = args["task"].as_str().unwrap_or("");
        if task.is_empty() {
            return "Error: task description is required".to_string();
        }

        let (api_url, api_key_opt, default_model) = {
            let cfg = safe_lock(&self.config);
            (cfg.api_url.clone(), cfg.api_key.clone(), cfg.model.clone())
        };
        let api_key = match &api_key_opt {
            Some(k) => k.clone(),
            None => return "Error: no API key configured".to_string(),
        };
        let model = args["model"].as_str().unwrap_or(&default_model).to_string();

        let task_short = if task.len() > 60 { format!("{}...", &task[..57]) } else { task.to_string() };
        let agent_idx = {
            let mut subs = safe_lock(&self.sub_agents);
            subs.push(SubAgentStatus { task: task_short.clone(), done: false });
            subs.len() - 1
        };

        let sub_prompt = format!(
r#"You are a focused execution agent for OpenZax. Your ONLY job is to complete the assigned task using your tools.

## CRITICAL RULES — Follow these EXACTLY:
1. ALWAYS use write_file to create files with COMPLETE, REAL content — never empty or placeholder files
2. ALWAYS write FULL code — every function body, every import, every line. No shortcuts, no "TODO", no "..."
3. When building a project, create ALL necessary files: source code, config, dependencies, README
4. After writing files, use execute_command to verify they work (compile, lint, run tests)
5. If a file needs 200 lines of code, write ALL 200 lines. Never truncate or summarize code
6. Include ALL imports, ALL error handling, ALL edge cases in every file you write
7. For web projects: write complete HTML with full CSS and JS inline or properly linked
8. For backend projects: write complete routes, middleware, database connections, models
9. NEVER respond with just text explaining what to do — USE YOUR TOOLS and DO IT

## Your Tools:
- read_file: read any file
- write_file: create/overwrite files (ALWAYS write complete content)
- list_directory: see what files exist
- execute_command: run shell commands (build, test, install, run)
- create_directory: make directories
- delete_file: remove files
- move_file: move/rename files
- search_files: find files by pattern
- search_text: search inside files

## Workflow:
1. Read existing files if relevant to understand context
2. Create necessary directories with create_directory
3. Write ALL files with write_file — every file must be COMPLETE and FUNCTIONAL
4. Run execute_command to verify your work compiles/runs
5. Fix any errors found and re-verify

TASK: {}"#, task
        );

        let sub_config = AgentConfig {
            api_url: api_url.clone(),
            api_key: Some(api_key.clone()),
            model: model.clone(),
            temperature: 0.2,
            max_tokens: 8192,
            system_prompt: Some(sub_prompt),
        };
        let sub_bus = EventBus::default();
        let sub_agent = Agent::new(sub_config, sub_bus);

        let _ = self.event_bus.publish(Event::AgentThinking {
            agent_id: self.id,
            thought_text: format!("[Agent #{}] {}", agent_idx + 1, task_short),
            timestamp: Utc::now(),
        });

        let mut messages = vec![
            serde_json::json!({"role": "user", "content": task}),
        ];

        let mut final_response = String::new();
        for _round in 0..10 {
            match sub_agent.stream_with_tools(&messages, &api_url, &api_key, &model, 0.2, 8192).await {
                Ok((content, tool_calls)) => {
                    if tool_calls.is_empty() {
                        final_response = content;
                        break;
                    }
                    messages.push(serde_json::json!({
                        "role": "assistant",
                        "content": &content,
                        "tool_calls": tool_calls,
                    }));
                    for tc in &tool_calls {
                        let tc_id = tc["id"].as_str().unwrap_or("");
                        let tc_name = tc["function"]["name"].as_str().unwrap_or("");
                        let tc_args_str = tc["function"]["arguments"].as_str().unwrap_or("{}");
                        let parsed: serde_json::Value = serde_json::from_str(tc_args_str).unwrap_or_default();
                        let result = Self::execute_tool(tc_name, &parsed).await;
                        messages.push(serde_json::json!({
                            "role": "tool",
                            "tool_call_id": tc_id,
                            "content": result
                        }));
                    }
                }
                Err(e) => {
                    match sub_agent.stream_simple(&messages, &api_url, &api_key, &model, 0.2, 8192).await {
                        Ok(content) => { final_response = content; break; }
                        Err(_) => { final_response = format!("Agent #{} error: {}", agent_idx + 1, e); break; }
                    }
                }
            }
        }

        {
            let mut subs = safe_lock(&self.sub_agents);
            if let Some(s) = subs.get_mut(agent_idx) {
                s.done = true;
            }
        }

        let _ = self.event_bus.publish(Event::AgentThinking {
            agent_id: self.id,
            thought_text: format!("[Agent #{}] Completed", agent_idx + 1),
            timestamp: Utc::now(),
        });

        if final_response.is_empty() {
            format!("Agent #{} completed but returned no response", agent_idx + 1)
        } else {
            final_response
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

        let response = self
            .client
            .post(api_url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .header("HTTP-Referer", "https://openzax.dev")
            .header("X-Title", "OpenZax")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let err = response.text().await?;
            if status.as_u16() == 401 {
                return Err(Error::Agent(
                    "Invalid API key (401 Unauthorized). Check your key at https://openrouter.ai/keys and re-enter it with /connect".to_string()
                ));
            }
            return Err(Error::Agent(format!("LLM API error ({}): {}", status, err)));
        }

        use tokio::io::AsyncBufReadExt;
        let mut lines = tokio::io::BufReader::new(tokio_util::io::StreamReader::new(
            response
                .bytes_stream()
                .map(|r: std::result::Result<bytes::Bytes, reqwest::Error>| {
                    r.map_err(std::io::Error::other)
                }),
        ))
        .lines();

        let mut full_content = String::new();
        // index → (id, name, args_buf)
        let mut tc_map: HashMap<usize, (String, String, String)> = HashMap::new();

        while let Some(line) = lines.next_line().await? {
            if !line.starts_with("data: ") {
                continue;
            }
            let data = &line[6..];
            if data == "[DONE]" {
                break;
            }

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
                    let entry = tc_map
                        .entry(idx)
                        .or_insert_with(|| (String::new(), String::new(), String::new()));
                    if let Some(id) = tc_delta["id"].as_str() {
                        entry.0 = id.to_string();
                    }
                    if let Some(name) = tc_delta["function"]["name"].as_str() {
                        entry.1.push_str(name);
                    }
                    if let Some(args) = tc_delta["function"]["arguments"].as_str() {
                        entry.2.push_str(args);
                    }
                }
            }
        }

        // Build tool_calls list from accumulated map
        let mut tc_indices: Vec<usize> = tc_map.keys().copied().collect();
        tc_indices.sort();
        let tool_calls: Vec<serde_json::Value> = tc_indices
            .iter()
            .filter_map(|idx| tc_map.get(idx))
            .map(|(id, name, args)| {
                serde_json::json!({
                    "id": id,
                    "type": "function",
                    "function": { "name": name, "arguments": args }
                })
            })
            .collect();

        Ok((full_content, tool_calls))
    }

    async fn stream_simple(
        &self,
        messages: &[serde_json::Value],
        api_url: &str,
        api_key: &str,
        model: &str,
        temp: f32,
        max_tok: usize,
    ) -> Result<String> {
        let request_body = serde_json::json!({
            "model": model,
            "messages": messages,
            "temperature": temp,
            "max_tokens": max_tok,
            "stream": true,
        });

        let response = self
            .client
            .post(api_url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .header("HTTP-Referer", "https://openzax.dev")
            .header("X-Title", "OpenZax")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let err = response.text().await?;
            if status.as_u16() == 401 {
                return Err(Error::Agent(
                    "Invalid API key (401 Unauthorized). Check your key at https://openrouter.ai/keys and re-enter it with /connect".to_string()
                ));
            }
            return Err(Error::Agent(format!("LLM API error ({}): {}", status, err)));
        }

        use tokio::io::AsyncBufReadExt;
        let mut lines = tokio::io::BufReader::new(tokio_util::io::StreamReader::new(
            response
                .bytes_stream()
                .map(|r: std::result::Result<bytes::Bytes, reqwest::Error>| {
                    r.map_err(std::io::Error::other)
                }),
        ))
        .lines();

        let mut full_content = String::new();
        while let Some(line) = lines.next_line().await? {
            if !line.starts_with("data: ") {
                continue;
            }
            let data = &line[6..];
            if data == "[DONE]" {
                break;
            }
            let json: serde_json::Value = match serde_json::from_str(data) {
                Ok(v) => v,
                Err(_) => continue,
            };
            if let Some(content) = json["choices"][0]["delta"]["content"].as_str() {
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
        }

        Ok(full_content)
    }

    pub async fn process_streaming(&self, prompt: &str) -> Result<()> {
        self.event_bus.publish(Event::AgentThinking {
            agent_id: self.id,
            thought_text: "Thinking...".to_string(),
            timestamp: Utc::now(),
        })?;

        let (api_url, api_key, model, temp, max_tok) = {
            let cfg = safe_lock(&self.config);
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
                "No API key set. Get a FREE key at https://openrouter.ai/keys then use /connect to add it".to_string()
            ))?;

        let mut messages = self.build_messages(prompt);
        let mut final_assistant_text = String::new();

        // Try with tools first; fall back to simple streaming if tools aren't supported
        let first_result = self
            .stream_with_tools(&messages, &api_url, &api_key, &model, temp, max_tok)
            .await;

        match first_result {
            Ok((content, tool_calls)) => {
                if tool_calls.is_empty() {
                    final_assistant_text = content;
                    self.save_to_history(prompt, &final_assistant_text);
                    self.event_bus.publish(Event::AgentOutput {
                        agent_id: self.id,
                        content: String::new(),
                        timestamp: Utc::now(),
                    })?;
                    return Ok(());
                }

                // Tool calling loop (max 8 rounds)
                messages.push(serde_json::json!({
                    "role": "assistant",
                    "content": if content.is_empty() { serde_json::Value::Null } else { serde_json::json!(content) },
                    "tool_calls": tool_calls,
                }));

                for tc in &tool_calls {
                    let tool_name = tc["function"]["name"].as_str().unwrap_or("unknown");
                    let args_str = tc["function"]["arguments"].as_str().unwrap_or("{}");
                    let tc_id = tc["id"].as_str().unwrap_or("call_0");
                    let args: serde_json::Value =
                        serde_json::from_str(args_str).unwrap_or(serde_json::json!({}));

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

                    let result = match tool_name {
                        "spawn_agent" => self.spawn_sub_agent(&args).await,
                        "remember_user" => {
                            let key = args["key"].as_str().unwrap_or("");
                            let val = args["value"].as_str().unwrap_or("");
                            self.save_user_memory(key, val);
                            format!("Remembered: {} = {}", key, val)
                        }
                        _ => Self::execute_tool(tool_name, &args).await,
                    };
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

                // Continue tool loop for remaining rounds
                for _round in 1..8 {
                    let (content, tool_calls) = self
                        .stream_with_tools(&messages, &api_url, &api_key, &model, temp, max_tok)
                        .await?;

                    if tool_calls.is_empty() {
                        final_assistant_text = content;
                        break;
                    }

                    messages.push(serde_json::json!({
                        "role": "assistant",
                        "content": if content.is_empty() { serde_json::Value::Null } else { serde_json::json!(content) },
                        "tool_calls": tool_calls,
                    }));

                    for tc in &tool_calls {
                        let tool_name = tc["function"]["name"].as_str().unwrap_or("unknown");
                        let args_str = tc["function"]["arguments"].as_str().unwrap_or("{}");
                        let tc_id = tc["id"].as_str().unwrap_or("call_0");
                        let args: serde_json::Value =
                            serde_json::from_str(args_str).unwrap_or(serde_json::json!({}));

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

                        let result = match tool_name {
                            "spawn_agent" => self.spawn_sub_agent(&args).await,
                            "remember_user" => {
                                let key = args["key"].as_str().unwrap_or("");
                                let val = args["value"].as_str().unwrap_or("");
                                self.save_user_memory(key, val);
                                format!("Remembered: {} = {}", key, val)
                            }
                            _ => Self::execute_tool(tool_name, &args).await,
                        };
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
            }
            Err(e) => {
                let err_str = format!("{}", e);
                if err_str.contains("401") {
                    return Err(e);
                }
                let fallback = self.stream_simple(&messages, &api_url, &api_key, &model, temp, max_tok)
                    .await?;
                final_assistant_text = fallback;
            }
        }

        self.save_to_history(prompt, &final_assistant_text);

        self.event_bus.publish(Event::AgentOutput {
            agent_id: self.id,
            content: String::new(),
            timestamp: Utc::now(),
        })?;

        Ok(())
    }
}
