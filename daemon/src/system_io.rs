// This file is part of fpgad, an application to manage FPGA subsystem together with device-tree and kernel modules.
//
// Copyright 2025 Canonical Ltd.
//
// SPDX-License-Identifier: GPL-3.0-only
//
// fpgad is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License version 3, as published by the Free Software Foundation.
//
// fpgad is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranties of MERCHANTABILITY, SATISFACTORY QUALITY, or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License along with this program.  If not, see http://www.gnu.org/licenses/.

//! Error Wrapping File System I/O Helpers
//!
//! This module provides convenient wrappers around standard Rust file system operations,
//! with automatic conversion to `FpgadError` types. All functions include trace logging
//! for debugging and provide detailed error context including file paths and operation types.
//!
//! Includes: read, write, and directory operations.
//!
//! # Examples
//!
//! ```rust,no_run
//! # use crate::system_io::{fs_read, fs_write};
//! # use std::path::Path;
//!
//! # fn example() -> Result<(), crate::error::FpgadError> {
//! // Read a file
//! let content = fs_read(Path::new("/sys/class/fpga_manager/fpga0/state"))?;
//!
//! // Write to a file
//! fs_write(Path::new("/sys/class/fpga_manager/fpga0/flags"), false, "0")?;
//! # Ok(())
//! # }
//! ```

use crate::error::FpgadError;
use log::trace;
use std::fs::OpenOptions;
use std::fs::{create_dir_all, remove_dir};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};

/// Read the contents of a file to a String.
///
/// This is a convenient wrapper around `std::fs::File::read_to_string` that provides
/// trace logging and automatic error conversion to `FpgadError::IORead`.
///
/// # Arguments
///
/// * `file_path` - Path to the file to read
///
/// # Returns: `Result<String, FpgadError>`
/// * `Ok(String)` - The complete contents of the file
/// * `Err(FpgadError::IORead)` - If the file cannot be read (doesn't exist, permissions, etc.)
///
/// # Examples
///
/// ```rust,no_run
/// # use crate::system_io::fs_read;
/// # use std::path::Path;
///
/// # fn example() -> Result<(), crate::error::FpgadError> {
/// let state = fs_read(Path::new("/sys/class/fpga_manager/fpga0/state"))?;
/// println!("FPGA state: {}", state.trim());
/// # Ok(())
/// # }
/// ```
pub fn fs_read(file_path: &Path) -> Result<String, FpgadError> {
    trace!("Attempting to read from {file_path:?}");
    let mut buf: String = String::new();
    let result = OpenOptions::new()
        .read(true)
        .open(file_path)
        .and_then(|mut f| f.read_to_string(&mut buf));

    match result {
        Ok(_) => {
            trace!("Reading done");
            Ok(buf)
        }
        Err(e) => Err(FpgadError::IORead {
            file: file_path.into(),
            e,
        }),
    }
}

/// Write a string value to a file.
///
/// This is a convenient wrapper around file write operations that provides trace logging
/// and automatic error conversion to `FpgadError::IOWrite`.
///
/// # Arguments
///
/// * `file_path` - Path to the file to write
/// * `create` - If `true`, create the file if it doesn't exist; if `false`, file must already exist
/// * `value` - The string value to write (implements `AsRef<str>`)
///
/// # Returns: `Result<(), FpgadError>`
/// * `Ok(())` - Write succeeded
/// * `Err(FpgadError::IOWrite)` - If the write fails (permissions, file doesn't exist when create=false, etc.)
///
/// # Examples
///
/// ```rust,no_run
/// # use crate::system_io::fs_write;
/// # use std::path::Path;
/// #
/// # fn example() -> Result<(), crate::error::FpgadError> {
/// // Write to an existing file
/// fs_write(Path::new("/sys/class/fpga_manager/fpga0/flags"), false, "0")?;
///
/// // Create and write to a new file
/// fs_write(Path::new("/tmp/myfile.txt"), true, "Hello, world!")?;
/// # Ok(())
/// # }
/// ```
pub fn fs_write(file_path: &Path, create: bool, value: impl AsRef<str>) -> Result<(), FpgadError> {
    trace!(
        "Attempting to write {:?} to {:?}",
        value.as_ref(),
        file_path
    );
    let result = OpenOptions::new()
        .create(create)
        .read(false)
        .write(true)
        .open(file_path)
        .and_then(|mut f| write!(f, "{}", value.as_ref()));
    match result {
        Ok(_) => {
            trace!("Write done.");
            Ok(())
        }
        Err(e) => Err(FpgadError::IOWrite {
            file: file_path.into(),
            e,
        }),
    }
}

