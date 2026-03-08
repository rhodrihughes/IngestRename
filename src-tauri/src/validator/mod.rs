// Validation logic for rename configurations and file paths
//
// Validates: Requirements 3.1.5, 3.1.6, 3.2, 3.3, 11.4, 11.5

use std::path::Path;
use thiserror::Error;

/// Operating system type for platform-specific validation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperatingSystem {
    Windows,
    MacOS,
    Unix,
}

impl OperatingSystem {
    /// Detect the current operating system
    pub fn current() -> Self {
        #[cfg(target_os = "windows")]
        return OperatingSystem::Windows;
        
        #[cfg(target_os = "macos")]
        return OperatingSystem::MacOS;
        
        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        return OperatingSystem::Unix;
    }
}

/// Validation errors
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ValidationError {
    #[error("Invalid starting number: {0}. Must be between 1 and 9999")]
    InvalidStartingNumber(u32),
    
    #[error("Invalid prefix: {0}. Contains characters not allowed in filenames")]
    InvalidPrefix(String),
    
    #[error("Path too long: {0} characters (max: {1})")]
    PathTooLong(usize, usize),
    
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("IO error: {0}")]
    IoError(String),
}

/// Validate prefix contains only OS-specific valid filename characters
///
/// Validates: Requirements 3.1.5, 3.1.6
///
/// # Arguments
/// * `prefix` - The prefix string to validate
/// * `os` - The target operating system
///
/// # Returns
/// * `Ok(())` if the prefix is valid
/// * `Err(ValidationError::InvalidPrefix)` if the prefix contains invalid characters
pub fn validate_prefix(prefix: &str, os: OperatingSystem) -> Result<(), ValidationError> {
    if prefix.is_empty() {
        return Err(ValidationError::InvalidPrefix("Prefix cannot be empty".to_string()));
    }
    
    // Platform-specific invalid characters
    let invalid_chars: Vec<char> = match os {
        OperatingSystem::Windows => {
            // Windows forbidden characters: < > : " / \ | ? *
            vec!['<', '>', ':', '"', '/', '\\', '|', '?', '*', '\0']
        }
        OperatingSystem::MacOS | OperatingSystem::Unix => {
            // macOS and Unix forbidden characters: / and null
            // macOS also doesn't allow : in Finder but the filesystem allows it
            vec!['/', '\0']
        }
    };
    
    // Check for invalid characters
    for ch in prefix.chars() {
        if invalid_chars.contains(&ch) {
            return Err(ValidationError::InvalidPrefix(
                format!("Character '{}' is not allowed in filenames on {:?}", ch, os)
            ));
        }
    }
    
    // Windows-specific: check for reserved names
    if os == OperatingSystem::Windows {
        let reserved_names = [
            "CON", "PRN", "AUX", "NUL",
            "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9",
            "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
        ];
        
        let upper_prefix = prefix.to_uppercase();
        if reserved_names.contains(&upper_prefix.as_str()) {
            return Err(ValidationError::InvalidPrefix(
                format!("'{}' is a reserved name on Windows", prefix)
            ));
        }
    }
    
    Ok(())
}

/// Validate starting number is within valid range (1-9999)
///
/// Validates: Requirements 3.2, 3.3
///
/// # Arguments
/// * `num` - The starting number to validate
///
/// # Returns
/// * `Ok(())` if the number is valid
/// * `Err(ValidationError::InvalidStartingNumber)` if the number is outside the valid range
pub fn validate_starting_number(num: u32) -> Result<(), ValidationError> {
    if num < 1 || num > 9999 {
        return Err(ValidationError::InvalidStartingNumber(num));
    }
    Ok(())
}

/// Validate path length against OS-specific limits
///
/// Validates: Requirements 11.4, 11.5
///
/// # Arguments
/// * `path` - The file path to validate
/// * `os` - The target operating system
///
/// # Returns
/// * `Ok(())` if the path length is valid
/// * `Err(ValidationError::PathTooLong)` if the path exceeds OS maximum
///
/// # Notes
/// * Windows: Maximum 32,767 characters (with long path support)
/// * macOS: Maximum 1,024 characters
/// * Unix: Maximum 4,096 characters (typical)
pub fn validate_path_length(path: &Path, os: OperatingSystem) -> Result<(), ValidationError> {
    let path_str = path.to_string_lossy();
    let len = path_str.len();
    
    let max_len = match os {
        OperatingSystem::Windows => 32767,
        OperatingSystem::MacOS => 1024,
        OperatingSystem::Unix => 4096,
    };
    
    if len > max_len {
        return Err(ValidationError::PathTooLong(len, max_len));
    }
    
    Ok(())
}

