// Tauri command handlers for IPC communication
//
// Validates: Requirements 1.1, 5.1, 5.2, 6.1, 9.4

use std::path::Path;
use tauri::Emitter;

use crate::models::{FileMetadata, PreviewResult, RenameConfig, RenameResult};
use crate::file_ops::{get_file_info as get_file_metadata};
use crate::preview::PreviewGenerator;
use crate::renamer::BatchRenamer;
use crate::validator::{
    validate_prefix, validate_starting_number, OperatingSystem,
};

#[cfg(test)]
mod tests;

/// Result of configuration validation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Progress event payload for batch rename operations
#[derive(Debug, Clone, serde::Serialize)]
pub struct ProgressEvent {
    pub current: usize,
    pub total: usize,
    pub file_name: String,
}

/// Get file information for a list of file paths
///
/// Validates: Requirements 1.1, 1.2
///
/// # Arguments
/// * `paths` - List of file paths to get information for
///
/// # Returns
/// * `Result<Vec<FileMetadata>, String>` - File metadata or error message
#[tauri::command]
pub async fn get_file_info(paths: Vec<String>) -> Result<Vec<FileMetadata>, String> {
    let mut results = Vec::new();
    
    for path_str in paths {
        let path = Path::new(&path_str);
        match get_file_metadata(path) {
            Ok(metadata) => results.push(metadata),
            Err(e) => return Err(format!("Failed to get file info for '{}': {}", path_str, e)),
        }
    }
    
    Ok(results)
}

/// Generate preview of rename operation
///
/// Validates: Requirements 5.1, 5.2
///
/// # Arguments
/// * `files` - List of files to preview
/// * `config` - Rename configuration
///
/// # Returns
/// * `Result<Vec<PreviewResult>, String>` - Preview results or error message
#[tauri::command]
pub async fn generate_preview(
    files: Vec<FileMetadata>,
    config: RenameConfig,
) -> Result<Vec<PreviewResult>, String> {
    // Validate configuration first
    let validation = validate_config_internal(&config);
    if !validation.valid {
        return Err(format!("Invalid configuration: {}", validation.errors.join(", ")));
    }
    
    // Generate preview
    let os = OperatingSystem::current();
    let generator = PreviewGenerator::new(config, os);
    let preview = generator.generate_preview(&files);
    
    Ok(preview)
}

/// Validate rename configuration
///
/// Validates: Requirements 3.1.5, 3.1.6, 3.2, 3.3
///
/// # Arguments
/// * `config` - Rename configuration to validate
///
/// # Returns
/// * `Result<ValidationResult, String>` - Validation result
#[tauri::command]
pub async fn validate_config(config: RenameConfig) -> Result<ValidationResult, String> {
    Ok(validate_config_internal(&config))
}

/// Internal validation function
fn validate_config_internal(config: &RenameConfig) -> ValidationResult {
    let mut errors = Vec::new();
    let warnings = Vec::new();
    
    let os = OperatingSystem::current();
    
    // Validate prefix
    if let Err(e) = validate_prefix(&config.prefix, os) {
        errors.push(e.to_string());
    }
    
    // Validate starting number
    if let Err(e) = validate_starting_number(config.starting_number) {
        errors.push(e.to_string());
    }
    
    // Validate additional text if provided
    if let Some(ref additional_text) = config.additional_text {
        if let Err(e) = validate_prefix(additional_text, os) {
            errors.push(format!("Invalid additional text: {}", e));
        }
    }
    
    ValidationResult {
        valid: errors.is_empty(),
        errors,
        warnings,
    }
}

/// Execute batch rename operation with progress events
///
/// Validates: Requirements 6.1, 6.2, 6.3, 6.4, 7.1, 7.2, 7.3, 7.4, 9.4
///
/// # Arguments
/// * `files` - List of files to rename
/// * `config` - Rename configuration
/// * `window` - Tauri window handle for emitting progress events
///
/// # Returns
/// * `Result<Vec<RenameResult>, String>` - Rename results or error message
#[tauri::command]
pub async fn execute_batch_rename(
    files: Vec<FileMetadata>,
    config: RenameConfig,
    window: tauri::Window,
) -> Result<Vec<RenameResult>, String> {
    // Validate configuration first
    let validation = validate_config_internal(&config);
    if !validation.valid {
        return Err(format!("Invalid configuration: {}", validation.errors.join(", ")));
    }
    
    // Create batch renamer
    let os = OperatingSystem::current();
    let renamer = BatchRenamer::new(config, os);
    
    // Execute rename with progress tracking
    let total = files.len();
    let mut results = Vec::new();
    
    for (index, file) in files.iter().enumerate() {
        // Emit progress event
        let progress = ProgressEvent {
            current: index + 1,
            total,
            file_name: file.name.clone(),
        };
        
        // Emit event (ignore errors if frontend isn't listening)
        let _ = window.emit("rename-progress", &progress);
        
        // Rename the file
        let seq_num = renamer.config.starting_number + index as u32;
        let result = renamer.rename_single(file, seq_num);
        results.push(result);
    }
    
    Ok(results)
}
