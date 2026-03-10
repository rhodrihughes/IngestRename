// Core data models for the IngestRename application
// These models support serialization/deserialization for IPC communication

use serde::{Deserialize, Serialize};

/// Configuration for a batch rename operation
/// 
/// Validates: Requirements 1.1, 3.1.1, 3.1.2, 4.1
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameConfig {
    /// User-defined prefix prepended to all renamed files (default: "BB")
    pub prefix: String,
    
    /// Starting sequential number for the batch (range: 1-9999, default: 1)
    pub starting_number: u32,
    
    /// Optional text inserted between sequential number and original filename
    pub additional_text: Option<String>,
    
    /// List of rename operations to apply (for Requirement 12)
    pub operations: Vec<RenameOperation>,
}

impl Default for RenameConfig {
    fn default() -> Self {
        Self {
            prefix: "BB".to_string(),
            starting_number: 1,
            additional_text: None,
            operations: Vec::new(),
        }
    }
}

/// Metadata about a file to be renamed
/// 
/// Validates: Requirements 1.1, 4.1
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileMetadata {
    /// Full path to the file
    pub path: String,
    
    /// Filename without directory path
    pub name: String,
    
    /// File extension (if present)
    pub extension: Option<String>,
    
    /// File size in bytes
    pub size: u64,
}

/// Result of a preview operation showing the proposed rename
/// 
/// Validates: Requirements 5.1, 5.2
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewResult {
    /// Original full path
    pub original_path: String,
    
    /// Original filename
    pub original_name: String,
    
    /// Proposed new filename
    pub new_name: String,
    
    /// Proposed new full path
    pub new_path: String,
    
    /// Status of the preview (valid, warning, or error)
    pub status: PreviewStatus,
    
    /// Optional message explaining warnings or errors
    pub message: Option<String>,
}

/// Status of a preview result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PreviewStatus {
    /// The rename operation is valid
    Valid,
    
    /// The rename operation has warnings (e.g., long path)
    Warning,
    
    /// The rename operation will fail (e.g., invalid characters, file exists)
    Error,
}

/// Result of a single file rename operation
/// 
/// Validates: Requirements 6.1, 7.1, 7.2, 7.3
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameResult {
    /// Original file path
    pub original_path: String,
    
    /// New file path (if successful)
    pub new_path: String,
    
    /// Whether the rename was successful
    pub success: bool,
    
    /// Error message if the rename failed
    pub error: Option<String>,
}

/// A record of a completed batch rename, used for undo
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoRecord {
    /// Human-readable label (e.g. timestamp)
    pub label: String,
    /// Each entry: (current_path, original_path) for files that were successfully renamed
    pub entries: Vec<(String, String)>,
}

/// A saved preset (named rename configuration)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Preset {
    pub name: String,
    pub config: RenameConfig,
}

/// Types of rename operations (for Requirement 12)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum RenameOperation {
    /// Find and replace text in filename (literal or regex)
    FindReplace {
        find: String,
        replace: String,
        /// Whether `find` is a regex pattern
        #[serde(default)]
        use_regex: bool,
    },
    
    /// Add text at a specific position
    AddText {
        text: String,
        position: TextPosition,
    },
    
    /// Add formatted text (sequential numbers, counters, dates)
    Format {
        format: FormatType,
        position: TextPosition,
        /// Optional separator placed between the formatted value and the filename
        separator: FormatSeparator,
    },

    /// Change the case of the filename
    ChangeCase {
        case: CaseType,
    },

    /// Remove a specific substring from the filename
    RemoveText {
        text: String,
    },
}

/// Case transformation types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum CaseType {
    Lowercase,
    Uppercase,
    TitleCase,
    CamelCase,
    SnakeCase,
}

/// Separator character for format operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FormatSeparator {
    None,
    #[serde(rename = "_")]
    Underscore,
    #[serde(rename = "-")]
    Hyphen,
}

/// Types of format operations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum FormatType {
    /// Sequential numbering with configurable start and increment
    Sequential {
        start: u32,
        increment: u32,
    },

    /// Counter with configurable start, increment, and zero-padding width
    Counter {
        start: u32,
        increment: u32,
        /// Number of digits to zero-pad to (default 4)
        #[serde(default = "default_padding")]
        padding: u32,
    },
    
    /// Date stamp with configurable format string
    Date {
        format: String,
    },
}

fn default_padding() -> u32 { 4 }

