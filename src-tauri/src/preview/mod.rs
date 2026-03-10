// Preview generation for rename operations
//
// Validates: Requirements 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 5.1, 12.1-12.10

use std::path::Path;
use chrono::Local;
use regex::Regex;

use crate::models::{
    FileMetadata, PreviewResult, PreviewStatus, RenameConfig, RenameOperation,
    FormatType, TextPosition, CaseType,
};
use crate::file_ops::{check_path_length, PathLengthCheck};
use crate::validator::{check_file_exists, OperatingSystem};

/// Preview generator for batch rename operations
pub struct PreviewGenerator {
    config: RenameConfig,
    #[allow(dead_code)]
    os: OperatingSystem,
}

impl PreviewGenerator {
    /// Create a new preview generator with the given configuration
    ///
    /// # Arguments
    /// * `config` - Rename configuration
    /// * `os` - Target operating system
    pub fn new(config: RenameConfig, os: OperatingSystem) -> Self {
        Self { config, os }
    }

    /// Generate preview results for a batch of files
    ///
    /// Validates: Requirements 5.1
    ///
    /// # Arguments
    /// * `files` - List of files to preview
    ///
    /// # Returns
    /// * Vector of preview results showing original and target names
    pub fn generate_preview(&self, files: &[FileMetadata]) -> Vec<PreviewResult> {
        files
            .iter()
            .enumerate()
            .map(|(index, file)| {
                let seq_num = self.config.starting_number + index as u32;
                self.generate_single_preview(file, seq_num)
            })
            .collect()
    }

    /// Generate preview for a single file
    fn generate_single_preview(&self, file: &FileMetadata, seq_num: u32) -> PreviewResult {
        // Construct the target filename
        let new_name = self.construct_target_name(&file.name, seq_num);

        // Construct the full target path
        let original_path = Path::new(&file.path);
        let parent = original_path.parent().unwrap_or_else(|| Path::new(""));
        let new_path = parent.join(&new_name);

        // Check for issues
        let (status, message) = self.check_preview_status(&new_path);

        PreviewResult {
            original_path: file.path.clone(),
            original_name: file.name.clone(),
            new_name,
            new_path: new_path.to_string_lossy().to_string(),
            status,
            message,
        }
    }

    /// Construct the target filename following the naming convention
    ///
    /// Validates: Requirements 4.1, 4.2, 4.3, 4.4, 4.5, 4.6
    ///
    /// Format: [prefix]_[NNNN]_[additional_text]_[original_filename]
    /// Where additional_text is omitted if not provided
    ///
    /// # Arguments
    /// * `original_name` - Original filename (including extension)
    /// * `seq_num` - Sequential number for this file
    ///
    /// # Returns
    /// * Target filename string
    fn construct_target_name(&self, original_name: &str, seq_num: u32) -> String {
        // Apply operations to the original filename first
        let processed_name = self.apply_operations(original_name, seq_num);

        // Format sequential number as 4-digit zero-padded string
        let seq_str = format!("{:04}", seq_num);

        // Build the target name according to the naming convention
        if let Some(ref additional_text) = self.config.additional_text {
            // With additional text: [prefix]_[NNNN]_[additional_text]_[processed_name]
            format!(
                "{}_{}_{}_{}",
                self.config.prefix, seq_str, additional_text, processed_name
            )
        } else {
            // Without additional text: [prefix]_[NNNN]_[processed_name]
            format!("{}_{}_{}",  self.config.prefix, seq_str, processed_name)
        }
    }

    /// Apply rename operations to a filename
    ///
    /// Validates: Requirements 12.1-12.10
    ///
    /// # Arguments
    /// * `filename` - Original filename
    /// * `seq_num` - Sequential number (for format operations)
    ///
    /// # Returns
    /// * Processed filename with all operations applied
    fn apply_operations(&self, filename: &str, seq_num: u32) -> String {
        let mut result = filename.to_string();

        for operation in &self.config.operations {
            result = self.apply_single_operation(&result, operation, seq_num);
        }

        result
    }

