// Unit tests for file operations module

use super::*;
use std::fs::{self, File};
use std::io::Write;
use tempfile::TempDir;

// Helper function to create a test file
fn create_test_file(dir: &Path, name: &str, content: &str) -> PathBuf {
    let path = dir.join(name);
    let mut file = File::create(&path).unwrap();
    file.write_all(content.as_bytes()).unwrap();
    path
}

#[test]
fn test_get_file_info_basic() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = create_test_file(temp_dir.path(), "test.txt", "hello world");
    
    let info = get_file_info(&file_path).unwrap();
    
    assert_eq!(info.name, "test.txt");
    assert_eq!(info.extension, Some("txt".to_string()));
    assert_eq!(info.size, 11); // "hello world" is 11 bytes
    assert!(info.path.contains("test.txt"));
}

#[test]
fn test_get_file_info_no_extension() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = create_test_file(temp_dir.path(), "README", "documentation");
    
    let info = get_file_info(&file_path).unwrap();
    
    assert_eq!(info.name, "README");
    assert_eq!(info.extension, None);
    assert_eq!(info.size, 13);
}

#[test]
fn test_get_file_info_multiple_dots() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = create_test_file(temp_dir.path(), "archive.tar.gz", "compressed");
    
    let info = get_file_info(&file_path).unwrap();
    
    assert_eq!(info.name, "archive.tar.gz");
    assert_eq!(info.extension, Some("gz".to_string())); // Only last extension
    assert_eq!(info.size, 10);
}

#[test]
fn test_get_file_info_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let non_existent = temp_dir.path().join("does_not_exist.txt");
    
    let result = get_file_info(&non_existent);
    
    assert!(result.is_err());
    match result.unwrap_err() {
        FileError::NotFound(_) => (),
        _ => panic!("Expected NotFound error"),
    }
}

#[test]
fn test_get_file_info_special_characters() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = create_test_file(temp_dir.path(), "file with spaces.txt", "content");
    
    let info = get_file_info(&file_path).unwrap();
    
    assert_eq!(info.name, "file with spaces.txt");
    assert_eq!(info.extension, Some("txt".to_string()));
}

#[test]
fn test_rename_file_basic() {
    let temp_dir = TempDir::new().unwrap();
    let old_path = create_test_file(temp_dir.path(), "old.txt", "content");
    let new_path = temp_dir.path().join("new.txt");
    
    let result = rename_file(&old_path, &new_path);
    
    assert!(result.is_ok());
    assert!(!old_path.exists());
    assert!(new_path.exists());
    
    // Verify content is preserved
    let content = fs::read_to_string(&new_path).unwrap();
    assert_eq!(content, "content");
}

#[test]
fn test_rename_file_preserve_extension() {
    let temp_dir = TempDir::new().unwrap();
    let old_path = create_test_file(temp_dir.path(), "video.MP4", "video data");
    let new_path = temp_dir.path().join("renamed.MP4");
    
    let result = rename_file(&old_path, &new_path);
    
    assert!(result.is_ok());
    assert!(new_path.exists());
    
    // Verify extension case is preserved
    assert_eq!(new_path.extension().unwrap(), "MP4");
}

#[test]
fn test_rename_file_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let old_path = temp_dir.path().join("does_not_exist.txt");
    let new_path = temp_dir.path().join("new.txt");
    
    let result = rename_file(&old_path, &new_path);
    
    assert!(result.is_err());
    match result.unwrap_err() {
        FileError::NotFound(_) => (),
        _ => panic!("Expected NotFound error"),
    }
}

#[test]
fn test_rename_file_already_exists() {
    let temp_dir = TempDir::new().unwrap();
    let old_path = create_test_file(temp_dir.path(), "old.txt", "old content");
    let new_path = create_test_file(temp_dir.path(), "new.txt", "new content");
    
    let result = rename_file(&old_path, &new_path);
    
    assert!(result.is_err());
    match result.unwrap_err() {
        FileError::AlreadyExists(_) => (),
        _ => panic!("Expected AlreadyExists error"),
    }
    
    // Verify both files still exist with original content
    assert!(old_path.exists());
    assert!(new_path.exists());
    assert_eq!(fs::read_to_string(&old_path).unwrap(), "old content");
    assert_eq!(fs::read_to_string(&new_path).unwrap(), "new content");
}

#[test]
fn test_rename_file_same_directory() {
    let temp_dir = TempDir::new().unwrap();
    let old_path = create_test_file(temp_dir.path(), "file1.txt", "data");
    let new_path = temp_dir.path().join("file2.txt");
    
    let result = rename_file(&old_path, &new_path);
    
    assert!(result.is_ok());
    assert!(!old_path.exists());
    assert!(new_path.exists());
    
    // Verify file is in the same directory
    assert_eq!(old_path.parent(), new_path.parent());
}

