// Tauri command handlers for IPC communication
//
// Validates: Requirements 1.1, 5.1, 5.2, 6.1, 9.4

use std::path::Path;
use tauri::{Emitter, Manager};

use crate::models::{AppSettings, FileMetadata, PreviewResult, RenameConfig, RenameResult, UndoRecord, Preset};
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
    if config.use_prefix {
        if let Err(e) = validate_prefix(&config.prefix, os) {
            errors.push(e.to_string());
        }
    }
    
    // Validate starting number (only relevant when the number is used)
    if config.use_number {
        if let Err(e) = validate_starting_number(config.starting_number) {
            errors.push(e.to_string());
        }
    }

    if config.use_number && (config.number_padding == 0 || config.number_padding > 10) {
        errors.push("Number digits must be between 1 and 10".to_string());
    }

    // Validate additional text if provided and enabled
    if config.use_additional_text {
        if let Some(ref additional_text) = config.additional_text {
            if let Err(e) = validate_prefix(additional_text, os) {
                errors.push(format!("Invalid additional text: {}", e));
            }
        }
    }

    // At least one part of the name must be enabled
    let has_extra_text = config.use_additional_text
        && config.additional_text.as_deref().map_or(false, |t| !t.is_empty());
    if !config.use_prefix && !config.use_number && !has_extra_text && !config.include_original_name {
        errors.push("Pick at least one part of the name".to_string());
    }

    ValidationResult {
        valid: errors.is_empty(),
        errors,
        warnings,
    }
}

/// Result of execute_batch_rename including an undo record
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BatchRenameResponse {
    pub results: Vec<RenameResult>,
    pub undo_record: UndoRecord,
}

/// Execute batch rename operation with progress events
///
/// Validates: Requirements 6.1, 6.2, 6.3, 6.4, 7.1, 7.2, 7.3, 7.4, 9.4
#[tauri::command]
pub async fn execute_batch_rename(
    files: Vec<FileMetadata>,
    config: RenameConfig,
    window: tauri::Window,
) -> Result<BatchRenameResponse, String> {
    // Validate configuration first
    let validation = validate_config_internal(&config);
    if !validation.valid {
        return Err(format!("Invalid configuration: {}", validation.errors.join(", ")));
    }
    
    let os = OperatingSystem::current();
    let renamer = BatchRenamer::new(config, os);
    
    let total = files.len();
    let mut results = Vec::new();
    let mut undo_entries: Vec<(String, String)> = Vec::new();
    
    for (index, file) in files.iter().enumerate() {
        let progress = ProgressEvent {
            current: index + 1,
            total,
            file_name: file.name.clone(),
        };
        let _ = window.emit("rename-progress", &progress);
        
        let result = renamer.rename_single(file, index);
        if result.success {
            undo_entries.push((result.new_path.clone(), file.path.clone()));
        }
        results.push(result);
    }

    let label = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let undo_record = UndoRecord { label, entries: undo_entries };
    
    Ok(BatchRenameResponse { results, undo_record })
}

/// Undo a previous batch rename by reversing each successful rename
///
/// # Arguments
/// * `record` - The undo record returned from a previous execute_batch_rename call
///
/// # Returns
/// * `Result<Vec<RenameResult>, String>` - Results of the reverse renames
#[tauri::command]
pub async fn undo_batch_rename(record: UndoRecord) -> Result<Vec<RenameResult>, String> {
    use crate::file_ops::rename_file;
    let mut results = Vec::new();
    for (current_path, original_path) in &record.entries {
        let src = Path::new(current_path);
        let dst = Path::new(original_path);
        let success = rename_file(src, dst).is_ok();
        let error = if success {
            None
        } else {
            Some(format!("Failed to restore '{}' to '{}'", current_path, original_path))
        };
        results.push(RenameResult {
            original_path: current_path.clone(),
            new_path: original_path.clone(),
            success,
            error,
        });
    }
    Ok(results)
}

/// List saved presets from app data directory
#[tauri::command]
pub async fn list_presets(app: tauri::AppHandle) -> Result<Vec<Preset>, String> {
    let presets = load_presets(&app)?;
    Ok(presets)
}

/// Save a preset to app data directory
#[tauri::command]
pub async fn save_preset(app: tauri::AppHandle, preset: Preset) -> Result<(), String> {
    let mut presets = load_presets(&app)?;
    // Replace if name already exists
    if let Some(existing) = presets.iter_mut().find(|p| p.name == preset.name) {
        *existing = preset;
    } else {
        presets.push(preset);
    }
    write_presets(&app, &presets)
}

/// Delete a preset by name
#[tauri::command]
pub async fn delete_preset(app: tauri::AppHandle, name: String) -> Result<(), String> {
    let mut presets = load_presets(&app)?;
    presets.retain(|p| p.name != name);
    let mut settings = load_settings(&app);
    if settings.default_preset.as_deref() == Some(name.as_str()) {
        settings.default_preset = None;
        write_settings(&app, &settings)?;
    }
    write_presets(&app, &presets)
}

// ── Settings (default preset + last used config) ────────────────────────────

fn settings_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Could not resolve app data dir: {}", e))?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("Could not create app data dir: {}", e))?;
    Ok(dir.join("settings.json"))
}

/// Load settings; a missing or unreadable file yields defaults rather than an error.
fn load_settings(app: &tauri::AppHandle) -> AppSettings {
    settings_path(app)
        .ok()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|d| serde_json::from_str(&d).ok())
        .unwrap_or_default()
}

fn write_settings(app: &tauri::AppHandle, settings: &AppSettings) -> Result<(), String> {
    let path = settings_path(app)?;
    let data = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;
    std::fs::write(&path, data).map_err(|e| format!("Failed to write settings: {}", e))
}

/// Get persisted app settings
#[tauri::command]
pub async fn get_settings(app: tauri::AppHandle) -> Result<AppSettings, String> {
    Ok(load_settings(&app))
}

/// Set (or clear, with `None`) the preset to load on startup
#[tauri::command]
pub async fn set_default_preset(app: tauri::AppHandle, name: Option<String>) -> Result<(), String> {
    let mut settings = load_settings(&app);
    settings.default_preset = name;
    write_settings(&app, &settings)
}

/// Remember the configuration currently in use
#[tauri::command]
pub async fn save_last_config(app: tauri::AppHandle, config: RenameConfig) -> Result<(), String> {
    let mut settings = load_settings(&app);
    settings.last_config = Some(config);
    write_settings(&app, &settings)
}

// ── Preset helpers ────────────────────────────────────────────────────────────

fn presets_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Could not resolve app data dir: {}", e))?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("Could not create app data dir: {}", e))?;
    Ok(dir.join("presets.json"))
}

fn load_presets(app: &tauri::AppHandle) -> Result<Vec<Preset>, String> {
    let path = presets_path(app)?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let data = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read presets: {}", e))?;
    serde_json::from_str(&data).map_err(|e| format!("Failed to parse presets: {}", e))
}

fn write_presets(app: &tauri::AppHandle, presets: &[Preset]) -> Result<(), String> {
    let path = presets_path(app)?;
    let data = serde_json::to_string_pretty(presets)
        .map_err(|e| format!("Failed to serialize presets: {}", e))?;
    std::fs::write(&path, data).map_err(|e| format!("Failed to write presets: {}", e))
}
