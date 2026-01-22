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
//! # use daemon::system_io::{fs_read, fs_write};
//! # use std::path::Path;
//!
//! # fn example() -> Result<(), daemon::error::FpgadError> {
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
use std::path::Path;

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
/// # use daemon::system_io::fs_read;
/// # use std::path::Path;
///
/// # fn example() -> Result<(), daemon::error::FpgadError> {
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
/// # use daemon::system_io::fs_write;
/// # use std::path::Path;
/// #
/// # fn example() -> Result<(), daemon::error::FpgadError> {
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
/// # use daemon::system_io::fs_write_bytes;
/// # use std::path::Path;
/// #
/// # fn example() -> Result<(), daemon::error::FpgadError> {
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
/// # use daemon::system_io::fs_create_dir;
/// # use std::path::Path;
/// #
/// # fn example() -> Result<(), daemon::error::FpgadError> {
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
/// # use daemon::system_io::fs_remove_dir;
/// # use std::path::Path;
/// #
/// # fn example() -> Result<(), daemon::error::FpgadError> {
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
/// # use daemon::system_io::fs_read_dir;
/// # use std::path::Path;
///
/// # fn example() -> Result<(), daemon::error::FpgadError> {
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
