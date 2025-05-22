use crate::error::{PromptError, Result};
use handlebars::{Handlebars, Helper, HelperResult, Output, RenderContext};
use serde_json::Value;
use std::collections::HashMap;
use tracing::debug;

/// Template engine for prompt variable substitution
pub struct TemplateEngine {
    handlebars: Handlebars<'static>,
}

impl TemplateEngine {
    /// Create a new template engine
    pub fn new() -> Self {
        let mut handlebars = Handlebars::new();
        
        // Register custom helpers
        handlebars.register_helper("upper", Box::new(upper_helper));
        handlebars.register_helper("lower", Box::new(lower_helper));
        handlebars.register_helper("capitalize", Box::new(capitalize_helper));
        handlebars.register_helper("default", Box::new(default_helper));
        
        // Configure handlebars
        handlebars.set_strict_mode(false); // Allow missing variables
        
        Self { handlebars }
    }
    
    /// Render a template with variables
    pub fn render(&self, template: &str, variables: &HashMap<String, String>) -> Result<String> {
        // Convert HashMap to serde_json::Value for handlebars
        let context: Value = variables.iter()
            .map(|(k, v)| (k.clone(), Value::String(v.clone())))
            .collect::<serde_json::Map<String, Value>>()
            .into();
        
        let rendered = self.handlebars.render_template(template, &context)?;
        debug!("Rendered template with {} variables", variables.len());
        Ok(rendered)
    }
    
    /// Validate a template for syntax errors
    pub fn validate_template(&self, template: &str) -> Result<()> {
        match self.handlebars.render_template(template, &Value::Object(serde_json::Map::new())) {
            Ok(_) => Ok(()),
            Err(e) => {
                // All render errors during validation indicate template issues
                Err(PromptError::TemplateValidation(format!("Invalid template syntax: {}", e)))
            }
        }
    }
    
    /// Extract variable names from a template
    pub fn extract_variables(&self, template: &str) -> Result<Vec<String>> {
        let mut variables = Vec::new();
        
        // Parse the template to extract variable names
        // This is a simple implementation - handlebars doesn't expose the AST directly
        let mut chars = template.chars().peekable();
        let mut in_variable = false;
        let mut current_var = String::new();
        let mut brace_count: i32 = 0;
        
        while let Some(ch) = chars.next() {
            if ch == '{' {
                brace_count += 1;
                if brace_count == 2 {
                    in_variable = true;
                    current_var.clear();
                }
            } else if ch == '}' {
                if in_variable && brace_count == 2 {
                    in_variable = false;
                    brace_count = 0;
                    
                    // Clean up variable name (remove helpers, etc.)
                    let var_name = current_var.trim().split_whitespace().next().unwrap_or("");
                    if !var_name.is_empty() && !var_name.starts_with('#') && !var_name.starts_with('/') {
                        variables.push(var_name.to_string());
                    }
                } else {
                    brace_count = brace_count.saturating_sub(1);
                }
            } else if in_variable {
                current_var.push(ch);
            } else {
                brace_count = 0;
            }
        }
        
        // Remove duplicates and sort
        variables.sort();
        variables.dedup();
        
        debug!("Extracted {} variables from template", variables.len());
        Ok(variables)
    }
    
