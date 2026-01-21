//! Link utilities for resource pool management
//!
//! This module provides cross-platform utilities for creating hard links and symlinks,
//! with automatic fallback strategies.

use crate::models::resource_pool::LinkStrategy;
use std::fs;
use std::io;
use std::path::Path;

/// Check if two paths are on the same filesystem
///
/// On Unix, compares device IDs. On Windows, compares volume serial numbers.
pub fn same_filesystem(path1: &Path, path2: &Path) -> io::Result<bool> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;

        // Get the parent directory that exists for each path
        let existing1 = get_existing_parent(path1)?;
        let existing2 = get_existing_parent(path2)?;

        let meta1 = fs::metadata(&existing1)?;
        let meta2 = fs::metadata(&existing2)?;

        Ok(meta1.dev() == meta2.dev())
    }

    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::Storage::FileSystem::{GetVolumeInformationW, GetVolumePathNameW};

        fn get_volume_serial(path: &Path) -> io::Result<u32> {
            let existing = get_existing_parent(path)?;
            let path_wide: Vec<u16> = existing
                .as_os_str()
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();

            let mut volume_path = vec![0u16; 260];
            let mut serial: u32 = 0;

            unsafe {
                if GetVolumePathNameW(path_wide.as_ptr(), volume_path.as_mut_ptr(), 260) == 0 {
                    return Err(io::Error::last_os_error());
                }

                if GetVolumeInformationW(
                    volume_path.as_ptr(),
                    std::ptr::null_mut(),
                    0,
                    &mut serial,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    0,
                ) == 0
                {
                    return Err(io::Error::last_os_error());
                }
            }

            Ok(serial)
        }

        let serial1 = get_volume_serial(path1)?;
        let serial2 = get_volume_serial(path2)?;

        Ok(serial1 == serial2)
    }

    #[cfg(not(any(unix, windows)))]
    {
        // For other platforms, assume same filesystem
        Ok(true)
    }
}

/// Get the first existing parent directory of a path
fn get_existing_parent(path: &Path) -> io::Result<std::path::PathBuf> {
    let mut current = path.to_path_buf();

    while !current.exists() {
        current = current
            .parent()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "No existing parent found"))?
            .to_path_buf();
    }

    Ok(current)
}

/// Create a hard link from `original` to `link`
///
/// The `original` file must exist. The `link` path must not exist.
/// On Windows, requires the source and destination to be on the same volume.
pub fn create_hard_link(original: &Path, link: &Path) -> io::Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = link.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::hard_link(original, link)
}

/// Create a symbolic link from `link` to `original`
///
/// On Windows, this requires Developer Mode or Administrator privileges.
/// The link points TO the original file.
#[cfg(unix)]
pub fn create_symlink(original: &Path, link: &Path) -> io::Result<()> {
    use std::os::unix::fs::symlink;

    // Ensure parent directory exists
    if let Some(parent) = link.parent() {
        fs::create_dir_all(parent)?;
    }

    symlink(original, link)
}

#[cfg(windows)]
pub fn create_symlink(original: &Path, link: &Path) -> io::Result<()> {
    use std::os::windows::fs::symlink_file;

    // Ensure parent directory exists
    if let Some(parent) = link.parent() {
        fs::create_dir_all(parent)?;
    }

    symlink_file(original, link)
}

/// Check if symbolic links are available on this system
///
/// On Windows, symlinks require either Developer Mode or Administrator privileges.
/// This function attempts to create a test symlink to verify availability.
pub fn symlinks_available() -> bool {
    #[cfg(unix)]
    {
        // Symlinks are always available on Unix
        true
    }

    #[cfg(windows)]
    {
        use std::env;

        // Try to create a test symlink in temp directory
        let temp_dir = env::temp_dir();
        let test_file = temp_dir.join("etlauncher_symlink_test_source");
        let test_link = temp_dir.join("etlauncher_symlink_test_link");

        // Clean up any previous test files
        let _ = fs::remove_file(&test_file);
        let _ = fs::remove_file(&test_link);

        // Create test source file
        if fs::write(&test_file, b"test").is_err() {
            return false;
        }

        // Try to create symlink
        let result = create_symlink(&test_file, &test_link).is_ok();

        // Clean up
        let _ = fs::remove_file(&test_link);
        let _ = fs::remove_file(&test_file);

        result
    }

    #[cfg(not(any(unix, windows)))]
    {
        false
    }
}

/// Copy a file from `source` to `dest`
pub fn copy_file(source: &Path, dest: &Path) -> io::Result<u64> {
    // Ensure parent directory exists
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::copy(source, dest)
}

/// Check if a path is a symbolic link
pub fn is_symlink(path: &Path) -> bool {
    path.symlink_metadata()
        .map(|m| m.file_type().is_symlink())
        .unwrap_or(false)
}