#[test]
fn test_check_file_permissions_readable() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = create_test_file(temp_dir.path(), "test.txt", "content");
    
    let result = check_file_permissions(&file_path);
    
    assert!(result.is_ok());
}

#[test]
#[cfg(unix)]
fn test_check_file_permissions_readonly() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = create_test_file(temp_dir.path(), "readonly.txt", "content");
    
    // Make file read-only
    let mut perms = fs::metadata(&file_path).unwrap().permissions();
    perms.set_readonly(true);
    fs::set_permissions(&file_path, perms).unwrap();
    
    let result = check_file_permissions(&file_path);
    
    assert!(result.is_err());
    match result.unwrap_err() {
        FileError::PermissionDenied(_) => (),
        _ => panic!("Expected PermissionDenied error"),
    }
}

#[test]
fn test_check_path_length_ok() {
    let short_path = Path::new("/home/user/file.txt");
    let result = check_path_length(short_path);
    
    assert_eq!(result, PathLengthCheck::Ok);
    assert!(result.is_acceptable());
    assert!(!result.is_too_long());
    assert!(result.message().is_none());
}

#[test]
#[cfg(target_os = "windows")]
fn test_check_path_length_warning_windows() {
    // Create a path longer than 260 characters but less than 32767
    let long_path = format!("C:\\{}", "a\\".repeat(150));
    let path = Path::new(&long_path);
    let result = check_path_length(path);
    
    match result {
        PathLengthCheck::Warning { length, threshold } => {
            assert!(length > 260);
            assert_eq!(threshold, 260);
            assert!(result.is_acceptable());
            assert!(!result.is_too_long());
            assert!(result.message().is_some());
        }
        _ => panic!("Expected Warning result"),
    }
}

#[test]
#[cfg(target_os = "windows")]
fn test_check_path_length_too_long_windows() {
    // Create a path longer than 32767 characters
    let very_long_path = format!("C:\\{}", "a\\".repeat(20000));
    let path = Path::new(&very_long_path);
    let result = check_path_length(path);
    
    match result {
        PathLengthCheck::TooLong { length, max } => {
            assert!(length > 32767);
            assert_eq!(max, 32767);
            assert!(!result.is_acceptable());
            assert!(result.is_too_long());
            assert!(result.message().is_some());
        }
        _ => panic!("Expected TooLong result"),
    }
}

#[test]
#[cfg(target_os = "macos")]
fn test_check_path_length_too_long_macos() {
    // Create a path longer than 1024 characters
    let long_path = format!("/Users/{}", "a/".repeat(600));
    let path = Path::new(&long_path);
    let result = check_path_length(path);
    
    match result {
        PathLengthCheck::TooLong { length, max } => {
            assert!(length > 1024);
            assert_eq!(max, 1024);
            assert!(!result.is_acceptable());
            assert!(result.is_too_long());
        }
        _ => panic!("Expected TooLong result"),
    }
}

#[test]
fn test_normalize_path_relative() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = create_test_file(temp_dir.path(), "test.txt", "content");
    
    // Get relative path
    let relative = file_path.strip_prefix(std::env::current_dir().unwrap()).unwrap_or(&file_path);
    
    let normalized = normalize_path(relative).unwrap();
    
    assert!(normalized.is_absolute());
}

#[test]
fn test_normalize_path_absolute() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = create_test_file(temp_dir.path(), "test.txt", "content");
    
    let normalized = normalize_path(&file_path).unwrap();
    
    assert!(normalized.is_absolute());
}

