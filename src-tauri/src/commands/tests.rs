// Integration tests for Tauri commands

use super::*;
use crate::models::{RenameOperation, TextPosition};

// Unit tests for internal functions

#[test]
fn test_validate_config_internal_valid() {
    let config = RenameConfig {
        prefix: "TEST".to_string(),
        starting_number: 1,
        additional_text: None,
        operations: vec![],
    
        ..Default::default()
    };
    
    let result = validate_config_internal(&config);
    assert!(result.valid);
    assert!(result.errors.is_empty());
}

#[test]
fn test_validate_config_internal_invalid_prefix() {
    let config = RenameConfig {
        prefix: "TEST<>".to_string(), // Invalid characters on Windows
        starting_number: 1,
        additional_text: None,
        operations: vec![],
    
        ..Default::default()
    };
    
    let result = validate_config_internal(&config);
    
    #[cfg(target_os = "windows")]
    {
        assert!(!result.valid);
        assert!(!result.errors.is_empty());
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        // On Unix, <> are valid
        assert!(result.valid);
    }
}

#[test]
fn test_validate_config_internal_invalid_starting_number() {
    let config = RenameConfig {
        prefix: "TEST".to_string(),
        starting_number: 10000, // Too high
        additional_text: None,
        operations: vec![],
    
        ..Default::default()
    };
    
    let result = validate_config_internal(&config);
    assert!(!result.valid);
    assert!(!result.errors.is_empty());
    assert!(result.errors[0].contains("starting number"));
}

#[test]
fn test_validate_config_internal_invalid_additional_text() {
    let config = RenameConfig {
        prefix: "TEST".to_string(),
        starting_number: 1,
        additional_text: Some("text/with/slashes".to_string()), // Invalid on all platforms
        operations: vec![],
    
        ..Default::default()
    };
    
    let result = validate_config_internal(&config);
    assert!(!result.valid);
    assert!(!result.errors.is_empty());
    assert!(result.errors[0].contains("additional text"));
}

#[test]
fn test_validate_config_internal_multiple_errors() {
    let config = RenameConfig {
        prefix: "TEST/".to_string(), // Invalid
        starting_number: 0, // Invalid
        additional_text: None,
        operations: vec![],
    
        ..Default::default()
    };
    
    let result = validate_config_internal(&config);
    assert!(!result.valid);
    assert_eq!(result.errors.len(), 2);
}

#[test]
fn test_validate_config_internal_with_operations() {
    let config = RenameConfig {
        prefix: "TEST".to_string(),
        starting_number: 1,
        additional_text: None,
        operations: vec![
            RenameOperation::FindReplace {
                find: "old".to_string(),
                replace: "new".to_string(),
                use_regex: false,
            },
            RenameOperation::AddText {
                text: "prefix".to_string(),
                position: TextPosition::Before,
            },
        ],
    
        ..Default::default()
    };
    
    let result = validate_config_internal(&config);
    assert!(result.valid);
    assert!(result.errors.is_empty());
}

#[test]
fn test_progress_event_serialization() {
    let event = ProgressEvent {
        current: 5,
        total: 10,
        file_name: "test.txt".to_string(),
    };
    
    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("\"current\":5"));
    assert!(json.contains("\"total\":10"));
    assert!(json.contains("\"file_name\":\"test.txt\""));
}

#[test]
fn test_validation_result_serialization() {
    let result = ValidationResult {
        valid: false,
        errors: vec!["Error 1".to_string(), "Error 2".to_string()],
        warnings: vec!["Warning 1".to_string()],
    };
    
    let json = serde_json::to_string(&result).unwrap();
    let deserialized: ValidationResult = serde_json::from_str(&json).unwrap();
    
    assert!(!deserialized.valid);
    assert_eq!(deserialized.errors.len(), 2);
    assert_eq!(deserialized.warnings.len(), 1);
}

// Integration tests for command handlers

#[cfg(test)]
mod integration_tests {
    use crate::commands::*;
    use crate::models::{FileMetadata, RenameConfig};
    use std::fs;
    use tempfile::TempDir;