    /// Apply a single rename operation
    fn apply_single_operation(
        &self,
        filename: &str,
        operation: &RenameOperation,
        seq_num: u32,
    ) -> String {
        match operation {
            RenameOperation::FindReplace { find, replace, use_regex } => {
                if *use_regex {
                    match Regex::new(find) {
                        Ok(re) => re.replace_all(filename, replace.as_str()).to_string(),
                        Err(_) => filename.to_string(), // invalid regex: leave unchanged
                    }
                } else {
                    filename.replace(find, replace)
                }
            }
            RenameOperation::AddText { text, position } => {
                self.add_text(filename, text, position)
            }
            RenameOperation::Format { format, position, separator } => {
                let formatted_text = self.format_text(format, seq_num);
                let sep = match separator {
                    crate::models::FormatSeparator::None => "",
                    crate::models::FormatSeparator::Underscore => "_",
                    crate::models::FormatSeparator::Hyphen => "-",
                };
                let text_with_sep = match position {
                    TextPosition::Before => format!("{}{}", formatted_text, sep),
                    TextPosition::After => format!("{}{}", sep, formatted_text),
                };
                self.add_text(filename, &text_with_sep, position)
            }
            RenameOperation::ChangeCase { case } => {
                apply_case(filename, case)
            }
            RenameOperation::RemoveText { text } => {
                filename.replace(text.as_str(), "")
            }
        }
    }

    /// Add text to a filename at the specified position
    fn add_text(&self, filename: &str, text: &str, position: &TextPosition) -> String {
        match position {
            TextPosition::Before => {
                // Insert text before the filename
                format!("{}{}", text, filename)
            }
            TextPosition::After => {
                // Insert text after the filename but before the extension
                if let Some(dot_pos) = filename.rfind('.') {
                    let (name, ext) = filename.split_at(dot_pos);
                    format!("{}{}{}", name, text, ext)
                } else {
                    // No extension, just append
                    format!("{}{}", filename, text)
                }
            }
        }
    }

    /// Format text according to the format type
    fn format_text(&self, format: &FormatType, seq_num: u32) -> String {
        match format {
            FormatType::Sequential { start, increment } => {
                let file_index = seq_num - self.config.starting_number;
                let value = start + (file_index * increment);
                format!("{:04}", value)
            }
            FormatType::Counter { start, increment, padding } => {
                let file_index = seq_num - self.config.starting_number;
                let value = start + (file_index * increment);
                let width = *padding as usize;
                format!("{:0>width$}", value, width = width)
            }
            FormatType::Date { format: date_format } => {
                let chrono_format = translate_date_format(date_format);
                let now = Local::now();
                now.format(&chrono_format).to_string()
            }
        }
    }

    /// Check the status of a preview (valid, warning, or error)
    fn check_preview_status(&self, new_path: &Path) -> (PreviewStatus, Option<String>) {
        // Check if target file already exists
        if check_file_exists(new_path) {
            return (
                PreviewStatus::Error,
                Some(format!(
                    "A file with the name '{}' already exists",
                    new_path.file_name().unwrap_or_default().to_string_lossy()
                )),
            );
        }

        // Check path length
        match check_path_length(new_path) {
            PathLengthCheck::Ok => (PreviewStatus::Valid, None),
            PathLengthCheck::Warning { .. } => {
                (PreviewStatus::Warning, check_path_length(new_path).message())
            }
            PathLengthCheck::TooLong { .. } => {
                (PreviewStatus::Error, check_path_length(new_path).message())
            }
        }
    }
}

/// Apply a case transformation to a filename (stem only, preserving extension)
fn apply_case(filename: &str, case: &CaseType) -> String {
    let (stem, ext) = if let Some(dot) = filename.rfind('.') {
        (&filename[..dot], &filename[dot..])
    } else {
        (filename, "")
    };

    let transformed = match case {
        CaseType::Lowercase => stem.to_lowercase(),
        CaseType::Uppercase => stem.to_uppercase(),
        CaseType::TitleCase => stem
            .split(|c: char| c == '_' || c == '-' || c == ' ')
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect::<Vec<_>>()
            .join(" "),
        CaseType::CamelCase => {
            let words: Vec<&str> = stem.split(|c: char| c == '_' || c == '-' || c == ' ').collect();
            let mut result = String::new();
            for (i, word) in words.iter().enumerate() {
                if i == 0 {
                    result.push_str(&word.to_lowercase());
                } else {
                    let mut chars = word.chars();
                    if let Some(first) = chars.next() {
                        result.push_str(&first.to_uppercase().to_string());
                        result.push_str(chars.as_str());
                    }
                }
            }
            result
        }
        CaseType::SnakeCase => stem
            .split(|c: char| c == '-' || c == ' ')
            .collect::<Vec<_>>()
            .join("_")
            .to_lowercase(),
    };

    format!("{}{}", transformed, ext)
}

