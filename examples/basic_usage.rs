use gamecode_prompt::{PromptManager, Config};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging for better debugging
    tracing_subscriber::fmt::init();

    println!("=== GameCode Prompt Manager Demo ===\n");

    // Create a prompt manager
    let mut manager = PromptManager::new()?;
    println!("✓ Created prompt manager");

    // 1. Work with default prompt
    println!("\n--- Default Prompt ---");
    let default_prompt = manager.load_default()?;
    println!("Default prompt preview:\n{}\n", 
        default_prompt.lines().take(3).collect::<Vec<_>>().join("\n"));

    // 2. Save and load named prompts
    println!("--- Named Prompts ---");
    
    // Save some example prompts
    manager.save_prompt("coding", 
        "You are an expert Rust programmer who writes clean, efficient, and well-documented code.")?;
    
    manager.save_prompt("debugging", 
        "You are a debugging specialist. Help identify and fix issues in code systematically.")?;
    
    manager.save_prompt("code-review", 
        "You are a senior developer conducting code reviews. Focus on best practices, security, and maintainability.")?;
    
    println!("✓ Saved example prompts");

    // List all prompts
    let prompts = manager.list_prompts()?;
    println!("Available prompts: {:?}", prompts);

    // Load and display a specific prompt
    let coding_prompt = manager.load_prompt("coding")?;
    println!("Coding prompt: {}", coding_prompt);

    // 3. Template functionality
    println!("\n--- Template System ---");
    
    let template_prompt = r#"You are a {{role}} specializing in {{language}}.

Your responsibilities:
- Write {{quality}} code
- Follow {{language}} best practices  
- Provide {{default explanation_style "detailed"}} explanations

Hello {{capitalize user_name}}! Let's work with {{upper language}} today."#;

    // Create variables for template
    let mut vars = HashMap::new();
    vars.insert("role".to_string(), "senior developer".to_string());
    vars.insert("language".to_string(), "rust".to_string());
    vars.insert("quality".to_string(), "production-quality".to_string());
    vars.insert("user_name".to_string(), "alice".to_string());
    vars.insert("explanation_style".to_string(), "".to_string()); // Empty to test default helper

    // Render the template
    let rendered = manager.render_template(template_prompt, &vars)?;
    println!("Rendered template:\n{}", rendered);

    // 4. Prompt metadata
    println!("\n--- Prompt Information ---");
    let info = manager.get_prompt_info("coding")?;
    println!("Coding prompt info:");
    println!("  Name: {}", info.name);
    println!("  Size: {} bytes", info.size);
    println!("  File: {}", info.file_path.display());

    // 5. Custom configuration example
    println!("\n--- Custom Configuration ---");
    let config = Config {
        storage_dir: None, // Use default
        validate_templates: true,
        max_prompt_length: 1000, // Smaller limit for demo
    };

    let mut custom_manager = PromptManager::with_config(config)?;
    println!("✓ Created manager with custom config");

    // Try to save a prompt that exceeds the limit
    let long_prompt = "This is a very long prompt. ".repeat(50);
    match custom_manager.save_prompt("too-long", &long_prompt) {
        Ok(_) => println!("Saved long prompt successfully"),
        Err(e) => println!("Expected error for long prompt: {}", e),
    }

    println!("\n=== Demo Complete ===");
    Ok(())
}

// Helper to add tracing subscriber dependency
use tracing_subscriber;