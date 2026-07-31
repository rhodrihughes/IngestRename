// Batch rename execution logic
//
// Validates: Requirements 6.1, 6.2, 6.3, 7.1, 7.2, 7.3, 7.4

use std::path::Path;

use crate::models::{FileMetadata, RenameConfig, RenameResult};
use crate::file_ops::{rename_file, FileError};
use crate::preview::PreviewGenerator;
use crate::validator::OperatingSystem;

/// Batch renamer for executing file rename operations
pub struct BatchRenamer {
    pub config: RenameConfig,
    os: OperatingSystem,
}

impl BatchRenamer {
    /// Create a new batch renamer with the given configuration
    ///
    /// # Arguments
    /// * `config` - Rename configuration
    /// * `os` - Target operating system
    pub fn new(config: RenameConfig, os: OperatingSystem) -> Self {
        Self { config, os }
    }

    /// Execute batch rename operation on all files
    ///
    /// Validates: Requirements 6.1, 6.2, 6.3, 7.3
    ///
    /// This method processes files in order, preserving the selection order.
    /// If a file fails to rename, the error is recorded and processing continues
    /// with the remaining files.
    ///
    /// # Arguments
    /// * `files` - List of files to rename
    ///
    /// # Returns
    /// * Vector of rename results with success/failure information for each file
    #[allow(dead_code)]
    pub fn execute(&self, files: &[FileMetadata]) -> Vec<RenameResult> {
        files
            .iter()
            .enumerate()
            .map(|(index, file)| self.rename_single(file, index))
            .collect()
    }

