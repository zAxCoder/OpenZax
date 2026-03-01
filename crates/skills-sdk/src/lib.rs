pub use openzax_skills_macros::*;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SkillError {
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("IO error: {0}")]
    Io(String),
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Permission denied: {0}")]
    Permission(String),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

pub type SkillResult<T> = Result<T, SkillError>;

/// Core trait that all skills must implement
pub trait Skill {
    fn name(&self) -> &str;
    fn version(&self) -> &str {
        "0.1.0"
    }
    fn description(&self) -> &str {
        ""
    }
}

/// Context provided to skills at runtime
pub struct SkillContext {
    config: HashMap<String, String>,
}

impl SkillContext {
    pub fn new() -> Self {
        Self {
            config: HashMap::new(),
        }
    }
    
    /// Get configuration value
    pub fn get_config(&self, key: &str) -> Option<&String> {
        self.config.get(key)
    }
    
    /// Set configuration value
    pub fn set_config(&mut self, key: String, value: String) {
        self.config.insert(key, value);
    }
    
    /// Log info message
    pub fn log_info(&self, message: &str) {
        #[cfg(target_arch = "wasm32")]
        {
            // Call WIT logging interface
            println!("[INFO] {}", message);
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            println!("[INFO] {}", message);
        }
    }
    
    /// Log error message
    pub fn log_error(&self, message: &str) {
        #[cfg(target_arch = "wasm32")]
        {
            eprintln!("[ERROR] {}", message);
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            eprintln!("[ERROR] {}", message);
        }
    }
    
    /// Read file content
    pub fn read_file(&self, path: &str) -> SkillResult<Vec<u8>> {
        #[cfg(target_arch = "wasm32")]
        {
            // Call WIT fs interface
            Err(SkillError::Internal("Not implemented in WASM".to_string()))
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            std::fs::read(path).map_err(|e| SkillError::Io(e.to_string()))
        }
    }
    
    /// Write file content
    pub fn write_file(&self, path: &str, content: &[u8]) -> SkillResult<()> {
        #[cfg(target_arch = "wasm32")]
        {
            Err(SkillError::Internal("Not implemented in WASM".to_string()))
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            std::fs::write(path, content).map_err(|e| SkillError::Io(e.to_string()))
        }
    }
    
    /// Make HTTP request
    pub fn http_get(&self, url: &str) -> SkillResult<String> {
        #[cfg(target_arch = "wasm32")]
        {
            Err(SkillError::Internal("Not implemented in WASM".to_string()))
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Err(SkillError::Network("HTTP not available in test mode".to_string()))
        }
    }
}

impl Default for SkillContext {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SkillManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub license: String,
    pub permissions: Vec<String>,
    pub dependencies: HashMap<String, String>,
}

impl SkillManifest {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: "0.1.0".to_string(),
            description: String::new(),
            author: String::new(),
            license: "MIT".to_string(),
            permissions: Vec::new(),
            dependencies: HashMap::new(),
        }
    }
    
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }
    
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = author.into();
        self
    }
    
    pub fn with_permission(mut self, perm: impl Into<String>) -> Self {
        self.permissions.push(perm.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_skill_context() {
        let mut ctx = SkillContext::new();
        ctx.set_config("key".to_string(), "value".to_string());
        assert_eq!(ctx.get_config("key"), Some(&"value".to_string()));
    }
    
    #[test]
    fn test_manifest_builder() {
        let manifest = SkillManifest::new("test-skill")
            .with_description("A test skill")
            .with_author("Test Author")
            .with_permission("fs:read");
        
        assert_eq!(manifest.name, "test-skill");
        assert_eq!(manifest.description, "A test skill");
        assert_eq!(manifest.permissions.len(), 1);
    }
}