#[test]
#[cfg(target_os = "windows")]
fn test_normalize_path_long_windows() {
    // Create a path longer than 260 characters
    let long_path = format!("C:\\{}", "a\\".repeat(150));
    let path = Path::new(&long_path);
    
    let normalized = normalize_path(path).unwrap();
    let normalized_str = normalized.to_string_lossy();
    
    // Should have \\?\ prefix for long paths
    if long_path.len() > 260 {
        assert!(normalized_str.starts_with(r"\\?\"));
    }
}

#[test]
fn test_file_error_display() {
    let err = FileError::NotFound("/path/to/file.txt".to_string());
    assert!(err.to_string().contains("File not found"));
    
    let err = FileError::PermissionDenied("/path/to/file.txt".to_string());
    assert!(err.to_string().contains("Permission denied"));
    
    let err = FileError::AlreadyExists("/path/to/file.txt".to_string());
    assert!(err.to_string().contains("File already exists"));
    
    let err = FileError::InvalidPath("/path/to/file.txt".to_string());
    assert!(err.to_string().contains("Invalid path"));
}

#[test]
fn test_path_length_check_message() {
    let ok = PathLengthCheck::Ok;
    assert!(ok.message().is_none());
    
    let warning = PathLengthCheck::Warning {
        length: 300,
        threshold: 260,
    };
    let msg = warning.message().unwrap();
    assert!(msg.contains("Warning"));
    assert!(msg.contains("300"));
    assert!(msg.contains("260"));
    
    let too_long = PathLengthCheck::TooLong {
        length: 40000,
        max: 32767,
    };
    let msg = too_long.message().unwrap();
    assert!(msg.contains("Error"));
    assert!(msg.contains("40000"));
    assert!(msg.contains("32767"));
}

// Property-based tests using proptest
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    
    // Helper to generate valid filename strings
    fn valid_filename() -> impl Strategy<Value = String> {
        // Generate strings with alphanumeric characters, spaces, hyphens, and underscores
        prop::string::string_regex("[a-zA-Z0-9_ -]{1,50}").unwrap()
    }
    
    // Helper to generate valid file extensions
    fn valid_extension() -> impl Strategy<Value = String> {
        prop::string::string_regex("[a-zA-Z0-9]{1,10}").unwrap()
    }
    
    proptest! {
        // Feature: ingestrename, Property 7: Original Filename Preservation
        #[test]
        fn test_get_file_info_preserves_filename(
            filename in valid_filename(),
            extension in valid_extension()
        ) {
            let temp_dir = TempDir::new().unwrap();
            let full_name = format!("{}.{}", filename, extension);
            let file_path = create_test_file(temp_dir.path(), &full_name, "test content");
            
            let info = get_file_info(&file_path).unwrap();
            
            // Filename should be preserved exactly
            prop_assert_eq!(&info.name, &full_name);
            prop_assert_eq!(info.extension.as_deref(), Some(extension.as_str()));
        }
        
        // Feature: ingestrename, Property 8: Extension Case Preservation
        #[test]
        fn test_extension_case_preservation(
            filename in valid_filename(),
            extension in prop::string::string_regex("[a-zA-Z]{1,5}").unwrap()
        ) {
            let temp_dir = TempDir::new().unwrap();
            
            // Test with different case variations
            for case_variant in [extension.to_lowercase(), extension.to_uppercase(), extension.clone()] {
                let full_name = format!("{}.{}", filename, case_variant);
                let file_path = create_test_file(temp_dir.path(), &full_name, "content");
                
                let info = get_file_info(&file_path).unwrap();
                
                // Extension case should be preserved exactly
                prop_assert_eq!(info.extension.as_deref(), Some(case_variant.as_str()));
            }
        }
        
        // Feature: ingestrename, Property 13: File Location Preservation
        #[test]
        fn test_rename_preserves_directory(
            old_name in valid_filename(),
            new_name in valid_filename(),
            extension in valid_extension()
        ) {
            let temp_dir = TempDir::new().unwrap();
            let old_filename = format!("{}.{}", old_name, extension);
            let new_filename = format!("{}.{}", new_name, extension);
            
            let old_path = create_test_file(temp_dir.path(), &old_filename, "data");
            let new_path = temp_dir.path().join(&new_filename);
            
            let result = rename_file(&old_path, &new_path);
            
            if result.is_ok() {
                // Directory should be the same
                prop_assert_eq!(old_path.parent(), new_path.parent());
                prop_assert!(new_path.exists());
                prop_assert!(!old_path.exists());
            }
        }
        
        // Feature: ingestrename, Property 22: Special Character Handling
        #[test]
        fn test_special_characters_in_filename(
            base_name in "[a-zA-Z0-9]{1,20}",
            has_space in prop::bool::ANY,
            has_hyphen in prop::bool::ANY,
            has_underscore in prop::bool::ANY
        ) {
            let temp_dir = TempDir::new().unwrap();
            
            let mut filename = base_name.clone();
            if has_space {
                filename.push_str(" file");
            }
            if has_hyphen {
                filename.push_str("-name");
            }
            if has_underscore {
                filename.push_str("_test");
            }
            filename.push_str(".txt");
            
            let file_path = create_test_file(temp_dir.path(), &filename, "content");
            let info = get_file_info(&file_path).unwrap();
            
            // Special characters should be preserved
            prop_assert_eq!(&info.name, &filename);
        }
    }
}