    fn create_test_file_in_dir(dir: &std::path::Path, name: &str, content: &str) -> String {
        let file_path = dir.join(name);
        fs::write(&file_path, content).unwrap();
        file_path.to_string_lossy().to_string()
    }

    #[tokio::test]
    async fn test_get_file_info_command() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = create_test_file_in_dir(temp_dir.path(), "test.txt", "content");

        let result = get_file_info(vec![file_path.clone()]).await;
        assert!(result.is_ok());

        let files = result.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].name, "test.txt");
        assert_eq!(files[0].extension, Some("txt".to_string()));
    }

    #[tokio::test]
    async fn test_get_file_info_command_nonexistent() {
        let result = get_file_info(vec!["/nonexistent/file.txt".to_string()]).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to get file info"));
    }

    #[tokio::test]
    async fn test_generate_preview_command() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = create_test_file_in_dir(temp_dir.path(), "test.txt", "content");

        let file = FileMetadata {
            path: file_path.clone(),
            name: "test.txt".to_string(),
            extension: Some("txt".to_string()),
            size: 7,
        };

        let config = RenameConfig {
            prefix: "TEST".to_string(),
            starting_number: 1,
            additional_text: None,
            operations: vec![],
        
            ..Default::default()
        };

        let result = generate_preview(vec![file], config).await;
        assert!(result.is_ok());

        let previews = result.unwrap();
        assert_eq!(previews.len(), 1);
        assert_eq!(previews[0].new_name, "TEST_0001_test.txt");
    }

    #[tokio::test]
    async fn test_generate_preview_command_invalid_config() {
        let file = FileMetadata {
            path: "/test/file.txt".to_string(),
            name: "file.txt".to_string(),
            extension: Some("txt".to_string()),
            size: 0,
        };

        let config = RenameConfig {
            prefix: "TEST".to_string(),
            starting_number: 10000, // Invalid
            additional_text: None,
            operations: vec![],
        
            ..Default::default()
        };

        let result = generate_preview(vec![file], config).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid configuration"));
    }

    #[tokio::test]
    async fn test_validate_config_command_valid() {
        let config = RenameConfig {
            prefix: "TEST".to_string(),
            starting_number: 1,
            additional_text: None,
            operations: vec![],
        
            ..Default::default()
        };

        let result = validate_config(config).await;
        assert!(result.is_ok());

        let validation = result.unwrap();
        assert!(validation.valid);
        assert!(validation.errors.is_empty());
    }

    #[tokio::test]
    async fn test_validate_config_command_invalid() {
        let config = RenameConfig {
            prefix: "TEST/".to_string(), // Invalid
            starting_number: 0, // Invalid
            additional_text: None,
            operations: vec![],
        
            ..Default::default()
        };

        let result = validate_config(config).await;
        assert!(result.is_ok());

        let validation = result.unwrap();
        assert!(!validation.valid);
        assert_eq!(validation.errors.len(), 2);
    }

    #[tokio::test]
    async fn test_validate_config_command_with_additional_text() {
        let config = RenameConfig {
            prefix: "TEST".to_string(),
            starting_number: 1,
            additional_text: Some("Scene1".to_string()),
            operations: vec![],
        
            ..Default::default()
        };

        let result = validate_config(config).await;
        assert!(result.is_ok());

        let validation = result.unwrap();
        assert!(validation.valid);
        assert!(validation.errors.is_empty());
    }

    #[tokio::test]
    async fn test_validate_config_command_invalid_additional_text() {
        let config = RenameConfig {
            prefix: "TEST".to_string(),
            starting_number: 1,
            additional_text: Some("Scene/1".to_string()), // Invalid
            operations: vec![],
        
            ..Default::default()
        };

        let result = validate_config(config).await;
        assert!(result.is_ok());

        let validation = result.unwrap();
        assert!(!validation.valid);
        assert!(!validation.errors.is_empty());
        assert!(validation.errors[0].contains("additional text"));
    }
}
