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

/// Read metadata for a path without opening the file's contents.
///
/// Cloud-synced files (OneDrive "Files On-Demand" placeholders) are reparse
/// points. `fs::metadata` follows them and can fail, or `Path::exists` can
/// report false, for files that are online-only. `symlink_metadata` reads the
/// attributes without following the reparse point, so nothing is downloaded.
/// If that still fails (other than "not found"), the entry is read from its
/// parent directory listing, which needs no open on the file at all.
pub fn stat_path(path: &Path) -> io::Result<fs::Metadata> {
    match fs::symlink_metadata(path) {
        Ok(meta) => {
            if meta.file_type().is_symlink() {
                // A real symlink: report the target when it resolves
                if let Ok(target) = fs::metadata(path) {
                    return Ok(target);
                }
            }
            Ok(meta)
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => Err(e),
        Err(e) => {
            let (Some(parent), Some(name)) = (path.parent(), path.file_name()) else {
                return Err(e);
            };
            let entries = fs::read_dir(parent).map_err(|_| io::Error::new(e.kind(), e.to_string()))?;
            for entry in entries.flatten() {
                if entry.file_name() == name {
                    return entry.metadata();
                }
            }
            Err(e)
        }
    }
}

/// True when something exists at `path`, without downloading cloud placeholders.
pub fn path_exists(path: &Path) -> bool {
    stat_path(path).is_ok()
}

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
    
    // Get file metadata (does not download OneDrive online-only files)
    let metadata = stat_path(&normalized_path).map_err(|e| match e.kind() {
        io::ErrorKind::NotFound => FileError::NotFound(path.display().to_string()),
        io::ErrorKind::PermissionDenied => FileError::PermissionDenied(path.display().to_string()),
        _ => FileError::IoError(e),
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

/// Rename a file, refusing to replace an existing file at `new_path`
///
/// Validates: Requirements 6.1, 6.3, 7.1, 7.2
///
/// The "already exists" check is enforced by the OS during the rename itself,
/// so a file that appears at `new_path` after the check is still never replaced.
///
/// # Errors
/// * `FileError::NotFound` - Source file does not exist
/// * `FileError::AlreadyExists` - Target file already exists
/// * `FileError::PermissionDenied` - Cannot rename file due to permissions
pub fn rename_file(old_path: &Path, new_path: &Path) -> FileResult<()> {
    rename_impl(old_path, new_path, false)
}

/// Rename a file, replacing any existing file at `new_path`.
///
/// The replaced file is permanently removed. Only call this after the user
/// has agreed to overwrite that specific file.
pub fn rename_file_overwrite(old_path: &Path, new_path: &Path) -> FileResult<()> {
    rename_impl(old_path, new_path, true)
}

fn rename_impl(old_path: &Path, new_path: &Path, overwrite: bool) -> FileResult<()> {
    // Normalize paths for Windows long path support
    let normalized_old = normalize_path(old_path)?;
    let normalized_new = normalize_path(new_path)?;

    // Check if source file exists
    if !path_exists(&normalized_old) {
        return Err(FileError::NotFound(old_path.display().to_string()));
    }

    // A case-only change (or no change) targets the source file itself
    let same_file = same_entry(&normalized_old, &normalized_new);

    if !overwrite && !same_file && path_exists(&normalized_new) {
        return Err(FileError::AlreadyExists(new_path.display().to_string()));
    }

    // Check permissions on source file
    check_file_permissions(&normalized_old)?;

    let result = if overwrite || same_file {
        fs::rename(&normalized_old, &normalized_new)
    } else {
        rename_noreplace(&normalized_old, &normalized_new)
    };

    result.map_err(|e| match e.kind() {
        io::ErrorKind::AlreadyExists => FileError::AlreadyExists(new_path.display().to_string()),
        io::ErrorKind::PermissionDenied => FileError::PermissionDenied(old_path.display().to_string()),
        _ => FileError::IoError(e),
    })
}

/// Whether two paths name the same directory entry. Windows and macOS file
/// names are case-insensitive, so `a.mov` and `A.mov` are the same file there.
fn same_entry(a: &Path, b: &Path) -> bool {
    if cfg!(any(target_os = "windows", target_os = "macos")) {
        a.to_string_lossy().to_lowercase() == b.to_string_lossy().to_lowercase()
    } else {
        a == b
    }
}

/// Rename that fails with `AlreadyExists` if `new` exists, checked atomically by the OS.
#[cfg(target_os = "windows")]
fn rename_noreplace(old: &Path, new: &Path) -> io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::MoveFileExW;

    fn wide(p: &Path) -> Vec<u16> {
        p.as_os_str().encode_wide().chain(std::iter::once(0)).collect()
    }
    let (old_w, new_w) = (wide(old), wide(new));
    // Flags 0: no MOVEFILE_REPLACE_EXISTING, so an existing target is an error
    if unsafe { MoveFileExW(old_w.as_ptr(), new_w.as_ptr(), 0) } == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

/// Rename that fails with `AlreadyExists` if `new` exists, checked atomically by the OS.
#[cfg(target_os = "macos")]
fn rename_noreplace(old: &Path, new: &Path) -> io::Result<()> {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;

    let to_c = |p: &Path| {
        CString::new(p.as_os_str().as_bytes())
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "path contains a NUL byte"))
    };
    let (old_c, new_c) = (to_c(old)?, to_c(new)?);
    if unsafe { libc::renamex_np(old_c.as_ptr(), new_c.as_ptr(), libc::RENAME_EXCL) } == 0 {
        return Ok(());
    }
    let err = io::Error::last_os_error();
    // Some volumes (e.g. network shares) don't support RENAME_EXCL
    if matches!(err.raw_os_error(), Some(libc::ENOTSUP) | Some(libc::EINVAL)) {
        return rename_checked(old, new);
    }
    Err(err)
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn rename_noreplace(old: &Path, new: &Path) -> io::Result<()> {
    rename_checked(old, new)
}

/// Fallback: check, then rename. Leaves a tiny window where a new file could be replaced.
#[cfg(not(target_os = "windows"))]
fn rename_checked(old: &Path, new: &Path) -> io::Result<()> {
    if path_exists(new) {
        return Err(io::Error::from(io::ErrorKind::AlreadyExists));
    }
    fs::rename(old, new)
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
    let metadata = stat_path(path).map_err(|e| {
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