/// Write binary data to a file.
///
/// This is a convenient wrapper for writing raw bytes to a file, with automatic truncation
/// of existing content, trace logging, and error conversion to `FpgadError::IOWrite`.
///
/// # Arguments
///
/// * `file_path` - Path to the file to write
/// * `create` - If `true`, create the file if it doesn't exist; if `false`, file must already exist
/// * `data` - The binary data to write as a byte slice
///
/// # Returns: `Result<(), FpgadError>`
/// * `Ok(())` - Write succeeded
/// * `Err(FpgadError::IOWrite)` - If the write fails
///
/// # Examples
///
/// ```rust,no_run
/// # use crate::system_io::fs_write_bytes;
/// # use std::path::Path;
/// #
/// # fn example() -> Result<(), crate::error::FpgadError> {
/// let data = vec![0xDE, 0xAD, 0xBE, 0xEF];
/// fs_write_bytes(Path::new("/tmp/binary_file"), true, &data)?;
/// # Ok(())
/// # }
/// ```
pub fn fs_write_bytes(file_path: &Path, create: bool, data: &[u8]) -> Result<(), FpgadError> {
    // Open the file
    let result = OpenOptions::new()
        .create(create)
        .write(true)
        .truncate(true) // usually good to overwrite old content
        .open(file_path)
        .and_then(|mut f| f.write_all(data));

    match result {
        Ok(_) => {
            trace!("Write done.");
            Ok(())
        }
        Err(e) => Err(FpgadError::IOWrite {
            file: file_path.into(),
            e,
        }),
    }
}

/// Recursively create directories up to the specified path.
///
/// This is a convenient wrapper around `std::fs::create_dir_all` that provides trace
/// logging and automatic error conversion to `FpgadError::IOCreate`. It will create all
/// missing parent directories in the path.
///
/// # Arguments
///
/// * `path` - The directory path to create (including all parents)
///
/// # Returns: `Result<(), FpgadError>`
/// * `Ok(())` - Directory created (or already existed)
/// * `Err(FpgadError::IOCreate)` - If directory creation fails (permissions, etc.)
///
/// # Examples
///
/// ```rust,no_run
/// # use crate::system_io::fs_create_dir;
/// # use std::path::Path;
/// #
/// # fn example() -> Result<(), crate::error::FpgadError> {
/// // Create nested directories
/// fs_create_dir(Path::new("/sys/kernel/config/device-tree/overlays/my_overlay"))?;
/// # Ok(())
/// # }
/// ```
pub fn fs_create_dir(path: &Path) -> Result<(), FpgadError> {
    trace!("Attempting to Create '{path:?}'");
    let result = create_dir_all(path);
    match result {
        Ok(_) => {
            trace!("Directory created at {path:?}.");
            Ok(())
        }
        Err(e) => Err(FpgadError::IOCreate {
            file: path.into(),
            e,
        }),
    }
}

