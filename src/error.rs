//! Error types for gamecode-prompt

use thiserror::Error;

/// Result type for prompt operations
pub type Result<T> = std::result::Result<T, PromptError>;

/// Errors that can occur during prompt management
#[derive(Error, Debug)]
pub enum PromptError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Template error: {0}")]
    Template(#[from] handlebars::TemplateError),

    #[error("Render error: {0}")]
    Render(#[from] handlebars::RenderError),

    #[error("Prompt not found: {0}")]
    PromptNotFound(String),

    #[error("Invalid prompt: {0}")]
    InvalidPrompt(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Template validation error: {0}")]
    TemplateValidation(String),
}