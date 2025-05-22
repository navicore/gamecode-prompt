use crate::error::{PromptError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tracing::{debug, info};

/// Trait for prompt storage backends
pub trait PromptStorage: Send + Sync {
    /// Load the default system prompt
    fn load_default(&self) -> Result<String>;
    
    /// Save the default system prompt
    fn save_default(&self, prompt: &str) -> Result<()>;
    
    /// Load a named prompt
    fn load_prompt(&self, name: &str) -> Result<String>;
    
    /// Save a named prompt
    fn save_prompt(&self, name: &str, prompt: &str) -> Result<()>;
    
    /// List all available named prompts
    fn list_prompts(&self) -> Result<Vec<String>>;
    
    /// Delete a named prompt
    fn delete_prompt(&self, name: &str) -> Result<()>;
    
    /// Check if a prompt exists
    fn prompt_exists(&self, name: &str) -> bool;
    
    /// Get prompt metadata
    fn get_prompt_info(&self, name: &str) -> Result<PromptInfo>;
}

/// Information about a stored prompt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptInfo {
    pub name: String,
    pub size: u64,
    pub created_at: SystemTime,
    pub modified_at: SystemTime,
    pub file_path: PathBuf,
}

/// Metadata for a prompt collection
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PromptMetadata {
    version: String,
    prompts: HashMap<String, PromptEntry>,
}

/// Individual prompt entry in metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PromptEntry {
    name: String,
    file_name: String,
    created_at: SystemTime,
    modified_at: SystemTime,
    size: u64,
}

impl Default for PromptMetadata {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            prompts: HashMap::new(),
        }
    }
}

/// File-based prompt storage implementation
pub struct FileStorage {
    prompts_dir: PathBuf,
    default_prompt_file: PathBuf,
    metadata_file: PathBuf,
}

impl FileStorage {
    /// Create a new file storage instance
    pub fn new() -> Result<Self> {
        let prompts_dir = Self::default_prompts_dir()?;
        Self::with_directory(prompts_dir)
    }
    
    /// Create a file storage instance with custom directory
    pub fn with_directory<P: AsRef<Path>>(dir: P) -> Result<Self> {
        let prompts_dir = dir.as_ref().to_path_buf();
        let default_prompt_file = prompts_dir.join("default.txt");
        let metadata_file = prompts_dir.join("metadata.json");
        
        // Create prompts directory if it doesn't exist
        if !prompts_dir.exists() {
            fs::create_dir_all(&prompts_dir)
                .map_err(|e| PromptError::Storage(format!("Failed to create prompts directory: {}", e)))?;
            info!("Created prompts directory: {}", prompts_dir.display());
        }
        
        let storage = Self {
            prompts_dir,
            default_prompt_file,
            metadata_file,
        };
        
        // Initialize default prompt if it doesn't exist
        if !storage.default_prompt_file.exists() {
            storage.save_default(&crate::PromptManager::factory_default_prompt())?;
        }
        
        Ok(storage)
    }
    
    /// Get the default prompts directory
    fn default_prompts_dir() -> Result<PathBuf> {
        let home_dir = home::home_dir()
            .ok_or_else(|| PromptError::Storage("Could not determine home directory".to_string()))?;
        
        #[cfg(target_os = "macos")]
        let config_dir = home_dir.join("Library").join("Application Support");
        
        #[cfg(target_os = "linux")]
        let config_dir = std::env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home_dir.join(".config"));
        
        #[cfg(target_os = "windows")]
        let config_dir = std::env::var("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home_dir.join("AppData").join("Roaming"));
        
        Ok(config_dir.join("gamecode").join("prompts"))
    }
    
    /// Get the file path for a named prompt
    fn prompt_file_path(&self, name: &str) -> PathBuf {
        self.prompts_dir.join(format!("{}.txt", Self::sanitize_name(name)))
    }
    
    /// Sanitize a prompt name for use as a filename
    fn sanitize_name(name: &str) -> String {
        name.chars()
            .map(|c| match c {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => c,
                _ => '_',
            })
            .collect()
    }
    
    /// Load metadata from file
    fn load_metadata(&self) -> Result<PromptMetadata> {
        if !self.metadata_file.exists() {
            return Ok(PromptMetadata::default());
        }
        
        let metadata_content = fs::read_to_string(&self.metadata_file)
            .map_err(|e| PromptError::Storage(format!("Failed to read metadata: {}", e)))?;
        
        let metadata: PromptMetadata = serde_json::from_str(&metadata_content)?;
        Ok(metadata)
    }
    
    /// Save metadata to file
    fn save_metadata(&self, metadata: &PromptMetadata) -> Result<()> {
        let metadata_json = serde_json::to_string_pretty(metadata)?;
        fs::write(&self.metadata_file, metadata_json)
            .map_err(|e| PromptError::Storage(format!("Failed to write metadata: {}", e)))?;
        Ok(())
    }
    
    /// Update metadata for a prompt
    fn update_prompt_metadata(&self, name: &str, file_path: &Path) -> Result<()> {
        let mut metadata = self.load_metadata()?;
        
        let file_metadata = fs::metadata(file_path)
            .map_err(|e| PromptError::Storage(format!("Failed to read file metadata: {}", e)))?;
        
        let entry = PromptEntry {
            name: name.to_string(),
            file_name: file_path.file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string(),
            created_at: file_metadata.created().unwrap_or_else(|_| SystemTime::now()),
            modified_at: file_metadata.modified().unwrap_or_else(|_| SystemTime::now()),
            size: file_metadata.len(),
        };
        
        metadata.prompts.insert(name.to_string(), entry);
        self.save_metadata(&metadata)?;
        Ok(())
    }
    