/// Translate user-friendly date format tokens to chrono strftime format.
/// Supports: YYYY → %Y, YY → %y, MM → %m, DD → %d, HH → %H, mm → %M, ss → %S
fn translate_date_format(fmt: &str) -> String {
    fmt
        .replace("YYYY", "%Y")
        .replace("YY", "%y")
        .replace("MM", "%m")
        .replace("DD", "%d")
        .replace("HH", "%H")
        .replace("mm", "%M")
        .replace("ss", "%S")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_file(name: &str, path: &str) -> FileMetadata {
        FileMetadata {
            path: path.to_string(),
            name: name.to_string(),
            extension: name.split('.').last().map(|s| s.to_string()),
            size: 1024,
        }
    }

    #[test]
    fn test_construct_target_name_basic() {
        let config = RenameConfig {
            prefix: "BB".to_string(),
            starting_number: 1,
            additional_text: None,
            operations: vec![],
        };

        let generator = PreviewGenerator::new(config, OperatingSystem::current());
        let result = generator.construct_target_name("video.mp4", 1);

        assert_eq!(result, "BB_0001_video.mp4");
    }

    #[test]
    fn test_construct_target_name_with_additional_text() {
        let config = RenameConfig {
            prefix: "KCOC4".to_string(),
            starting_number: 42,
            additional_text: Some("DailyRush".to_string()),
            operations: vec![],
        };

        let generator = PreviewGenerator::new(config, OperatingSystem::current());
        let result = generator.construct_target_name("audio.wav", 42);

        assert_eq!(result, "KCOC4_0042_DailyRush_audio.wav");
    }

    #[test]
    fn test_construct_target_name_no_extension() {
        let config = RenameConfig {
            prefix: "DOC".to_string(),
            starting_number: 1,
            additional_text: None,
            operations: vec![],
        };

        let generator = PreviewGenerator::new(config, OperatingSystem::current());
        let result = generator.construct_target_name("README", 1);

        assert_eq!(result, "DOC_0001_README");
    }

    #[test]
    fn test_construct_target_name_preserves_extension_case() {
        let config = RenameConfig::default();
        let generator = PreviewGenerator::new(config, OperatingSystem::current());

        // Test various extension cases
        assert_eq!(
            generator.construct_target_name("video.MP4", 1),
            "BB_0001_video.MP4"
        );
        assert_eq!(
            generator.construct_target_name("video.Mp4", 1),
            "BB_0001_video.Mp4"
        );
        assert_eq!(
            generator.construct_target_name("video.mp4", 1),
            "BB_0001_video.mp4"
        );
    }

    #[test]
    fn test_sequential_number_formatting() {
        let config = RenameConfig::default();
        let generator = PreviewGenerator::new(config, OperatingSystem::current());

        // Test various sequential numbers
        assert_eq!(
            generator.construct_target_name("file.txt", 1),
            "BB_0001_file.txt"
        );
        assert_eq!(
            generator.construct_target_name("file.txt", 99),
            "BB_0099_file.txt"
        );
        assert_eq!(
            generator.construct_target_name("file.txt", 9999),
            "BB_9999_file.txt"
        );
    }

    #[test]
    fn test_apply_operations_find_replace() {
        let config = RenameConfig {
            prefix: "TEST".to_string(),
            starting_number: 1,
            additional_text: None,
            operations: vec![RenameOperation::FindReplace {
                find: "old".to_string(),
                replace: "new".to_string(),
                use_regex: false,
            }],
        };

        let generator = PreviewGenerator::new(config, OperatingSystem::current());
        let result = generator.apply_operations("old_file_old.txt", 1);

        assert_eq!(result, "new_file_new.txt");
    }

    #[test]
    fn test_apply_operations_add_text_before() {
        let config = RenameConfig {
            prefix: "TEST".to_string(),
            starting_number: 1,
            additional_text: None,
            operations: vec![RenameOperation::AddText {
                text: "prefix_".to_string(),
                position: TextPosition::Before,
            }],
        };

        let generator = PreviewGenerator::new(config, OperatingSystem::current());
        let result = generator.apply_operations("file.txt", 1);

        assert_eq!(result, "prefix_file.txt");
    }

    #[test]
    fn test_apply_operations_add_text_after() {
        let config = RenameConfig {
            prefix: "TEST".to_string(),
            starting_number: 1,
            additional_text: None,
            operations: vec![RenameOperation::AddText {
                text: "_suffix".to_string(),
                position: TextPosition::After,
            }],
        };

        let generator = PreviewGenerator::new(config, OperatingSystem::current());
        let result = generator.apply_operations("file.txt", 1);

        assert_eq!(result, "file_suffix.txt");
    }

    #[test]
    fn test_apply_operations_add_text_after_no_extension() {
        let config = RenameConfig {
            prefix: "TEST".to_string(),
            starting_number: 1,
            additional_text: None,
            operations: vec![RenameOperation::AddText {
                text: "_suffix".to_string(),
                position: TextPosition::After,
            }],
        };

        let generator = PreviewGenerator::new(config, OperatingSystem::current());
        let result = generator.apply_operations("README", 1);

        assert_eq!(result, "README_suffix");
    }

    #[test]
    fn test_apply_operations_format_sequential() {
        let config = RenameConfig {
            prefix: "TEST".to_string(),
            starting_number: 1,
            additional_text: None,
            operations: vec![RenameOperation::Format {
                format: FormatType::Sequential {
                    start: 100,
                    increment: 5,
                },
                position: TextPosition::Before,
                separator: crate::models::FormatSeparator::None,
            }],
        };

        let generator = PreviewGenerator::new(config, OperatingSystem::current());

        // First file (seq_num = 1): 100 + (1-1)*5 = 100
        assert_eq!(generator.apply_operations("file.txt", 1), "0100file.txt");

        // Second file (seq_num = 2): 100 + (2-1)*5 = 105
        assert_eq!(generator.apply_operations("file.txt", 2), "0105file.txt");

        // Third file (seq_num = 3): 100 + (3-1)*5 = 110
        assert_eq!(generator.apply_operations("file.txt", 3), "0110file.txt");
    }

    #[test]
    fn test_apply_operations_format_date() {
        let config = RenameConfig {
            prefix: "TEST".to_string(),
            starting_number: 1,
            additional_text: None,
            operations: vec![RenameOperation::Format {
                format: FormatType::Date {
                    format: "%Y%m%d".to_string(),
                },
                position: TextPosition::Before,
                separator: crate::models::FormatSeparator::None,
            }],
        };

        let generator = PreviewGenerator::new(config, OperatingSystem::current());
        let result = generator.apply_operations("file.txt", 1);

        // Check that the result contains a date in YYYYMMDD format
        let date_part = &result[..8];
        assert!(date_part.chars().all(|c| c.is_ascii_digit()));
        assert_eq!(date_part.len(), 8);
        assert!(result.ends_with("file.txt"));
    }

    #[test]
    fn test_apply_operations_multiple() {
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
                    text: "prefix_".to_string(),
                    position: TextPosition::Before,
                },
            ],
        };

        let generator = PreviewGenerator::new(config, OperatingSystem::current());
        let result = generator.apply_operations("old_file.txt", 1);

        // First: old_file.txt -> new_file.txt
        // Then: new_file.txt -> prefix_new_file.txt
        assert_eq!(result, "prefix_new_file.txt");
    }

    #[test]
    fn test_generate_preview_batch() {
        let config = RenameConfig {
            prefix: "TEST".to_string(),
            starting_number: 5,
            additional_text: None,
            operations: vec![],
        };

        let files = vec![
            create_test_file("file1.txt", "/test/file1.txt"),
            create_test_file("file2.txt", "/test/file2.txt"),
            create_test_file("file3.txt", "/test/file3.txt"),
        ];

        let generator = PreviewGenerator::new(config, OperatingSystem::current());
        let previews = generator.generate_preview(&files);

        assert_eq!(previews.len(), 3);
        assert_eq!(previews[0].new_name, "TEST_0005_file1.txt");
        assert_eq!(previews[1].new_name, "TEST_0006_file2.txt");
        assert_eq!(previews[2].new_name, "TEST_0007_file3.txt");
    }

    #[test]
    fn test_generate_preview_preserves_original_info() {
        let config = RenameConfig::default();
        let file = create_test_file("original.txt", "/test/original.txt");

        let generator = PreviewGenerator::new(config, OperatingSystem::current());
        let previews = generator.generate_preview(&[file]);

        assert_eq!(previews[0].original_name, "original.txt");
        assert_eq!(previews[0].original_path, "/test/original.txt");
    }
}
