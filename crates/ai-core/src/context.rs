use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContextError {
    #[error("Context window exceeded: {0}/{1} tokens")]
    WindowExceeded(u32, u32),
    #[error("Compression failed: {0}")]
    CompressionFailed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
    pub token_count: u32,
    pub timestamp: DateTime<Utc>,
}

impl Message {
    pub fn new(role: impl Into<String>, content: impl Into<String>) -> Self {
        let content = content.into();
        let token_count = ContextCompressor::estimate_tokens(&content);
        Self {
            role: role.into(),
            content,
            token_count,
            timestamp: Utc::now(),
        }
    }

    pub fn system(content: impl Into<String>) -> Self {
        Self::new("system", content)
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self::new("user", content)
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self::new("assistant", content)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextWindow {
    pub messages: VecDeque<Message>,
    pub max_tokens: u32,
    pub used_tokens: u32,
}

impl ContextWindow {
    pub fn new(max_tokens: u32) -> Self {
        Self {
            messages: VecDeque::new(),
            max_tokens,
            used_tokens: 0,
        }
    }

    pub fn remaining_tokens(&self) -> u32 {
        self.max_tokens.saturating_sub(self.used_tokens)
    }

    pub fn utilization(&self) -> f32 {
        self.used_tokens as f32 / self.max_tokens as f32
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CompressionStrategy {
    SlidingWindow,
    RecursiveSummarize,
    SemanticRetrieval,
    AggressivePrune,
}

pub struct ContextCompressor;

impl ContextCompressor {
    /// Simple token estimation: ~4 characters per token (GPT-style approximation).
    pub fn estimate_tokens(text: &str) -> u32 {
        (text.len() as f32 / 4.0).ceil() as u32
    }

    pub fn add_message(window: &mut ContextWindow, msg: Message) {
        window.used_tokens += msg.token_count;
        window.messages.push_back(msg);
    }

    pub fn sliding_window(window: &mut ContextWindow, keep_last_n: usize) {
        // Always keep the first message (system prompt)
        let system_prompt = window
            .messages
            .front()
            .filter(|m| m.role == "system")
            .cloned();

        let total = window.messages.len();
        if total <= keep_last_n {
            return;
        }

        let start_idx = if system_prompt.is_some() { 1 } else { 0 };
        let drop_count = total.saturating_sub(keep_last_n).saturating_sub(start_idx);

        let mut tokens_freed = 0u32;
        for _ in 0..drop_count {
            if let Some(removed) = window.messages.remove(start_idx) {
                tokens_freed += removed.token_count;
            }
        }
        window.used_tokens = window.used_tokens.saturating_sub(tokens_freed);
    }

    /// Creates a summary message from a slice of messages.
    /// In production, `model_fn` would call an LLM to summarize.
    pub fn summarize_range<F>(messages: &[Message], model_fn: F) -> Message
    where
        F: Fn(&str) -> String,
    {
        let combined: String = messages
            .iter()
            .map(|m| format!("[{}]: {}", m.role, m.content))
            .collect::<Vec<_>>()
            .join("\n");
        let summary_text = model_fn(&combined);
        Message {
            role: "system".to_string(),
            content: format!("[SUMMARY OF PRIOR CONTEXT]\n{}", summary_text),
            token_count: Self::estimate_tokens(&summary_text),
            timestamp: Utc::now(),
        }
    }

    pub fn compress<F>(
        window: &mut ContextWindow,
        strategy: CompressionStrategy,
        summarizer: Option<F>,
    ) -> Result<(), ContextError>
    where
        F: Fn(&str) -> String,
    {
        match strategy {
            CompressionStrategy::SlidingWindow => {
                // Keep last 75% of messages when at 90%+ utilization
                if window.utilization() > 0.9 {
                    let keep = (window.messages.len() as f32 * 0.75) as usize;
                    Self::sliding_window(window, keep.max(2));
                }
            }
            CompressionStrategy::RecursiveSummarize => {
                if window.utilization() > 0.8 {
                    let summarizer = summarizer.ok_or_else(|| {
                        ContextError::CompressionFailed(
                            "RecursiveSummarize requires a summarizer function".into(),
                        )
                    })?;
                    // Summarize the oldest 50% of messages (excluding system)
                    let has_system = window
                        .messages
                        .front()
                        .map(|m| m.role == "system")
                        .unwrap_or(false);
                    let start = if has_system { 1 } else { 0 };
                    let mid = start + (window.messages.len().saturating_sub(start)) / 2;
                    if mid > start {
                        let to_summarize: Vec<Message> =
                            window.messages.drain(start..mid).collect();
                        let freed_tokens: u32 = to_summarize.iter().map(|m| m.token_count).sum();
                        window.used_tokens = window.used_tokens.saturating_sub(freed_tokens);
                        let summary = Self::summarize_range(&to_summarize, summarizer);
                        window.used_tokens += summary.token_count;
                        window.messages.insert(start, summary);
                    }
                }
            }
            CompressionStrategy::AggressivePrune => {
                // Remove all messages except system prompt and the most recent 3
                let has_system = window
                    .messages
                    .front()
                    .map(|m| m.role == "system")
                    .unwrap_or(false);
                let keep_count = if has_system { 4 } else { 3 };
                let total = window.messages.len();
                if total > keep_count {
                    let start = if has_system { 1 } else { 0 };
                    let end = total.saturating_sub(3);
                    let to_remove = end.saturating_sub(start);
                    let mut freed = 0u32;
                    for _ in 0..to_remove {
                        if let Some(msg) = window.messages.remove(start) {
                            freed += msg.token_count;
                        }
                    }
                    window.used_tokens = window.used_tokens.saturating_sub(freed);
                }
            }
            CompressionStrategy::SemanticRetrieval => {
                // Placeholder: would use embeddings to find and keep most relevant messages
                // For now, falls back to sliding window
                if window.utilization() > 0.9 {
                    let keep = (window.messages.len() as f32 * 0.6) as usize;
                    Self::sliding_window(window, keep.max(2));
                }
            }
        }
        Ok(())
    }
}

pub struct ContextAssembler;

impl ContextAssembler {
    /// Assembles a context window from components, respecting token budget.
    /// Priority: system_prompt > current_task > recent_history > retrieved_docs
    pub fn assemble(
        system_prompt: &str,
        history: &[Message],
        relevant_docs: &[Message],
        current_task: &str,
        budget_tokens: u32,
    ) -> Vec<Message> {
        let mut result = vec![];
        let mut used = 0u32;

        // 1. System prompt (always included)
        let sys_msg = Message::system(system_prompt);
        used += sys_msg.token_count;
        result.push(sys_msg);

        // 2. Current task
        let task_msg = Message::user(current_task);
        if used + task_msg.token_count <= budget_tokens {
            used += task_msg.token_count;
            result.push(task_msg);
        }

        // 3. Recent history (newest first, then reversed for chronological order)
        let mut history_budget = (budget_tokens.saturating_sub(used)) * 6 / 10;
        let mut selected_history: Vec<&Message> = vec![];
        for msg in history.iter().rev() {
            if history_budget >= msg.token_count {
                history_budget -= msg.token_count;
                used += msg.token_count;
                selected_history.push(msg);
            }
        }
        selected_history.reverse();
        for msg in selected_history {
            result.insert(result.len() - 1, msg.clone());
        }

        // 4. Retrieved docs (remaining budget)
        for doc in relevant_docs {
            if used + doc.token_count <= budget_tokens {
                used += doc.token_count;
                result.push(doc.clone());
            }
        }

        result
    }

    /// Estimate whether the assembled context fits within the model's context window.
    pub fn fits_in_window(messages: &[Message], max_tokens: u32) -> bool {
        let total: u32 = messages.iter().map(|m| m.token_count).sum();
        total <= max_tokens
    }

    /// Total token count of a message slice.
    pub fn total_tokens(messages: &[Message]) -> u32 {
        messages.iter().map(|m| m.token_count).sum()
    }
}