/// Check if a path is a hard link (has more than one link)
#[cfg(unix)]
pub fn is_hard_link(path: &Path) -> bool {
    use std::os::unix::fs::MetadataExt;
    path.metadata().map(|m| m.nlink() > 1).unwrap_or(false)
}

#[cfg(windows)]
pub fn is_hard_link(path: &Path) -> bool {
    // On Windows, we need to use GetFileInformationByHandle
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::Storage::FileSystem::{
        GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION,
    };

    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return false,
    };

    let mut info: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
    let handle = file.as_raw_handle() as isize;

    unsafe {
        if GetFileInformationByHandle(handle, &mut info) != 0 {
            info.nNumberOfLinks > 1
        } else {
            false
        }
    }
}

#[cfg(not(any(unix, windows)))]
pub fn is_hard_link(_path: &Path) -> bool {
    false
}

/// Result of a link operation with the strategy that was used
pub struct LinkOperationResult {
    pub strategy_used: LinkStrategy,
}

/// Create a link with automatic fallback
///
/// Tries strategies in order based on the preferred strategy:
/// - Auto: hard link -> symlink -> copy
/// - HardLink: hard link only
/// - Symlink: symlink -> copy
/// - Copy: copy only
///
/// Returns the strategy that was actually used.
pub fn link_with_fallback(
    source: &Path,
    dest: &Path,
    preferred_strategy: LinkStrategy,
) -> io::Result<LinkOperationResult> {
    match preferred_strategy {
        LinkStrategy::Auto => {
            // Try hard link first (only if same filesystem)
            if same_filesystem(source, dest).unwrap_or(false)
                && create_hard_link(source, dest).is_ok()
            {
                return Ok(LinkOperationResult {
                    strategy_used: LinkStrategy::HardLink,
                });
            }

            // Try symlink
            if symlinks_available() && create_symlink(source, dest).is_ok() {
                return Ok(LinkOperationResult {
                    strategy_used: LinkStrategy::Symlink,
                });
            }

            // Fall back to copy
            copy_file(source, dest)?;
            Ok(LinkOperationResult {
                strategy_used: LinkStrategy::Copy,
            })
        }

        LinkStrategy::HardLink => {
            create_hard_link(source, dest)?;
            Ok(LinkOperationResult {
                strategy_used: LinkStrategy::HardLink,
            })
        }

        LinkStrategy::Symlink => {
            if create_symlink(source, dest).is_ok() {
                return Ok(LinkOperationResult {
                    strategy_used: LinkStrategy::Symlink,
                });
            }

            // Fall back to copy
            copy_file(source, dest)?;
            Ok(LinkOperationResult {
                strategy_used: LinkStrategy::Copy,
            })
        }

        LinkStrategy::Copy => {
            copy_file(source, dest)?;
            Ok(LinkOperationResult {
                strategy_used: LinkStrategy::Copy,
            })
        }
    }
}

/// Remove a link or file at the given path
///
/// Works for hard links, symlinks, and regular files.
pub fn remove_link(path: &Path) -> io::Result<()> {
    fs::remove_file(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_same_filesystem() {
        let temp = env::temp_dir();
        let path1 = temp.join("test1");
        let path2 = temp.join("test2");

        // Same directory should be same filesystem
        assert!(same_filesystem(&path1, &path2).unwrap_or(false));
    }

    #[test]
    fn test_hard_link() {
        let temp = env::temp_dir();
        let source = temp.join("etlauncher_test_hard_link_source");
        let link = temp.join("etlauncher_test_hard_link_dest");

        // Clean up
        let _ = fs::remove_file(&source);
        let _ = fs::remove_file(&link);

        // Create source
        fs::write(&source, b"test content").unwrap();

        // Create hard link
        let result = create_hard_link(&source, &link);
        assert!(result.is_ok());

        // Verify content
        let content = fs::read_to_string(&link).unwrap();
        assert_eq!(content, "test content");

        // Clean up
        let _ = fs::remove_file(&source);
        let _ = fs::remove_file(&link);
    }

    #[test]
    #[cfg(unix)]
    fn test_symlink() {
        let temp = env::temp_dir();
        let source = temp.join("etlauncher_test_symlink_source");
        let link = temp.join("etlauncher_test_symlink_dest");

        // Clean up
        let _ = fs::remove_file(&source);
        let _ = fs::remove_file(&link);

        // Create source
        fs::write(&source, b"test content").unwrap();

        // Create symlink
        let result = create_symlink(&source, &link);
        assert!(result.is_ok());

        // Verify it's a symlink
        assert!(is_symlink(&link));

        // Verify content
        let content = fs::read_to_string(&link).unwrap();
        assert_eq!(content, "test content");

        // Clean up
        let _ = fs::remove_file(&source);
        let _ = fs::remove_file(&link);
    }
}