    /// Rename a single file with error handling
    ///
    /// Validates: Requirements 6.1, 6.3, 7.1, 7.2
    ///
    /// This method handles errors gracefully:
    /// - If the target file already exists, returns an error result
    /// - If permission is denied, returns an error result
    /// - Other errors are also captured and returned
    ///
    /// # Arguments
    /// * `file` - File metadata for the file to rename
    /// * `index` - Zero-based position of this file within the batch
    ///
    /// # Returns
    /// * RenameResult with success status and error message if failed
    pub fn rename_single(&self, file: &FileMetadata, index: usize) -> RenameResult {
        // Use the preview generator to construct the target name.
        // This ensures consistency between preview and execution: the same
        // index drives the leading number, trailing counter and format ops.
        let generator = PreviewGenerator::new(self.config.clone(), self.os);
        let preview = generator.preview_for_index(file, index);

        let target_path = &preview.new_path;
        
        // Attempt to rename the file
        let original_path = Path::new(&file.path);
        let new_path = Path::new(target_path);
        
        match rename_file(original_path, new_path) {
            Ok(()) => RenameResult {
                original_path: file.path.clone(),
                new_path: target_path.clone(),
                success: true,
                error: None,
            },
            Err(e) => {
                // Handle different error types with appropriate messages
                let error_message = match e {
                    FileError::AlreadyExists(_) => {
                        format!(
                            "A file with the name '{}' already exists",
                            new_path.file_name().unwrap_or_default().to_string_lossy()
                        )
                    }
                    FileError::PermissionDenied(_) => {
                        format!(
                            "Permission denied. The file may be open in another program or you may not have write access"
                        )
                    }
                    FileError::NotFound(_) => {
                        format!("File not found: {}", file.path)
                    }
                    FileError::InvalidPath(_) => {
                        format!("Invalid file path: {}", file.path)
                    }
                    FileError::IoError(io_err) => {
                        format!("IO error: {}", io_err)
                    }
                };
                
                RenameResult {
                    original_path: file.path.clone(),
                    new_path: target_path.clone(),
                    success: false,
                    error: Some(error_message),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn create_test_file_in_dir(dir: &Path, name: &str, content: &str) -> FileMetadata {
        let file_path = dir.join(name);
        fs::write(&file_path, content).unwrap();
        
        let extension = file_path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_string());
        
        FileMetadata {
            path: file_path.to_string_lossy().to_string(),
            name: name.to_string(),
            extension,
            size: content.len() as u64,
        }
    }

    #[test]
    fn test_rename_single_file_success() {
        let temp_dir = TempDir::new().unwrap();
        let file = create_test_file_in_dir(temp_dir.path(), "test.txt", "content");
        
        let config = RenameConfig {
            prefix: "TEST".to_string(),
            starting_number: 1,
            additional_text: None,
            operations: vec![],
        
            ..Default::default()
        };
        
        let renamer = BatchRenamer::new(config, OperatingSystem::current());
        let result = renamer.rename_single(&file, 0);
        
        assert!(result.success);
        assert!(result.error.is_none());
        assert!(result.new_path.contains("TEST_0001_test.txt"));
        
        // Verify the file was actually renamed
        let new_path = PathBuf::from(&result.new_path);
        assert!(new_path.exists());
        assert!(!PathBuf::from(&file.path).exists());
    }

    #[test]
    fn test_rename_single_file_already_exists() {
        let temp_dir = TempDir::new().unwrap();
        let file = create_test_file_in_dir(temp_dir.path(), "test.txt", "content");
        
        // Create a file with the target name
        create_test_file_in_dir(temp_dir.path(), "TEST_0001_test.txt", "existing");
        
        let config = RenameConfig {
            prefix: "TEST".to_string(),
            starting_number: 1,
            additional_text: None,
            operations: vec![],
        
            ..Default::default()
        };
        
        let renamer = BatchRenamer::new(config, OperatingSystem::current());
        let result = renamer.rename_single(&file, 0);
        
        assert!(!result.success);
        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("already exists"));
        
        // Verify the original file still exists
        assert!(PathBuf::from(&file.path).exists());
    }

    #[test]
    fn test_rename_single_file_not_found() {
        let config = RenameConfig {
            prefix: "TEST".to_string(),
            starting_number: 1,
            additional_text: None,
            operations: vec![],
        
            ..Default::default()
        };
        
        let file = FileMetadata {
            path: "/nonexistent/file.txt".to_string(),
            name: "file.txt".to_string(),
            extension: Some("txt".to_string()),
            size: 0,
        };
        
        let renamer = BatchRenamer::new(config, OperatingSystem::current());
        let result = renamer.rename_single(&file, 0);
        
        assert!(!result.success);
        assert!(result.error.is_some());
    }

    #[test]
    fn test_execute_batch_success() {
        let temp_dir = TempDir::new().unwrap();
        let files = vec![
            create_test_file_in_dir(temp_dir.path(), "file1.txt", "content1"),
            create_test_file_in_dir(temp_dir.path(), "file2.txt", "content2"),
            create_test_file_in_dir(temp_dir.path(), "file3.txt", "content3"),
        ];
        
        let config = RenameConfig {
            prefix: "BATCH".to_string(),
            starting_number: 10,
            additional_text: None,
            operations: vec![],
        
            ..Default::default()
        };
        
        let renamer = BatchRenamer::new(config, OperatingSystem::current());
        let results = renamer.execute(&files);
        
        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| r.success), "All renames should succeed");
        
        // Check that the new paths end with the expected filenames
        assert!(results[0].new_path.ends_with("BATCH_0010_file1.txt"));
        assert!(results[1].new_path.ends_with("BATCH_0011_file2.txt"));
        assert!(results[2].new_path.ends_with("BATCH_0012_file3.txt"));
        
        // Verify all files were renamed
        for result in &results {
            assert!(PathBuf::from(&result.new_path).exists());
            assert!(!PathBuf::from(&result.original_path).exists());
        }
    }

    #[test]
    fn test_execute_batch_preserves_order() {
        let temp_dir = TempDir::new().unwrap();
        let files = vec![
            create_test_file_in_dir(temp_dir.path(), "zebra.txt", "z"),
            create_test_file_in_dir(temp_dir.path(), "alpha.txt", "a"),
            create_test_file_in_dir(temp_dir.path(), "beta.txt", "b"),
        ];
        
        let config = RenameConfig {
            prefix: "ORDER".to_string(),
            starting_number: 1,
            additional_text: None,
            operations: vec![],
        
            ..Default::default()
        };
        
        let renamer = BatchRenamer::new(config, OperatingSystem::current());
        let results = renamer.execute(&files);
        
        // Files should be numbered in the order they were provided, not alphabetically
        assert!(results[0].new_path.ends_with("ORDER_0001_zebra.txt"));
        assert!(results[1].new_path.ends_with("ORDER_0002_alpha.txt"));
        assert!(results[2].new_path.ends_with("ORDER_0003_beta.txt"));
    }