/// Remove an empty directory.
///
/// This is a convenient wrapper around `std::fs::remove_dir` that provides trace logging
/// and automatic error conversion to `FpgadError::IODelete`. The directory must be empty
/// for the operation to succeed. This works correctly with overlayfs directories.
///
/// # Arguments
///
/// * `path` - The directory path to remove
///
/// # Returns: `Result<(), FpgadError>`
/// * `Ok(())` - Directory removed successfully
/// * `Err(FpgadError::IODelete)` - If removal fails (not empty, doesn't exist, permissions, etc.)
///
/// # Examples
///
/// ```rust,no_run
/// # use crate::system_io::fs_remove_dir;
/// # use std::path::Path;
/// #
/// # fn example() -> Result<(), crate::error::FpgadError> {
/// // Remove an overlay directory
/// fs_remove_dir(Path::new("/sys/kernel/config/device-tree/overlays/my_overlay"))?;
/// # Ok(())
/// # }
/// ```
pub fn fs_remove_dir(path: &Path) -> Result<(), FpgadError> {
    trace!("Attempting to delete '{path:?}'");
    let result = remove_dir(path);
    match result {
        Ok(_) => {
            trace!("Deleted {path:?}");
            Ok(())
        }
        Err(e) => Err(FpgadError::IODelete {
            file: path.into(),
            e,
        }),
    }
}

/// Read the contents of a directory and return entry names.
///
/// This is a convenient wrapper around `std::fs::read_dir` that provides trace logging,
/// automatic error conversion to `FpgadError::IOReadDir`, and returns a vector of entry
/// names (not full paths). Entries that cannot be read are silently skipped.
///
/// # Arguments
///
/// * `dir` - The directory path to list
///
/// # Returns: `Result<Vec<String>, FpgadError>`
/// * `Ok(Vec<String>)` - List of entry names in the directory (files and subdirectories)
/// * `Err(FpgadError::IOReadDir)` - If the directory cannot be read (doesn't exist, permissions, etc.)
///
/// # Examples
///
/// ```rust,no_run
/// # use crate::system_io::fs_read_dir;
/// # use std::path::Path;
///
/// # fn example() -> Result<(), crate::error::FpgadError> {
/// // List all FPGA devices
/// let devices = fs_read_dir(Path::new("/sys/class/fpga_manager"))?;
/// for device in devices {
///     println!("Found device: {}", device);
/// }
/// # Ok(())
/// # }
/// ```
pub fn fs_read_dir(dir: &Path) -> Result<Vec<String>, FpgadError> {
    trace!("Attempting to read directory '{dir:?}'");
    std::fs::read_dir(dir).map_or_else(
        |e| {
            Err(FpgadError::IOReadDir {
                dir: dir.to_owned(),
                e,
            })
        },
        |iter| {
            let ret = iter
                .filter_map(Result::ok)
                .map(|entry| entry.file_name().to_string_lossy().into_owned())
                .collect();
            trace!("Dir reading done.");
            Ok(ret)
        },
    )
}

/// Helper function to find the overlap between two paths and to return a tuple of the overlap and
/// the difference. Note: the paths must both share the same root otherwise no overlap will be found
///
/// Typically, this is used to create a fw_lookup_path and a corresponding relative path which points
/// to the source file
///
/// # Arguments
///
/// * `source_path`: the full path to the target file (or containing directory?)
/// * `firmware_path`: the root common path for all files to be loaded by the FW subsystem
///
/// # Returns: `Result<(PathBuf, PathBuf), FpgadError>`
/// * `(PathBuf, PathBuf)` - A tuple of (prefix, suffix) where prefix is
///   typically used as the fw_lookup_path and the suffix is remaining relative path from that prefix
/// * `FpgadError::Argument` in case `firmware_path` is not within `source_path`, or for inputs
///   resulting in an empty suffix value
/// # Examples
///
/// ```
/// #use std::path::Path;
/// let (prefix, suffix) = make_firmware_pair(
///      Path::new("/lib/firmware/file.bin"),
///      Path::new("/lib/firmware/"),
/// )?;
/// assert_eq!(prefix, "/lib/firmware");
/// assert_eq!(suffix, "file.bin");
/// ```
pub(crate) fn make_firmware_pair(
    source_path: &Path,
    firmware_path: &Path,
) -> Result<(PathBuf, PathBuf), FpgadError> {
    // No firmware search path provided, so just try to use parent dir
    if firmware_path.as_os_str().is_empty() {
        return extract_path_and_filename(source_path);
    }
    if let Ok(suffix) = source_path.strip_prefix(firmware_path) {
        // Remove leading '/' if present
        let cleaned_suffix_path = suffix
            .components()
            .skip_while(|c| matches!(c, Component::RootDir))
            .collect::<PathBuf>();
        if cleaned_suffix_path.as_os_str().is_empty() {
            return Err(FpgadError::Argument(format!(
                "The resulting filename from stripping {firmware_path:?} from {source_path:?} \
                was empty. Cannot write empty string to fpga."
            )));
        }
        Ok((firmware_path.to_path_buf(), cleaned_suffix_path))
    } else {
        Err(FpgadError::Argument(format!(
            "Could not find {source_path:?} inside {firmware_path:?}"
        )))
    }
}