/// Position for text insertion
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TextPosition {
    /// Insert before the filename
    Before,
    
    /// Insert after the filename (before extension)
    After,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rename_config_default() {
        let config = RenameConfig::default();
        assert_eq!(config.prefix, "BB");
        assert_eq!(config.starting_number, 1);
        assert!(config.additional_text.is_none());
        assert!(config.operations.is_empty());
    }

    #[test]
    fn test_rename_config_serialization() {
        let config = RenameConfig {
            prefix: "TEST".to_string(),
            starting_number: 42,
            additional_text: Some("Scene1".to_string()),
            operations: vec![],
        };
        
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: RenameConfig = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.prefix, "TEST");
        assert_eq!(deserialized.starting_number, 42);
        assert_eq!(deserialized.additional_text, Some("Scene1".to_string()));
    }

    #[test]
    fn test_file_metadata_serialization() {
        let metadata = FileMetadata {
            path: "/test/file.txt".to_string(),
            name: "file.txt".to_string(),
            extension: Some("txt".to_string()),
            size: 1024,
        };
        
        let json = serde_json::to_string(&metadata).unwrap();
        let deserialized: FileMetadata = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.path, "/test/file.txt");
        assert_eq!(deserialized.name, "file.txt");
        assert_eq!(deserialized.extension, Some("txt".to_string()));
        assert_eq!(deserialized.size, 1024);
    }

    #[test]
    fn test_preview_result_serialization() {
        let preview = PreviewResult {
            original_path: "/test/old.txt".to_string(),
            original_name: "old.txt".to_string(),
            new_name: "BB_0001_old.txt".to_string(),
            new_path: "/test/BB_0001_old.txt".to_string(),
            status: PreviewStatus::Valid,
            message: None,
        };
        
        let json = serde_json::to_string(&preview).unwrap();
        let deserialized: PreviewResult = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.status, PreviewStatus::Valid);
        assert_eq!(deserialized.new_name, "BB_0001_old.txt");
    }

    #[test]
    fn test_rename_result_serialization() {
        let result = RenameResult {
            original_path: "/test/old.txt".to_string(),
            new_path: "/test/new.txt".to_string(),
            success: true,
            error: None,
        };
        
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: RenameResult = serde_json::from_str(&json).unwrap();
        
        assert!(deserialized.success);
        assert!(deserialized.error.is_none());
    }

    #[test]
    fn test_rename_operation_find_replace() {
        let op = RenameOperation::FindReplace {
            find: "old".to_string(),
            replace: "new".to_string(),
            use_regex: false,
        };
        
        let json = serde_json::to_string(&op).unwrap();
        let deserialized: RenameOperation = serde_json::from_str(&json).unwrap();
        
        match deserialized {
            RenameOperation::FindReplace { find, replace, .. } => {
                assert_eq!(find, "old");
                assert_eq!(replace, "new");
            }
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_rename_operation_add_text() {
        let op = RenameOperation::AddText {
            text: "prefix".to_string(),
            position: TextPosition::Before,
        };
        
        let json = serde_json::to_string(&op).unwrap();
        let deserialized: RenameOperation = serde_json::from_str(&json).unwrap();
        
        match deserialized {
            RenameOperation::AddText { text, position } => {
                assert_eq!(text, "prefix");
                assert_eq!(position, TextPosition::Before);
            }
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_format_type_sequential() {
        let format = FormatType::Sequential {
            start: 1,
            increment: 2,
        };
        
        let json = serde_json::to_string(&format).unwrap();
        let deserialized: FormatType = serde_json::from_str(&json).unwrap();
        
        match deserialized {
            FormatType::Sequential { start, increment } => {
                assert_eq!(start, 1);
                assert_eq!(increment, 2);
            }
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_format_type_date() {
        let format = FormatType::Date {
            format: "%Y-%m-%d".to_string(),
        };
        
        let json = serde_json::to_string(&format).unwrap();
        let deserialized: FormatType = serde_json::from_str(&json).unwrap();
        
        match deserialized {
            FormatType::Date { format: fmt } => {
                assert_eq!(fmt, "%Y-%m-%d");
            }
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_text_position() {
        assert_eq!(TextPosition::Before, TextPosition::Before);
        assert_eq!(TextPosition::After, TextPosition::After);
        assert_ne!(TextPosition::Before, TextPosition::After);
    }

    #[test]
    fn test_preview_status() {
        assert_eq!(PreviewStatus::Valid, PreviewStatus::Valid);
        assert_eq!(PreviewStatus::Warning, PreviewStatus::Warning);
        assert_eq!(PreviewStatus::Error, PreviewStatus::Error);
        assert_ne!(PreviewStatus::Valid, PreviewStatus::Error);
    }
}