    /// Check if a template has all required variables
    pub fn check_variables(&self, template: &str, provided: &HashMap<String, String>) -> Result<Vec<String>> {
        let required = self.extract_variables(template)?;
        let missing: Vec<String> = required.iter()
            .filter(|var| !provided.contains_key(*var))
            .cloned()
            .collect();
        
        Ok(missing)
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

// Custom Handlebars helpers

/// Convert text to uppercase
fn upper_helper(
    h: &Helper,
    _: &Handlebars,
    _: &handlebars::Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    if let Some(param) = h.param(0) {
        let value = param.value().as_str().unwrap_or("");
        out.write(&value.to_uppercase())?;
    }
    Ok(())
}

/// Convert text to lowercase
fn lower_helper(
    h: &Helper,
    _: &Handlebars,
    _: &handlebars::Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    if let Some(param) = h.param(0) {
        let value = param.value().as_str().unwrap_or("");
        out.write(&value.to_lowercase())?;
    }
    Ok(())
}

/// Capitalize first letter of text
fn capitalize_helper(
    h: &Helper,
    _: &Handlebars,
    _: &handlebars::Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    if let Some(param) = h.param(0) {
        let value = param.value().as_str().unwrap_or("");
        let capitalized = if value.is_empty() {
            String::new()
        } else {
            let mut chars = value.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        };
        out.write(&capitalized)?;
    }
    Ok(())
}

/// Provide default value if variable is missing or empty
fn default_helper(
    h: &Helper,
    _: &Handlebars,
    _: &handlebars::Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    let value = h.param(0)
        .and_then(|p| p.value().as_str())
        .unwrap_or("");
    
    let default_value = h.param(1)
        .and_then(|p| p.value().as_str())
        .unwrap_or("");
    
    if value.is_empty() {
        out.write(default_value)?;
    } else {
        out.write(value)?;
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_template_rendering() {
        let engine = TemplateEngine::new();
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "World".to_string());
        vars.insert("greeting".to_string(), "Hello".to_string());
        
        let template = "{{greeting}}, {{name}}!";
        let result = engine.render(template, &vars).unwrap();
        assert_eq!(result, "Hello, World!");
    }
    
    #[test]
    fn test_custom_helpers() {
        let engine = TemplateEngine::new();
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "john doe".to_string());
        vars.insert("role".to_string(), "DEVELOPER".to_string());
        
        // Test upper helper
        let template = "Hello {{upper name}}!";
        let result = engine.render(template, &vars).unwrap();
        assert_eq!(result, "Hello JOHN DOE!");
        
        // Test lower helper
        let template = "You are a {{lower role}}.";
        let result = engine.render(template, &vars).unwrap();
        assert_eq!(result, "You are a developer.");
        
        // Test capitalize helper
        let template = "{{capitalize name}} is here.";
        let result = engine.render(template, &vars).unwrap();
        assert_eq!(result, "John doe is here.");
    }
    
    #[test]
    fn test_default_helper() {
        let engine = TemplateEngine::new();
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "Alice".to_string());
        // Note: missing "role" variable
        
        let template = "Hello {{name}}, you are a {{default role \"assistant\"}}.";
        let result = engine.render(template, &vars).unwrap();
        assert_eq!(result, "Hello Alice, you are a assistant.");
    }
    
    #[test]
    fn test_variable_extraction() {
        let engine = TemplateEngine::new();
        let template = "Hello {{name}}, you are a {{role}} working with {{language}}.";
        let variables = engine.extract_variables(template).unwrap();
        
        assert_eq!(variables.len(), 3);
        assert!(variables.contains(&"name".to_string()));
        assert!(variables.contains(&"role".to_string()));
        assert!(variables.contains(&"language".to_string()));
    }
    
    #[test]
    fn test_template_validation() {
        let engine = TemplateEngine::new();
        
        // Valid template
        assert!(engine.validate_template("Hello {{name}}!").is_ok());
        
        // Invalid template (unclosed braces)
        assert!(engine.validate_template("Hello {{name}").is_err());
    }
    
    #[test]
    fn test_variable_checking() {
        let engine = TemplateEngine::new();
        let template = "Hello {{name}}, you are a {{role}}.";
        
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "Alice".to_string());
        // Missing "role" variable
        
        let missing = engine.check_variables(template, &vars).unwrap();
        assert_eq!(missing.len(), 1);
        assert_eq!(missing[0], "role");
    }
    
    #[test]
    fn test_complex_template() {
        let engine = TemplateEngine::new();
        let mut vars = HashMap::new();
        vars.insert("user_name".to_string(), "alice".to_string());
        vars.insert("task".to_string(), "CODING".to_string());
        vars.insert("language".to_string(), "rust".to_string());
        vars.insert("experience".to_string(), "".to_string()); // Empty string
        
        let template = r#"Hello {{capitalize user_name}}!
        
You are an expert in {{lower task}} with {{upper language}}.
Your experience level is {{default experience "beginner"}}."#;
        
        let result = engine.render(template, &vars).unwrap();
        let expected = r#"Hello Alice!
        
You are an expert in coding with RUST.
Your experience level is beginner."#;
        
        assert_eq!(result, expected);
    }
}