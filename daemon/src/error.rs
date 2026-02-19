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

//! Error types for the fpgad daemon.
//!
//! This module defines the `FpgadError` enum, which represents all possible error
//! conditions that can occur during FPGA management operations. The error types are
//! designed to provide detailed context about failures, including file paths, data
//! values, and underlying system errors.
//!
//! # Error Categories
//!
//! - **I/O Errors** - File system operations (read, write, create, delete, directory listing)
//! - **State Errors** - FPGA device state validation failures
//! - **Argument Errors** - Invalid parameters or paths
//! - **Softener Errors** - Platform-specific operation failures (feature-gated)
//! - **Internal Errors** - Unexpected internal conditions
//!
//! # DBus Integration
//!
//! Errors are automatically converted to `zbus::fdo::Error` types for transmission over
//! DBus. The error message always includes the `FpgadError::<variant>:` prefix to allow
//! CLI clients to distinguish between application errors and DBus communication errors.
//!
//! # Examples
//!
//! ```rust,no_run
//! # use daemon::error::FpgadError;
//! # use std::path::Path;
//! #
//! fn read_config(path: &Path) -> Result<String, FpgadError> {
//!     // Will produce: FpgadError::IORead: An IO error occurred when reading from ...
//!     daemon::system_io::fs_read(path)
//! }
//! ```

use log::error;
use std::path::PathBuf;
use zbus::fdo;

/// Application-level errors for FPGA management operations.
///
/// This enum represents all possible error conditions in the fpgad daemon. Each variant
/// includes detailed context about the failure, such as file paths, data being processed,
/// and the underlying system error when applicable.
///
/// All errors implement `Display` and will be formatted with the `FpgadError::<variant>:`
/// prefix, making them easily identifiable in logs and error messages sent over DBus.
#[derive(Debug, thiserror::Error)]
pub enum FpgadError {
    /// Failed to read FPGA programming flags from sysfs.
    #[error("FpgadError::Flag: Failed to read flags: {0}")]
    Flag(String),

    /// Device tree overlay was not successfully applied.
    #[error("FpgadError::OverlayStatus: Overlay was not applied: {0}")]
    OverlayStatus(String),

    /// FPGA device is not in the expected state for the requested operation.
    #[error("FpgadError::FPGAState: FPGA state is not as expected: {0}")]
    FPGAState(String),

    /// Invalid argument provided to a function (e.g., invalid path, bad device handle).
    #[error("FpgadError::Argument: {0}")]
    Argument(String),

    /// Failed to read from a file system path - wrapper around std::io::Error
    #[error("FpgadError::IORead: An IO error occurred when reading from {file:?}: {e}")]
    IORead { file: PathBuf, e: std::io::Error },
    /// Failed to write data to a file system path.
    #[error("FpgadError::IOWrite: An IO error occurred when writing to {file:?}: {e}")]
    IOWrite { file: PathBuf, e: std::io::Error },
    /// Failed to create a file or directory - wrapper around std::io::Error
    #[error("FpgadError::IOCreate: An IO error occurred when creating {file:?}: {e}")]
    IOCreate { file: PathBuf, e: std::io::Error },

    /// Failed to delete a file or directory - wrapper around std::io::Error
    #[error("FpgadError::IODelete: An IO error occurred when deleting {file:?}: {e}")]
    IODelete { file: PathBuf, e: std::io::Error },

    /// Failed to list directory contents.
    #[error("FpgadError::IOReadDir: An IO error occurred when reading directory {dir:?}: {e}")]
    IOReadDir { dir: PathBuf, e: std::io::Error },

    /// Platform-specific softener operation failed (only available with softeners feature).
    #[cfg(feature = "softeners")]
    #[error("FpgadError::Softener: An error occurred using softener: {0}")]
    Softener(crate::softeners::error::FpgadSoftenerError),

    /// Any other unexpected internal error occurred.
    #[error("FpgadError::Internal: An Internal error occurred: {0}")]
    Internal(String),
}

/// Convert FpgadError to DBus-compatible fdo::Error.
///
/// This implementation maps application-level errors to appropriate DBus error types
/// and logs the error before conversion. The error message retains the `FpgadError::<variant>:`
/// prefix to allow clients to distinguish between different error types.
///
/// # Error Mapping
///
/// - `Argument` → `InvalidArgs` - Invalid parameters
/// - `IORead`, `IOWrite`, `IOCreate`, `IODelete`, `IOReadDir` → `IOError` - I/O failures
/// - All others → `Failed` - General failures
impl From<FpgadError> for fdo::Error {
    fn from(err: FpgadError) -> Self {
        error!("{err}");
        match err {
            FpgadError::Argument(..) => fdo::Error::InvalidArgs(err.to_string()),
            FpgadError::IORead { .. } => fdo::Error::IOError(err.to_string()),
            FpgadError::IOWrite { .. } => fdo::Error::IOError(err.to_string()),
            FpgadError::IOCreate { .. } => fdo::Error::IOError(err.to_string()),
            FpgadError::IODelete { .. } => fdo::Error::IOError(err.to_string()),
            FpgadError::IOReadDir { .. } => fdo::Error::IOError(err.to_string()),
            _ => fdo::Error::Failed(err.to_string()),
        }
    }
}
