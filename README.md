# gamecode-prompt

A Rust library for system prompt management in gamecode applications.

## Features

- **Default Prompt Management**: Store and edit default system prompts in user config directory
- **Named Prompts**: Save, load, and manage multiple named system prompts
- **Template Support**: Variable substitution using Handlebars templating
- **Cross-Platform Storage**: Platform-specific config directories
- **Validation**: Prompt validation and preprocessing
- **Simple API**: Easy-to-use interface for prompt operations

## Quick Start

```rust
use gamecode_prompt::PromptManager;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a prompt manager
    let mut manager = PromptManager::new()?;

    // Load the default system prompt
    let default_prompt = manager.load_default()?;
    println!("Default prompt: {}", default_prompt);

    // Save a custom named prompt
    manager.save_prompt("coding", "You are an expert Rust programmer.")?;

    // Load the custom prompt
    let coding_prompt = manager.load_prompt("coding")?;
    println!("Coding prompt: {}", coding_prompt);

    // Use template variables
    let template_prompt = "You are a {{role}} specializing in {{language}}.";
    let mut vars = std::collections::HashMap::new();
    vars.insert("role".to_string(), "programmer".to_string());
    vars.insert("language".to_string(), "Rust".to_string());
    let rendered = manager.render_template(template_prompt, &vars)?;
    println!("Rendered: {}", rendered);

    Ok(())
}
```

## Storage Locations

Prompts are stored in platform-specific directories:
- **macOS**: `~/Library/Application Support/gamecode/prompts/`
- **Linux**: `~/.config/gamecode/prompts/`
- **Windows**: `%APPDATA%/gamecode/prompts/`

The default prompt is saved as `default.txt` and can be edited by users.

## Template System

The library uses Handlebars for template variable substitution with custom helpers:

### Basic Variables
```
Hello {{name}}, you are working with {{language}}.
```

### Custom Helpers

- **`{{upper text}}`** - Convert to uppercase
- **`{{lower text}}`** - Convert to lowercase  
- **`{{capitalize text}}`** - Capitalize first letter
- **`{{default variable fallback}}`** - Use fallback if variable is empty

### Example Template
```
Hello {{capitalize user_name}}!

You are an expert in {{lower task}} with {{upper language}}.
Your experience level is {{default experience "beginner"}}.
```

## API Reference

### PromptManager

```rust
// Create new manager
let manager = PromptManager::new()?;
let manager = PromptManager::with_config(config)?;

// Default prompt operations
let prompt = manager.load_default()?;
manager.save_default("New default prompt")?;
manager.reset_default()?; // Reset to factory default

// Named prompt operations
let prompt = manager.load_prompt("name")?;
manager.save_prompt("name", "Prompt content")?;
manager.delete_prompt("name")?;
let prompts = manager.list_prompts()?;
let exists = manager.prompt_exists("name");

// Template operations
let rendered = manager.render_template(template, &variables)?;

// Metadata
let info = manager.get_prompt_info("name")?;
```

### Configuration

```rust
use gamecode_prompt::{Config, PromptManager};

let config = Config {
    storage_dir: Some("/custom/path".into()),
    validate_templates: true,
    max_prompt_length: 5000,
};

let manager = PromptManager::with_config(config)?;
```

## Error Handling

The library uses `anyhow::Result` for error handling and provides detailed error types:

- `PromptError::PromptNotFound` - Prompt doesn't exist
- `PromptError::InvalidPrompt` - Prompt validation failed
- `PromptError::TemplateError` - Template syntax error
- `PromptError::Storage` - File system errors

## Integration

This crate is designed to work independently or with other gamecode crates:

```toml
[dependencies]
gamecode-prompt = { git = "https://github.com/navicore/gamecode-prompt" }
```

## License

This project is licensed under the MIT License.