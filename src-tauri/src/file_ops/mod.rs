// File system operations module
// Provides core file operations for extracting metadata and renaming files
// Includes Windows long path support and proper error handling

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;

use crate::models::FileMetadata;

#[cfg(test)]
mod tests;

/// Errors that can occur during file operations
#[derive(Debug, Error)]
pub enum FileError {
    #[error("File not found: {0}")]
    NotFound(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("File already exists: {0}")]
    AlreadyExists(String),
    
    #[error("Invalid path: {0}")]
    InvalidPath(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
}

pub type FileResult<T> = Result<T, FileError>;

/// Extract file metadata from a given path
/// 
/// Validates: Requirements 1.2, 4.2, 10.2
/// 
/// # Arguments
/// * `path` - Path to the file
/// 
/// # Returns
/// * `FileResult<FileMetadata>` - File metadata or error
/// 
/// # Errors
/// * `FileError::NotFound` - File does not exist
/// * `FileError::PermissionDenied` - Cannot read file metadata
/// * `FileError::InvalidPath` - Path is invalid
pub fn get_file_info(path: &Path) -> FileResult<FileMetadata> {
    // Normalize path for Windows long path support
    let normalized_path = normalize_path(path)?;
    
    // Check if file exists
    if !normalized_path.exists() {
        return Err(FileError::NotFound(path.display().to_string()));
    }
    
    // Get file metadata
    let metadata = fs::metadata(&normalized_path).map_err(|e| {
        if e.kind() == io::ErrorKind::PermissionDenied {
            FileError::PermissionDenied(path.display().to_string())
        } else {
            FileError::IoError(e)
        }
    })?;
    
    // Extract filename
    let name = normalized_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| FileError::InvalidPath(path.display().to_string()))?
        .to_string();
    
    // Extract extension
    let extension = normalized_path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_string());
    
    Ok(FileMetadata {
        path: path.display().to_string(),
        name,
        extension,
        size: metadata.len(),
    })
}

/// Rename a file with proper error handling
/// 
/// Validates: Requirements 6.1, 6.3, 7.1, 7.2
/// 
/// # Arguments
/// * `old_path` - Current file path
/// * `new_path` - Target file path
/// 
/// # Returns
/// * `FileResult<()>` - Success or error
/// 
/// # Errors
/// * `FileError::NotFound` - Source file does not exist
/// * `FileError::AlreadyExists` - Target file already exists
/// * `FileError::PermissionDenied` - Cannot rename file due to permissions
pub fn rename_file(old_path: &Path, new_path: &Path) -> FileResult<()> {
    // Normalize paths for Windows long path support
    let normalized_old = normalize_path(old_path)?;
    let normalized_new = normalize_path(new_path)?;
    
    // Check if source file exists
    if !normalized_old.exists() {
        return Err(FileError::NotFound(old_path.display().to_string()));
    }
    
    // Check if target file already exists
    if normalized_new.exists() {
        return Err(FileError::AlreadyExists(new_path.display().to_string()));
    }
    
    // Check permissions on source file
    check_file_permissions(&normalized_old)?;
    
    // Perform the rename
    fs::rename(&normalized_old, &normalized_new).map_err(|e| {
        if e.kind() == io::ErrorKind::PermissionDenied {
            FileError::PermissionDenied(old_path.display().to_string())
        } else {
            FileError::IoError(e)
        }
    })?;
    
    Ok(())
}

/// Check if the current process has permission to modify a file
/// 
/// Validates: Requirements 7.2
/// 
/// # Arguments
/// * `path` - Path to the file
/// 
/// # Returns
/// * `FileResult<()>` - Success or permission error
pub fn check_file_permissions(path: &Path) -> FileResult<()> {
    let metadata = fs::metadata(path).map_err(|e| {
        if e.kind() == io::ErrorKind::PermissionDenied {
            FileError::PermissionDenied(path.display().to_string())
        } else {
            FileError::IoError(e)
        }
    })?;
    
    // Check if file is read-only
    if metadata.permissions().readonly() {
        return Err(FileError::PermissionDenied(format!(
            "File is read-only: {}",
            path.display()
        )));
    }
    
    Ok(())
}

