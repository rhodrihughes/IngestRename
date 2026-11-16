// Preview generation for rename operations
//
// Validates: Requirements 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 5.1, 12.1-12.10

use std::path::Path;
use chrono::Local;
use regex::Regex;

use crate::models::{
    FileMetadata, PreviewResult, PreviewStatus, RenameConfig, RenameOperation,
    FormatType, TextPosition, CaseType, NumberPosition, AdditionalTextPosition,
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
            .map(|(index, file)| self.generate_single_preview(file, index))
            .collect()
    }

    /// Generate the preview for a single file at a given position in the batch.
    ///
    /// `index` is the zero-based position of the file within the batch and
    /// drives all incrementing values (leading number, trailing counter and
    /// format operations). This is used by both preview generation and the
    /// batch renamer so previews and executed renames always match.
    pub fn preview_for_index(&self, file: &FileMetadata, index: usize) -> PreviewResult {
        self.generate_single_preview(file, index)
    }

    /// Generate preview for a single file
    fn generate_single_preview(&self, file: &FileMetadata, index: usize) -> PreviewResult {
        // Construct the target filename
        let new_name = self.construct_target_name(&file.name, index);

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
    /// Format (Start position): [prefix]_[NNNN]_[additional_text]_[original_filename]
    /// Format (End position):   [prefix]_[NNNN]_[additional_text]_[original_filename]_[MM]
    /// Where additional_text is omitted if not provided.
    ///
    /// In `Start` position the leading `NNNN` increments per file. In `End`
    /// position the leading `NNNN` stays fixed at `starting_number` and the
    /// trailing `MM` counter increments per file instead.
    ///
    /// # Arguments
    /// * `original_name` - Original filename (including extension)
    /// * `index` - Zero-based position of this file within the batch
    ///
    /// # Returns
    /// * Target filename string
    fn construct_target_name(&self, original_name: &str, index: usize) -> String {
        // Apply operations to the original filename first
        let processed_name = self.apply_operations(original_name, index);

        // Without a leading number there is nothing to fix or append, so End mode
        // behaves like Start.
        let position = if self.config.use_number {
            self.config.number_position
        } else {
            NumberPosition::Start
        };

        // The leading number is fixed in End mode, incrementing in Start mode.
        let leading_num = match position {
            NumberPosition::Start => self.config.starting_number + index as u32,
            NumberPosition::End => self.config.starting_number,
        };

        // Format leading number as 4-digit zero-padded string
        let seq_str = format!("{:0>width$}", leading_num, width = self.config.number_padding as usize);

        // Build the base name from the enabled parts, joined with underscores:
        // [prefix]_[NNNN]_[additional_text]_[processed_name]
        let mut parts: Vec<String> = Vec::new();
        if self.config.use_prefix && !self.config.prefix.is_empty() {
            parts.push(self.config.prefix.clone());
        }
        let extra = if self.config.use_additional_text {
            self.config.additional_text.clone()
        } else {
            None
        };
        let extra_first = self.config.additional_text_position == AdditionalTextPosition::Before;
        if extra_first {
            parts.extend(extra.clone());
        }
        if self.config.use_number {
            parts.push(seq_str);
        }
        if !extra_first {
            parts.extend(extra);
        }
        let base = if self.config.include_original_name {
            parts.push(processed_name);
            parts.join("_")
        } else {
            // Start blank: drop the original name but keep its extension.
            let ext = original_name
                .rfind('.')
                .filter(|&d| d > 0)
                .map(|d| &original_name[d..])
                .unwrap_or("");
            format!("{}{}", parts.join("_"), ext)
        };

        // In End mode, append an incrementing counter before the extension.
        match position {
            NumberPosition::Start => base,
            NumberPosition::End => {
                let counter = self.config.end_number_start + index as u32;
                let width = self.config.end_number_padding as usize;
                let counter_str = format!("{:0>width$}", counter, width = width);
                insert_before_extension(&base, &format!("_{}", counter_str))
            }
        }
    }

    /// Apply rename operations to a filename
    ///
    /// Validates: Requirements 12.1-12.10
    ///
    /// # Arguments
    /// * `filename` - Original filename
    /// * `index` - Zero-based position of this file within the batch (for format operations)
    ///
    /// # Returns
    /// * Processed filename with all operations applied
    fn apply_operations(&self, filename: &str, index: usize) -> String {
        let mut result = filename.to_string();

        for (step, operation) in self.config.operations.iter().enumerate() {
            if self.config.disabled_steps.contains(&step) {
                continue;
            }
            result = self.apply_single_operation(&result, operation, index);
        }

        result
    }

    /// Apply a single rename operation
    fn apply_single_operation(
        &self,
        filename: &str,
        operation: &RenameOperation,
        index: usize,
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
                let formatted_text = self.format_text(format, index);
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
    ///
    /// `index` is the zero-based position of the file within the batch, so
    /// format counters increment correctly regardless of the leading number
    /// position (Start or End).
    fn format_text(&self, format: &FormatType, index: usize) -> String {
        match format {
            FormatType::Sequential { start, increment } => {
                let value = start + (index as u32 * increment);
                format!("{:04}", value)
            }
            FormatType::Counter { start, increment, padding } => {
                let value = start + (index as u32 * increment);
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

/// Insert text into a filename immediately before its extension.
///
/// If the filename has no extension the text is appended to the end.
fn insert_before_extension(filename: &str, insert: &str) -> String {
    if let Some(dot) = filename.rfind('.') {
        let (stem, ext) = filename.split_at(dot);
        format!("{}{}{}", stem, insert, ext)
    } else {
        format!("{}{}", filename, insert)
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
        
            ..Default::default()
        };

        let generator = PreviewGenerator::new(config, OperatingSystem::current());
        let result = generator.construct_target_name("video.mp4", 0);

        assert_eq!(result, "BB_0001_video.mp4");
    }

    #[test]
    fn test_construct_target_name_with_additional_text() {
        let config = RenameConfig {
            prefix: "KCOC4".to_string(),
            starting_number: 42,
            additional_text: Some("DailyRush".to_string()),
            operations: vec![],
        
            ..Default::default()
        };

        let generator = PreviewGenerator::new(config, OperatingSystem::current());
        let result = generator.construct_target_name("audio.wav", 0);

        assert_eq!(result, "KCOC4_0042_DailyRush_audio.wav");
    }

    #[test]
    fn test_construct_target_name_no_extension() {
        let config = RenameConfig {
            prefix: "DOC".to_string(),
            starting_number: 1,
            additional_text: None,
            operations: vec![],
        
            ..Default::default()
        };

        let generator = PreviewGenerator::new(config, OperatingSystem::current());
        let result = generator.construct_target_name("README", 0);

        assert_eq!(result, "DOC_0001_README");
    }

    #[test]
    fn test_construct_target_name_preserves_extension_case() {
        let config = RenameConfig::default();
        let generator = PreviewGenerator::new(config, OperatingSystem::current());

        // Test various extension cases
        assert_eq!(
            generator.construct_target_name("video.MP4", 0),
            "BB_0001_video.MP4"
        );
        assert_eq!(
            generator.construct_target_name("video.Mp4", 0),
            "BB_0001_video.Mp4"
        );
        assert_eq!(
            generator.construct_target_name("video.mp4", 0),
            "BB_0001_video.mp4"
        );
    }

    #[test]
    fn test_sequential_number_formatting() {
        let config = RenameConfig::default();
        let generator = PreviewGenerator::new(config, OperatingSystem::current());

        // Leading number = starting_number (1) + batch index, zero-padded to 4 digits
        assert_eq!(
            generator.construct_target_name("file.txt", 0),
            "BB_0001_file.txt"
        );
        assert_eq!(
            generator.construct_target_name("file.txt", 98),
            "BB_0099_file.txt"
        );
        assert_eq!(
            generator.construct_target_name("file.txt", 9998),
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
        
            ..Default::default()
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
        
            ..Default::default()
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
        
            ..Default::default()
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
        
            ..Default::default()
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
        
            ..Default::default()
        };

        let generator = PreviewGenerator::new(config, OperatingSystem::current());

        // First file (index = 0): 100 + 0*5 = 100
        assert_eq!(generator.apply_operations("file.txt", 0), "0100file.txt");

        // Second file (index = 1): 100 + 1*5 = 105
        assert_eq!(generator.apply_operations("file.txt", 1), "0105file.txt");

        // Third file (index = 2): 100 + 2*5 = 110
        assert_eq!(generator.apply_operations("file.txt", 2), "0110file.txt");
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
        
            ..Default::default()
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
        
            ..Default::default()
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
        
            ..Default::default()
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

    // ── Number position: End (fixed leading number, incrementing trailing counter) ──

    #[test]
    fn test_number_position_end_keeps_leading_number_fixed() {
        // Reproduces the requested behaviour:
        //   BFR_0001_STILL_..._01, BFR_0001_STILL_..._02
        // instead of incrementing the leading number.
        let config = RenameConfig {
            prefix: "BFR".to_string(),
            use_prefix: true,
            use_number: true,
            use_additional_text: true,
            include_original_name: true,
            disabled_steps: vec![],
            number_padding: 4,
            additional_text_position: AdditionalTextPosition::After,
            starting_number: 1,
            additional_text: Some("STILL".to_string()),
            number_position: NumberPosition::End,
            end_number_start: 1,
            end_number_padding: 2,
            operations: vec![],
        };

        let files = vec![
            create_test_file("shot.mov", "/test/shot.mov"),
            create_test_file("clip.mov", "/test/clip.mov"),
            create_test_file("take.mov", "/test/take.mov"),
        ];

        let generator = PreviewGenerator::new(config, OperatingSystem::current());
        let previews = generator.generate_preview(&files);

        // Leading number stays 0001; trailing counter increments 01, 02, 03
        assert_eq!(previews[0].new_name, "BFR_0001_STILL_shot_01.mov");
        assert_eq!(previews[1].new_name, "BFR_0001_STILL_clip_02.mov");
        assert_eq!(previews[2].new_name, "BFR_0001_STILL_take_03.mov");
    }

    #[test]
    fn test_number_position_end_without_additional_text() {
        let config = RenameConfig {
            prefix: "BFR".to_string(),
            use_prefix: true,
            use_number: true,
            use_additional_text: true,
            include_original_name: true,
            disabled_steps: vec![],
            number_padding: 4,
            additional_text_position: AdditionalTextPosition::After,
            starting_number: 7,
            additional_text: None,
            number_position: NumberPosition::End,
            end_number_start: 1,
            end_number_padding: 2,
            operations: vec![],
        };

        let generator = PreviewGenerator::new(config, OperatingSystem::current());

        // Leading number fixed at 0007; trailing counter appended before extension
        assert_eq!(generator.construct_target_name("a.txt", 0), "BFR_0007_a_01.txt");
        assert_eq!(generator.construct_target_name("a.txt", 1), "BFR_0007_a_02.txt");
    }

    #[test]
    fn test_number_position_end_no_extension_appends_counter() {
        let config = RenameConfig {
            prefix: "BFR".to_string(),
            use_prefix: true,
            use_number: true,
            use_additional_text: true,
            include_original_name: true,
            disabled_steps: vec![],
            number_padding: 4,
            additional_text_position: AdditionalTextPosition::After,
            starting_number: 1,
            additional_text: None,
            number_position: NumberPosition::End,
            end_number_start: 1,
            end_number_padding: 2,
            operations: vec![],
        };

        let generator = PreviewGenerator::new(config, OperatingSystem::current());
        assert_eq!(generator.construct_target_name("README", 4), "BFR_0001_README_05");
    }

    #[test]
    fn test_number_position_end_custom_start_and_padding() {
        let config = RenameConfig {
            prefix: "BFR".to_string(),
            use_prefix: true,
            use_number: true,
            use_additional_text: true,
            include_original_name: true,
            disabled_steps: vec![],
            number_padding: 4,
            additional_text_position: AdditionalTextPosition::After,
            starting_number: 1,
            additional_text: None,
            number_position: NumberPosition::End,
            end_number_start: 10,
            end_number_padding: 4,
            operations: vec![],
        };

        let generator = PreviewGenerator::new(config, OperatingSystem::current());
        assert_eq!(generator.construct_target_name("a.txt", 0), "BFR_0001_a_0010.txt");
        assert_eq!(generator.construct_target_name("a.txt", 2), "BFR_0001_a_0012.txt");
    }

    #[test]
    fn test_no_prefix() {
        let config = RenameConfig {
            prefix: "BB".to_string(),
            use_prefix: false,
            ..Default::default()
        };
        let g = PreviewGenerator::new(config, OperatingSystem::current());
        assert_eq!(g.construct_target_name("video.mp4", 0), "0001_video.mp4");
    }

    #[test]
    fn test_blank_original_name_keeps_extension() {
        let config = RenameConfig {
            prefix: "BB".to_string(),
            include_original_name: false,
            additional_text: Some("Scene".to_string()),
            ..Default::default()
        };
        let g = PreviewGenerator::new(config, OperatingSystem::current());
        assert_eq!(g.construct_target_name("video.mp4", 1), "BB_0002_Scene.mp4");
    }

    #[test]
    fn test_no_number() {
        let config = RenameConfig {
            prefix: "BB".to_string(),
            use_number: false,
            ..Default::default()
        };
        let g = PreviewGenerator::new(config, OperatingSystem::current());
        assert_eq!(g.construct_target_name("video.mp4", 0), "BB_video.mp4");
    }

    #[test]
    fn test_additional_text_disabled() {
        let config = RenameConfig {
            prefix: "BB".to_string(),
            additional_text: Some("Scene".to_string()),
            use_additional_text: false,
            ..Default::default()
        };
        let g = PreviewGenerator::new(config, OperatingSystem::current());
        assert_eq!(g.construct_target_name("video.mp4", 0), "BB_0001_video.mp4");
    }

    #[test]
    fn test_end_mode_without_number_appends_no_counter() {
        let config = RenameConfig {
            prefix: "BB".to_string(),
            use_number: false,
            number_position: NumberPosition::End,
            ..Default::default()
        };
        let g = PreviewGenerator::new(config, OperatingSystem::current());
        assert_eq!(g.construct_target_name("video.mp4", 3), "BB_video.mp4");
    }

    #[test]
    fn test_disabled_step_is_skipped() {
        let config = RenameConfig {
            prefix: "BB".to_string(),
            operations: vec![
                RenameOperation::ChangeCase { case: CaseType::Uppercase },
                RenameOperation::RemoveText { text: "VIDEO".to_string() },
            ],
            disabled_steps: vec![0],
            ..Default::default()
        };
        let g = PreviewGenerator::new(config, OperatingSystem::current());
        // Step 0 (uppercase) is off, so "video" is not matched by step 1 ("VIDEO").
        assert_eq!(g.construct_target_name("video.mp4", 0), "BB_0001_video.mp4");
    }

    #[test]
    fn test_fixed_number_with_counter_at_end() {
        // BFR_001_PICS_003, BFR_001_PICS_004
        let config = RenameConfig {
            prefix: "BFR".to_string(),
            starting_number: 1,
            number_padding: 3,
            number_position: NumberPosition::End,
            end_number_start: 3,
            end_number_padding: 3,
            include_original_name: false,
            additional_text: Some("PICS".to_string()),
            ..Default::default()
        };
        let g = PreviewGenerator::new(config, OperatingSystem::current());
        assert_eq!(g.construct_target_name("a.jpg", 0), "BFR_001_PICS_003.jpg");
        assert_eq!(g.construct_target_name("b.jpg", 1), "BFR_001_PICS_004.jpg");
    }

    #[test]
    fn test_incrementing_number_at_start_custom_padding() {
        // BFR_001_PORTRAIT, BFR_002_PORTRAIT
        let config = RenameConfig {
            prefix: "BFR".to_string(),
            number_padding: 3,
            include_original_name: false,
            additional_text: Some("PORTRAIT".to_string()),
            ..Default::default()
        };
        let g = PreviewGenerator::new(config, OperatingSystem::current());
        assert_eq!(g.construct_target_name("a.jpg", 0), "BFR_001_PORTRAIT.jpg");
        assert_eq!(g.construct_target_name("b.jpg", 1), "BFR_002_PORTRAIT.jpg");
    }

    #[test]
    fn test_additional_text_before_number() {
        let config = RenameConfig {
            prefix: "BFR".to_string(),
            additional_text: Some("PICS".to_string()),
            additional_text_position: AdditionalTextPosition::Before,
            ..Default::default()
        };
        let g = PreviewGenerator::new(config, OperatingSystem::current());
        assert_eq!(g.construct_target_name("a.jpg", 0), "BFR_PICS_0001_a.jpg");
    }

    #[test]
    fn test_additional_text_before_number_with_counter_at_end() {
        let config = RenameConfig {
            prefix: "BFR".to_string(),
            additional_text: Some("PICS".to_string()),
            additional_text_position: AdditionalTextPosition::Before,
            number_position: NumberPosition::End,
            include_original_name: false,
            ..Default::default()
        };
        let g = PreviewGenerator::new(config, OperatingSystem::current());
        assert_eq!(g.construct_target_name("a.jpg", 1), "BFR_PICS_0001_02.jpg");
    }
}