/// Splits a Path object into its parent directory and basename/filename
///
/// # Arguments
///
/// * `path`: path to be split
///
/// # Returns: `Result<(PathBuf, PathBuf), FpgadError>`
/// * `(PathBuf, PathBuf)` - Tuple of parent directory and basename/filename
/// * `FpgadError::Argument` on invalid `path` or `path` is a root directory (no parent)
/// # Examples
///
/// ```rust
/// let (parent, base) = extract_path_and_filename(Path::new("/lib/firmware/file.bin"));
/// assert_eq!(parent.to_string_lossy(), "/lib/firmware");
/// assert_eq!(base.to_string_lossy(), "file.bin");
/// ```
pub fn extract_path_and_filename(path: &Path) -> Result<(PathBuf, PathBuf), FpgadError> {
    // Extract filename
    let filename = path
        .file_name()
        .and_then(|f| f.to_str())
        .ok_or(FpgadError::Argument(format!(
            "Provided bitstream path {path:?} is not a file or a valid directory."
        )))?;

    // Extract parent directory
    let base_path = path
        .parent()
        .and_then(|p| p.to_str())
        .ok_or(FpgadError::Argument(format!(
            "Provided bitstream path {path:?} is missing a parent dir."
        )))?;

    Ok((base_path.into(), filename.into()))
}

#[cfg(test)]
mod test_make_firmware_pair {
    use crate::error::FpgadError;
    use crate::system_io::make_firmware_pair;
    use googletest::prelude::*;
    use rstest::*;
    use std::path::PathBuf;

    #[gtest]
    #[rstest]
    #[case::all_good(
        "/lib/firmware/file.bin",
        "/lib/firmware/",
        "/lib/firmware/",
        "file.bin"
    )]
    #[case::no_fw_path("/lib/firmware/file.bin", "", "/lib/firmware/", "file.bin")]
    #[case::no_fw_path_no_file("/lib/firmware/", "", "/lib/", "firmware")]
    fn should_pass(
        #[case] source: &str,
        #[case] fw_path: &str,
        #[case] exp_prefix: &str,
        #[case] exp_suffix: &str,
    ) {
        let result = make_firmware_pair(&PathBuf::from(source), &PathBuf::from(fw_path));
        assert_that!(
            result,
            ok(eq(&(PathBuf::from(exp_prefix), PathBuf::from(exp_suffix))))
        );
    }

    #[gtest]
    #[rstest]
    #[case::no_file(
        "/lib/firmware/",
        "/lib/firmware/",
        err(displays_as(contains_substring("The resulting filename from stripping")))
    )]
    #[case::not_in_dir(
        "/lib/firmware/file.bin",
        "/snap/x1/data/file.bin",
        err(displays_as(contains_substring("Could not find")))
    )]
    fn should_fail<M: for<'a> Matcher<&'a std::result::Result<(PathBuf, PathBuf), FpgadError>>>(
        #[case] source: &str,
        #[case] fw_path: &str,
        #[case] condition: M,
    ) {
        let result = make_firmware_pair(&PathBuf::from(source), &PathBuf::from(fw_path));
        assert_that!(&result, condition);
    }
}