/// Normalize a path for cross-platform compatibility and Windows long path support
/// 
/// On Windows, paths longer than 260 characters are prefixed with \\?\
/// to enable long path support (up to 32,767 characters).
/// 
/// Validates: Requirements 11.1, 11.2
/// 
/// # Arguments
/// * `path` - Path to normalize
/// 
/// # Returns
/// * `FileResult<PathBuf>` - Normalized path
#[cfg(target_os = "windows")]
fn normalize_path(path: &Path) -> FileResult<PathBuf> {
    // Convert to absolute path
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(FileError::IoError)?
            .join(path)
    };
    
    // Get the path as a string
    let path_str = absolute.to_str()
        .ok_or_else(|| FileError::InvalidPath(path.display().to_string()))?;
    
    // If path is longer than 260 characters and doesn't already have the \\?\ prefix,
    // add it to enable long path support
    if path_str.len() > 260 && !path_str.starts_with(r"\\?\") {
        // Convert to UNC path format for long path support
        let long_path = if path_str.starts_with(r"\\") {
            // Network path: \\server\share -> \\?\UNC\server\share
            format!(r"\\?\UNC\{}", &path_str[2..])
        } else {
            // Local path: C:\path -> \\?\C:\path
            format!(r"\\?\{}", path_str)
        };
        Ok(PathBuf::from(long_path))
    } else {
        Ok(absolute)
    }
}

/// Normalize a path for cross-platform compatibility
/// 
/// On non-Windows platforms, simply converts to absolute path
#[cfg(not(target_os = "windows"))]
fn normalize_path(path: &Path) -> FileResult<PathBuf> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        std::env::current_dir()
            .map_err(FileError::IoError)?
            .join(path)
            .canonicalize()
            .map_err(FileError::IoError)
    }
}

/// Check if a path would exceed the operating system's maximum path length
/// 
/// Validates: Requirements 11.3, 11.4, 11.5
/// 
/// # Arguments
/// * `path` - Path to check
/// 
/// # Returns
/// * `PathLengthCheck` - Result of the path length check
pub fn check_path_length(path: &Path) -> PathLengthCheck {
    let path_str = path.to_string_lossy();
    let len = path_str.len();
    
    #[cfg(target_os = "windows")]
    {
        const MAX_PATH_WINDOWS: usize = 32767;
        const WARN_PATH_WINDOWS: usize = 260;
        
        if len > MAX_PATH_WINDOWS {
            PathLengthCheck::TooLong {
                length: len,
                max: MAX_PATH_WINDOWS,
            }
        } else if len > WARN_PATH_WINDOWS {
            PathLengthCheck::Warning {
                length: len,
                threshold: WARN_PATH_WINDOWS,
            }
        } else {
            PathLengthCheck::Ok
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        const MAX_PATH_MACOS: usize = 1024;
        
        if len > MAX_PATH_MACOS {
            PathLengthCheck::TooLong {
                length: len,
                max: MAX_PATH_MACOS,
            }
        } else {
            PathLengthCheck::Ok
        }
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        // For other Unix-like systems, use a conservative limit
        const MAX_PATH_UNIX: usize = 4096;
        
        if len > MAX_PATH_UNIX {
            PathLengthCheck::TooLong {
                length: len,
                max: MAX_PATH_UNIX,
            }
        } else {
            PathLengthCheck::Ok
        }
    }
}

/// Result of a path length check
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum PathLengthCheck {
    /// Path length is acceptable
    Ok,
    
    /// Path length exceeds a warning threshold but is still valid
    Warning {
        length: usize,
        threshold: usize,
    },
    
    /// Path length exceeds the operating system maximum
    TooLong {
        length: usize,
        max: usize,
    },
}

impl PathLengthCheck {
    /// Check if the path length is acceptable (Ok or Warning)
    #[allow(dead_code)]
    pub fn is_acceptable(&self) -> bool {
        matches!(self, PathLengthCheck::Ok | PathLengthCheck::Warning { .. })
    }
    
    /// Check if the path length is too long
    #[allow(dead_code)]
    pub fn is_too_long(&self) -> bool {
        matches!(self, PathLengthCheck::TooLong { .. })
    }
    
    /// Get a user-friendly message describing the check result
    pub fn message(&self) -> Option<String> {
        match self {
            PathLengthCheck::Ok => None,
            PathLengthCheck::Warning { length, threshold } => {
                Some(format!(
                    "Warning: Path length ({} characters) exceeds {} characters. \
                    This may cause compatibility issues with older applications.",
                    length, threshold
                ))
            }
            PathLengthCheck::TooLong { length, max } => {
                Some(format!(
                    "Error: Path length ({} characters) exceeds the operating system \
                    maximum of {} characters.",
                    length, max
                ))
            }
        }
    }
}