    #[test]
    fn test_execute_batch_partial_failure() {
        let temp_dir = TempDir::new().unwrap();
        let files = vec![
            create_test_file_in_dir(temp_dir.path(), "file1.txt", "content1"),
            create_test_file_in_dir(temp_dir.path(), "file2.txt", "content2"),
            create_test_file_in_dir(temp_dir.path(), "file3.txt", "content3"),
        ];
        
        // Create a file that will conflict with the second file's target name
        create_test_file_in_dir(temp_dir.path(), "FAIL_0002_file2.txt", "existing");
        
        let config = RenameConfig {
            prefix: "FAIL".to_string(),
            starting_number: 1,
            additional_text: None,
            operations: vec![],
        
            ..Default::default()
        };
        
        let renamer = BatchRenamer::new(config, OperatingSystem::current());
        let results = renamer.execute(&files);
        
        assert_eq!(results.len(), 3);
        
        // First file should succeed
        assert!(results[0].success, "First file should succeed");
        assert!(PathBuf::from(&results[0].new_path).exists());
        
        // Second file should fail (already exists)
        assert!(!results[1].success, "Second file should fail due to existing file");
        assert!(results[1].error.is_some());
        assert!(results[1].error.as_ref().unwrap().contains("already exists"));
        
        // Third file should still succeed (error continuation)
        assert!(results[2].success, "Third file should succeed despite second file failure");
        assert!(PathBuf::from(&results[2].new_path).exists());
    }

    #[test]
    fn test_execute_empty_batch() {
        let config = RenameConfig::default();
        let renamer = BatchRenamer::new(config, OperatingSystem::current());
        let results = renamer.execute(&[]);
        
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_execute_with_additional_text() {
        let temp_dir = TempDir::new().unwrap();
        let files = vec![
            create_test_file_in_dir(temp_dir.path(), "video.mp4", "video content"),
        ];
        
        let config = RenameConfig {
            prefix: "PROJ".to_string(),
            starting_number: 1,
            additional_text: Some("Scene5".to_string()),
            operations: vec![],
        
            ..Default::default()
        };
        
        let renamer = BatchRenamer::new(config, OperatingSystem::current());
        let results = renamer.execute(&files);
        
        assert_eq!(results.len(), 1);
        assert!(results[0].success);
        assert!(results[0].new_path.contains("PROJ_0001_Scene5_video.mp4"));
    }

    #[test]
    fn test_execute_preserves_extension_case() {
        let temp_dir = TempDir::new().unwrap();
        let files = vec![
            create_test_file_in_dir(temp_dir.path(), "video.MP4", "content"),
        ];
        
        let config = RenameConfig::default();
        let renamer = BatchRenamer::new(config, OperatingSystem::current());
        let results = renamer.execute(&files);
        
        assert_eq!(results.len(), 1);
        assert!(results[0].success);
        assert!(results[0].new_path.ends_with(".MP4"));
    }

    #[test]
    fn test_execute_file_without_extension() {
        let temp_dir = TempDir::new().unwrap();
        let files = vec![
            create_test_file_in_dir(temp_dir.path(), "README", "readme content"),
        ];
        
        let config = RenameConfig {
            prefix: "DOC".to_string(),
            starting_number: 1,
            additional_text: None,
            operations: vec![],
        
            ..Default::default()
        };
        
        let renamer = BatchRenamer::new(config, OperatingSystem::current());
        let results = renamer.execute(&files);
        
        assert_eq!(results.len(), 1);
        assert!(results[0].success);
        assert!(results[0].new_path.ends_with("DOC_0001_README"));
    }

    #[test]
    fn test_execute_preserves_file_location() {
        let temp_dir = TempDir::new().unwrap();
        let file = create_test_file_in_dir(temp_dir.path(), "test.txt", "content");
        
        let config = RenameConfig::default();
        let renamer = BatchRenamer::new(config, OperatingSystem::current());
        let results = renamer.execute(&[file.clone()]);
        
        assert_eq!(results.len(), 1);
        assert!(results[0].success);
        
        // Verify the file is still in the same directory
        let original_dir = PathBuf::from(&file.path).parent().unwrap().to_path_buf();
        let new_dir = PathBuf::from(&results[0].new_path).parent().unwrap().to_path_buf();
        assert_eq!(original_dir, new_dir);
    }
}