    /// Remove prompt from metadata
    fn remove_prompt_metadata(&self, name: &str) -> Result<()> {
        let mut metadata = self.load_metadata()?;
        metadata.prompts.remove(name);
        self.save_metadata(&metadata)?;
        Ok(())
    }
}

impl PromptStorage for FileStorage {
    fn load_default(&self) -> Result<String> {
        if !self.default_prompt_file.exists() {
            debug!("Default prompt file not found, returning factory default");
            return Ok(crate::PromptManager::factory_default_prompt());
        }
        
        let prompt = fs::read_to_string(&self.default_prompt_file)
            .map_err(|e| PromptError::Storage(format!("Failed to read default prompt: {}", e)))?;
        
        debug!("Loaded default prompt from {}", self.default_prompt_file.display());
        Ok(prompt.trim().to_string())
    }
    
    fn save_default(&self, prompt: &str) -> Result<()> {
        fs::write(&self.default_prompt_file, prompt.trim())
            .map_err(|e| PromptError::Storage(format!("Failed to write default prompt: {}", e)))?;
        
        info!("Saved default prompt to {}", self.default_prompt_file.display());
        Ok(())
    }
    
    fn load_prompt(&self, name: &str) -> Result<String> {
        let file_path = self.prompt_file_path(name);
        
        if !file_path.exists() {
            return Err(PromptError::PromptNotFound(name.to_string()));
        }
        
        let prompt = fs::read_to_string(&file_path)
            .map_err(|e| PromptError::Storage(format!("Failed to read prompt '{}': {}", name, e)))?;
        
        debug!("Loaded prompt '{}' from {}", name, file_path.display());
        Ok(prompt.trim().to_string())
    }
    
    fn save_prompt(&self, name: &str, prompt: &str) -> Result<()> {
        let file_path = self.prompt_file_path(name);
        
        fs::write(&file_path, prompt.trim())
            .map_err(|e| PromptError::Storage(format!("Failed to write prompt '{}': {}", name, e)))?;
        
        // Update metadata
        self.update_prompt_metadata(name, &file_path)?;
        
        info!("Saved prompt '{}' to {}", name, file_path.display());
        Ok(())
    }
    
    fn list_prompts(&self) -> Result<Vec<String>> {
        let metadata = self.load_metadata()?;
        let mut prompts: Vec<String> = metadata.prompts.keys().cloned().collect();
        prompts.sort();
        
        debug!("Listed {} prompts", prompts.len());
        Ok(prompts)
    }
    
    fn delete_prompt(&self, name: &str) -> Result<()> {
        let file_path = self.prompt_file_path(name);
        
        if !file_path.exists() {
            return Err(PromptError::PromptNotFound(name.to_string()));
        }
        
        fs::remove_file(&file_path)
            .map_err(|e| PromptError::Storage(format!("Failed to delete prompt '{}': {}", name, e)))?;
        
        // Remove from metadata
        self.remove_prompt_metadata(name)?;
        
        info!("Deleted prompt '{}'", name);
        Ok(())
    }
    
    fn prompt_exists(&self, name: &str) -> bool {
        self.prompt_file_path(name).exists()
    }
    
    fn get_prompt_info(&self, name: &str) -> Result<PromptInfo> {
        let metadata = self.load_metadata()?;
        
        if let Some(entry) = metadata.prompts.get(name) {
            Ok(PromptInfo {
                name: entry.name.clone(),
                size: entry.size,
                created_at: entry.created_at,
                modified_at: entry.modified_at,
                file_path: self.prompt_file_path(name),
            })
        } else {
            Err(PromptError::PromptNotFound(name.to_string()))
        }
    }
}

impl Default for FileStorage {
    fn default() -> Self {
        Self::new().expect("Failed to create default file storage")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_file_storage_basic_operations() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FileStorage::with_directory(temp_dir.path()).unwrap();
        
        // Test default prompt
        let default_prompt = storage.load_default().unwrap();
        assert!(!default_prompt.is_empty());
        
        // Save a custom default
        let custom_default = "Custom default prompt";
        storage.save_default(custom_default).unwrap();
        let loaded_default = storage.load_default().unwrap();
        assert_eq!(loaded_default, custom_default);
        
        // Test named prompts
        let prompt_name = "test_prompt";
        let prompt_content = "This is a test prompt";
        
        // Save prompt
        storage.save_prompt(prompt_name, prompt_content).unwrap();
        
        // Load prompt
        let loaded_prompt = storage.load_prompt(prompt_name).unwrap();
        assert_eq!(loaded_prompt, prompt_content);
        
        // Check existence
        assert!(storage.prompt_exists(prompt_name));
        assert!(!storage.prompt_exists("nonexistent"));
        
        // List prompts
        let prompts = storage.list_prompts().unwrap();
        assert_eq!(prompts.len(), 1);
        assert_eq!(prompts[0], prompt_name);
        
        // Get prompt info
        let info = storage.get_prompt_info(prompt_name).unwrap();
        assert_eq!(info.name, prompt_name);
        assert_eq!(info.size, prompt_content.len() as u64);
        
        // Delete prompt
        storage.delete_prompt(prompt_name).unwrap();
        assert!(!storage.prompt_exists(prompt_name));
        let prompts = storage.list_prompts().unwrap();
        assert_eq!(prompts.len(), 0);
    }
    
    #[test]
    fn test_name_sanitization() {
        assert_eq!(FileStorage::sanitize_name("valid-name_123"), "valid-name_123");
        assert_eq!(FileStorage::sanitize_name("invalid/name:with*chars"), "invalid_name_with_chars");
        let result = FileStorage::sanitize_name("спеціальні символи");
        assert!(result.chars().all(|c| c == '_'));
        assert_eq!(result.len(), "спеціальні символи".chars().count());
    }
}