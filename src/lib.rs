//! # gamecode-prompt
//!
//! A Rust library for system prompt management in gamecode applications.
//!
//! ## Features
//!
//! - **Default Prompt Management**: Store and edit default system prompts in user config directory
//! - **Named Prompts**: Save, load, and manage multiple named system prompts
//! - **Template Support**: Variable substitution using Handlebars templating
//! - **Cross-Platform Storage**: Platform-specific config directories
//! - **Validation**: Prompt validation and preprocessing
//! - **Simple API**: Easy-to-use interface for prompt operations
//!
//! ## Quick Start
//!
//! ```rust
//! use gamecode_prompt::PromptManager;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a prompt manager
//! let mut manager = PromptManager::new()?;
//!
//! // Load the default system prompt
//! let default_prompt = manager.load_default()?;
//!
//! // Save a custom named prompt
//! manager.save_prompt("coding", "You are an expert Rust programmer.")?;
//!
//! // Load the custom prompt
//! let coding_prompt = manager.load_prompt("coding")?;
//!
//! // Use template variables
//! let template_prompt = "You are a {{role}} specializing in {{language}}.";
//! let mut vars = std::collections::HashMap::new();
//! vars.insert("role".to_string(), "programmer".to_string());
//! vars.insert("language".to_string(), "Rust".to_string());
//! let rendered = manager.render_template(template_prompt, &vars)?;
//! # Ok(())
//! # }
//! ```

pub mod storage;
pub mod template;
pub mod error;

use crate::error::{PromptError, Result};
use crate::storage::PromptStorage;
use crate::template::TemplateEngine;
use std::collections::HashMap;

/// Configuration for prompt management
#[derive(Debug, Clone)]
pub struct Config {
    /// Custom storage directory (uses default if None)
    pub storage_dir: Option<std::path::PathBuf>,
    /// Enable template validation
    pub validate_templates: bool,
    /// Maximum prompt length in characters
    pub max_prompt_length: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            storage_dir: None,
            validate_templates: true,
            max_prompt_length: 10000,
        }
    }
}

/// Main interface for prompt management
pub struct PromptManager {
    storage: Box<dyn PromptStorage>,
    template_engine: TemplateEngine,
    config: Config,
}

impl PromptManager {
    /// Create a new prompt manager with default configuration
    pub fn new() -> Result<Self> {
        let config = Config::default();
        Self::with_config(config)
    }
    
    /// Create a new prompt manager with custom configuration
    pub fn with_config(config: Config) -> Result<Self> {
        let storage = match &config.storage_dir {
            Some(dir) => crate::storage::FileStorage::with_directory(dir)?,
            None => crate::storage::FileStorage::new()?,
        };
        
        Ok(Self {
            storage: Box::new(storage),
            template_engine: TemplateEngine::new(),
            config,
        })
    }
    
    /// Load the default system prompt
    pub fn load_default(&self) -> Result<String> {
        self.storage.load_default()
    }
    
    /// Save the default system prompt
    pub fn save_default(&mut self, prompt: &str) -> Result<()> {
        self.validate_prompt(prompt)?;
        self.storage.save_default(prompt)
    }
    
    /// Load a named prompt
    pub fn load_prompt(&self, name: &str) -> Result<String> {
        self.storage.load_prompt(name)
    }
    
    /// Save a named prompt
    pub fn save_prompt(&mut self, name: &str, prompt: &str) -> Result<()> {
        self.validate_prompt(prompt)?;
        self.storage.save_prompt(name, prompt)
    }
    
    /// List all available named prompts
    pub fn list_prompts(&self) -> Result<Vec<String>> {
        self.storage.list_prompts()
    }
    
    /// Delete a named prompt
    pub fn delete_prompt(&mut self, name: &str) -> Result<()> {
        self.storage.delete_prompt(name)
    }
    
    /// Check if a prompt exists
    pub fn prompt_exists(&self, name: &str) -> bool {
        self.storage.prompt_exists(name)
    }
    
    /// Render a template with variables
    pub fn render_template(&self, template: &str, variables: &HashMap<String, String>) -> Result<String> {
        if self.config.validate_templates {
            self.template_engine.validate_template(template)?;
        }
        
        let rendered = self.template_engine.render(template, variables)?;
        self.validate_prompt(&rendered)?;
        Ok(rendered)
    }
    
    /// Get prompt metadata (size, modification time, etc.)
    pub fn get_prompt_info(&self, name: &str) -> Result<crate::storage::PromptInfo> {
        self.storage.get_prompt_info(name)
    }
    
    /// Reset to factory default prompt
    pub fn reset_default(&mut self) -> Result<()> {
        let factory_default = Self::factory_default_prompt();
        self.save_default(&factory_default)
    }
    
    /// Get the factory default prompt
    pub fn factory_default_prompt() -> String {
        r#"You are Claude, an AI assistant created by Anthropic. You are helpful, harmless, and honest.

When helping with code:
- Provide clear, concise explanations
- Follow best practices and conventions
- Consider security and performance implications
- Test your suggestions when possible

When helping with general tasks:
- Be direct and actionable
- Ask clarifying questions when needed
- Provide step-by-step guidance for complex tasks
- Acknowledge limitations or uncertainties"#.to_string()
    }
    
    /// Validate a prompt according to current config
    fn validate_prompt(&self, prompt: &str) -> Result<()> {
        if prompt.trim().is_empty() {
            return Err(PromptError::InvalidPrompt("Prompt cannot be empty".to_string()));
        }
        
        if prompt.len() > self.config.max_prompt_length {
            return Err(PromptError::InvalidPrompt(
                format!("Prompt exceeds maximum length of {} characters", self.config.max_prompt_length)
            ));
        }
        
        Ok(())
    }
}

impl Default for PromptManager {
    fn default() -> Self {
        Self::new().expect("Failed to create default PromptManager")
    }
}

// Re-export important types
pub use crate::storage::PromptInfo;