/// Check if a file exists at the given path
///
/// Validates: Requirements 7.1
///
/// # Arguments
/// * `path` - The file path to check
///
/// # Returns
/// * `true` if the file exists
/// * `false` if the file does not exist
pub fn check_file_exists(path: &Path) -> bool {
    path.exists()
}

/// Check if the current process has permissions to rename a file
///
/// Validates: Requirements 7.2
///
/// # Arguments
/// * `path` - The file path to check
///
/// # Returns
/// * `Ok(())` if the file has appropriate permissions
/// * `Err(ValidationError)` if there are permission issues or the file doesn't exist
pub fn check_file_permissions(path: &Path) -> Result<(), ValidationError> {
    // First check if the file exists
    if !path.exists() {
        return Err(ValidationError::FileNotFound(
            path.to_string_lossy().to_string()
        ));
    }
    
    // Check if we can read the file metadata (basic permission check)
    match std::fs::metadata(path) {
        Ok(metadata) => {
            // Check if the file is read-only on Windows
            #[cfg(target_os = "windows")]
            {
                use std::os::windows::fs::MetadataExt;
                let attributes = metadata.file_attributes();
                // FILE_ATTRIBUTE_READONLY = 0x1
                if attributes & 0x1 != 0 {
                    return Err(ValidationError::PermissionDenied(
                        format!("File is read-only: {}", path.to_string_lossy())
                    ));
                }
            }
            
            // Check if the file is writable on Unix-like systems
            #[cfg(not(target_os = "windows"))]
            {
                use std::os::unix::fs::PermissionsExt;
                let permissions = metadata.permissions();
                let mode = permissions.mode();
                // Check if owner has write permission (0o200)
                if mode & 0o200 == 0 {
                    return Err(ValidationError::PermissionDenied(
                        format!("File is not writable: {}", path.to_string_lossy())
                    ));
                }
            }
            
            // Check if the parent directory is writable
            if let Some(parent) = path.parent() {
                match std::fs::metadata(parent) {
                    Ok(parent_metadata) => {
                        #[cfg(not(target_os = "windows"))]
                        {
                            use std::os::unix::fs::PermissionsExt;
                            let permissions = parent_metadata.permissions();
                            let mode = permissions.mode();
                            // Check if owner has write permission (0o200)
                            if mode & 0o200 == 0 {
                                return Err(ValidationError::PermissionDenied(
                                    format!("Parent directory is not writable: {}", parent.to_string_lossy())
                                ));
                            }
                        }
                        #[cfg(target_os = "windows")]
                        let _ = parent_metadata;
                    }
                    Err(e) => {
                        return Err(ValidationError::IoError(
                            format!("Cannot access parent directory: {}", e)
                        ));
                    }
                }
            }
            
            Ok(())
        }
        Err(e) => {
            Err(ValidationError::IoError(
                format!("Cannot access file metadata: {}", e)
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    // Tests for validate_prefix
    
    #[test]
    fn test_validate_prefix_valid() {
        assert!(validate_prefix("BB", OperatingSystem::Windows).is_ok());
        assert!(validate_prefix("KCOC4", OperatingSystem::Windows).is_ok());
        assert!(validate_prefix("ProjectX", OperatingSystem::Windows).is_ok());
        assert!(validate_prefix("Test-123", OperatingSystem::Windows).is_ok());
    }
    
    #[test]
    fn test_validate_prefix_empty() {
        let result = validate_prefix("", OperatingSystem::Windows);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            ValidationError::InvalidPrefix("Prefix cannot be empty".to_string())
        );
    }
    
    #[test]
    fn test_validate_prefix_windows_invalid_chars() {
        let invalid_chars = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
        for ch in invalid_chars {
            let prefix = format!("Test{}Name", ch);
            let result = validate_prefix(&prefix, OperatingSystem::Windows);
            assert!(result.is_err(), "Character '{}' should be invalid", ch);
        }
    }
    
    #[test]
    fn test_validate_prefix_unix_invalid_chars() {
        // Unix only forbids / and null
        assert!(validate_prefix("Test/Name", OperatingSystem::Unix).is_err());
        assert!(validate_prefix("Test\0Name", OperatingSystem::Unix).is_err());
        
        // These should be valid on Unix
        assert!(validate_prefix("Test:Name", OperatingSystem::Unix).is_ok());
        assert!(validate_prefix("Test*Name", OperatingSystem::Unix).is_ok());
    }
    
    #[test]
    fn test_validate_prefix_windows_reserved_names() {
        let reserved = ["CON", "PRN", "AUX", "NUL", "COM1", "LPT1"];
        for name in reserved {
            let result = validate_prefix(name, OperatingSystem::Windows);
            assert!(result.is_err(), "'{}' should be reserved", name);
            
            // Test case-insensitive
            let result = validate_prefix(&name.to_lowercase(), OperatingSystem::Windows);
            assert!(result.is_err(), "'{}' (lowercase) should be reserved", name);
        }
    }
    
    // Tests for validate_starting_number
    
    #[test]
    fn test_validate_starting_number_valid() {
        assert!(validate_starting_number(1).is_ok());
        assert!(validate_starting_number(100).is_ok());
        assert!(validate_starting_number(9999).is_ok());
    }
    
    #[test]
    fn test_validate_starting_number_too_low() {
        let result = validate_starting_number(0);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ValidationError::InvalidStartingNumber(0));
    }
    
    #[test]
    fn test_validate_starting_number_too_high() {
        let result = validate_starting_number(10000);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ValidationError::InvalidStartingNumber(10000));
    }
    
    // Tests for validate_path_length
    
    #[test]
    fn test_validate_path_length_valid() {
        let short_path = Path::new("/test/file.txt");
        assert!(validate_path_length(short_path, OperatingSystem::Windows).is_ok());
        assert!(validate_path_length(short_path, OperatingSystem::MacOS).is_ok());
    }
    
    #[test]
    fn test_validate_path_length_windows_too_long() {
        // Create a path longer than 32767 characters
        let long_path_str = "C:\\".to_string() + &"a\\".repeat(16384);
        let long_path = Path::new(&long_path_str);
        
        let result = validate_path_length(long_path, OperatingSystem::Windows);
        assert!(result.is_err());
        match result.unwrap_err() {
            ValidationError::PathTooLong(len, max) => {
                assert!(len > 32767);
                assert_eq!(max, 32767);
            }
            _ => panic!("Expected PathTooLong error"),
        }
    }
    
    #[test]
    fn test_validate_path_length_macos_too_long() {
        // Create a path longer than 1024 characters
        let long_path_str = "/".to_string() + &"a/".repeat(513);
        let long_path = Path::new(&long_path_str);
        
        let result = validate_path_length(long_path, OperatingSystem::MacOS);
        assert!(result.is_err());
        match result.unwrap_err() {
            ValidationError::PathTooLong(len, max) => {
                assert!(len > 1024);
                assert_eq!(max, 1024);
            }
            _ => panic!("Expected PathTooLong error"),
        }
    }
    
    // Tests for check_file_exists
    
    #[test]
    fn test_check_file_exists_true() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        File::create(&file_path).unwrap();
        
        assert!(check_file_exists(&file_path));
    }
    
    #[test]
    fn test_check_file_exists_false() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("nonexistent.txt");
        
        assert!(!check_file_exists(&file_path));
    }
    
    // Tests for check_file_permissions
    
    #[test]
    fn test_check_file_permissions_valid() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"test content").unwrap();
        
        let result = check_file_permissions(&file_path);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_check_file_permissions_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("nonexistent.txt");
        
        let result = check_file_permissions(&file_path);
        assert!(result.is_err());
        match result.unwrap_err() {
            ValidationError::FileNotFound(_) => {}
            _ => panic!("Expected FileNotFound error"),
        }
    }
    
    #[test]
    #[cfg(unix)]
    fn test_check_file_permissions_readonly() {
        use std::os::unix::fs::PermissionsExt;
        
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("readonly.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"test content").unwrap();
        drop(file);
        
        // Make file read-only
        let mut perms = std::fs::metadata(&file_path).unwrap().permissions();
        perms.set_mode(0o444); // Read-only for all
        std::fs::set_permissions(&file_path, perms).unwrap();
        
        let result = check_file_permissions(&file_path);
        assert!(result.is_err());
        match result.unwrap_err() {
            ValidationError::PermissionDenied(_) => {}
            _ => panic!("Expected PermissionDenied error"),
        }
    }
}
