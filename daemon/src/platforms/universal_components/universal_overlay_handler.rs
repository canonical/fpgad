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

//! Universal device tree overlay handler implementation.
//!
//! This module provides the [`UniversalOverlayHandler`] struct, which implements the
//! [`OverlayHandler`] trait for generic device tree overlay management using the Linux
//! configfs mechanism. It handles overlay application, removal, and status checking
//! without vendor-specific logic.
//!
//! # Configfs Interface
//!
//! ## Overlay Directory Structure
//!
//! Overlays are managed in `/sys/kernel/config/device-tree/overlays/<handle>/`:
//! ```text
//! /sys/kernel/config/device-tree/overlays/my_overlay/
//! ├── dtbo       # Device tree blob (appears unused in current implementation)
//! ├── path       # Write overlay source path here to apply; read to verify application
//! └── status     # Read to check if overlay was applied successfully
//!```
//!
//! # Overlay Application Flow
//!
//! 1. Create overlay directory in configfs
//! 2. Write overlay source path to the `path` file
//! 3. Verify overlay applied by checking both `path` and `status` files
//!
//! # Examples
//!
//! ```rust,no_run
//! use daemon::platforms::universal_components::universal_overlay_handler::UniversalOverlayHandler;
//! use daemon::platforms::platform::OverlayHandler;
//! use std::path::Path;
//!
//! # fn example() -> Result<(), daemon::error::FpgadError> {
//! let handler =  platform_for_known_platform("universal").overlay_handler("my_overlay")?;
//!
//! // Apply overlay
//! handler.apply_overlay(Path::new("/lib/firmware/design.dtbo"))?;
//!
//! // Check status
//! let status = handler.status()?;
//! println!("Overlay status: {}", status);
//!
//! // Remove overlay
//! handler.remove_overlay()?;
//! # Ok(())
//! # }
//! ```

use crate::config;
use crate::error::FpgadError;
use crate::platforms::platform::OverlayHandler;
use crate::system_io::{fs_create_dir, fs_read, fs_remove_dir, fs_write};
use log::{info, trace};
use std::path::{Path, PathBuf};

/// Construct the configfs path for an overlay directory.
///
/// Creates the full path to an overlay's configfs directory by combining the
/// base overlay control directory with the provided handle.
/// See [config::OVERLAY_CONTROL_DIR] for information about the base path.
///
/// # Arguments
///
/// * `overlay_handle` - The overlay handle (directory name)
///
/// # Returns: `PathBuf`
/// * Full path to overlay directory in configfs
///
/// # Examples
///
/// ```rust,no_run
/// # use std::path::PathBuf;
/// # fn construct_overlay_fs_path(overlay_handle: &str) -> PathBuf {
/// #     PathBuf::from("/sys/kernel/config/device-tree/overlays").join(overlay_handle)
/// # }
/// let path = construct_overlay_fs_path("my_overlay");
/// assert!(path.to_string_lossy().contains("my_overlay"));
/// ```
fn construct_overlay_fs_path(overlay_handle: &str) -> PathBuf {
    let overlay_fs_path = PathBuf::from(config::OVERLAY_CONTROL_DIR).join(overlay_handle);
    trace!("overlay_fs_path will be {overlay_fs_path:?}");
    overlay_fs_path
}

/// Universal device tree overlay handler using Linux configfs.
///
/// This struct manages a single device tree overlay through the Linux configfs
/// mechanism. It stores the overlay's configfs directory path and provides methods
/// to apply, remove, and check the status of the overlay.
///
/// # Fields
///
/// * `overlay_fs_path` - Path to the overlay's directory in configfs
///   (e.g., `/sys/kernel/config/device-tree/overlays/my_overlay`)
///
/// see [config::OVERLAY_CONTROL_DIR] for the base overlay control directory.
#[derive(Debug)]
pub struct UniversalOverlayHandler {
    /// The path which points to the overlay virtual filesystem's dir which contains
    /// `path`, `status` and `dtbo` virtual files for overlay control. `dtbo` appears unused?
    overlay_fs_path: PathBuf,
}

/// Implementation of helper methods for UniversalOverlayHandler.
impl UniversalOverlayHandler {
    /// Read the overlay status from the configfs `status` file.
    ///
    /// Reads from the `<overlay_fs_path>/status` file and returns
    /// the status string with trailing newlines removed.
    ///
    /// # Returns: `Result<String, FpgadError>`
    /// * `Ok(String)` - Status string (typically "applied" or empty)
    /// * `Err(FpgadError::IORead)` - Failed to read status file
    fn get_vfs_status(&self) -> Result<String, FpgadError> {
        let status_path = self.overlay_fs_path()?.join("status");
        trace!("Reading from {status_path:?}");
        fs_read(&status_path).map(|s| s.trim_end_matches('\n').to_string())
    }

    /// Read the overlay path from the configfs `path` file.
    ///
    /// Reads from the  `<overlay_fs_path>/path` file which contains
    /// the device tree path where the overlay was applied.
    ///
    /// # Returns: `Result<PathBuf, FpgadError>`
    /// * `Ok(PathBuf)` - Device tree path from the path file
    /// * `Err(FpgadError::IORead)` - Failed to read path file
    fn get_vfs_path(&self) -> Result<PathBuf, FpgadError> {
        let path_path = self.overlay_fs_path()?.join("path");
        trace!("Reading from {path_path:?}");
        let path_string = fs_read(&path_path).map(|s| s.trim_end_matches('\n').to_string())?;
        Ok(PathBuf::from(path_string))
    }

    /// Verify that an overlay was successfully applied.
    ///
    /// Checks both the `path` and `status` files within  `overlay_fs_path` to ensure that
    /// the overlay was correctly applied. Sometimes an overlay may show "applied" status
    /// but have an empty path, indicating a failure.
    ///
    /// # Arguments
    ///
    /// * `source_path_rel` - The source path that was written to apply the overlay
    ///
    /// # Returns: `Result<(), FpgadError>`
    /// * `Ok(())` - Overlay successfully applied (path matches and status is "applied")
    /// * `Err(FpgadError::OverlayStatus)` - Path doesn't match or status not "applied"
    /// * `Err(FpgadError::IORead)` - Failed to read path or status file
    fn vfs_check_applied(&self, source_path_rel: &Path) -> Result<(), FpgadError> {
        let path_file_contents = &self.get_vfs_path()?;
        if path_file_contents.ends_with(source_path_rel) {
            info!("overlay path contents is valid: '{path_file_contents:?}'");
        } else {
            return Err(FpgadError::OverlayStatus(format!(
                "When trying to apply overlay '{source_path_rel:?}', the resulting vfs path contained '{path_file_contents:?}'"
            )));
        }

        let status = self.status()?;
        match status.contains("applied") {
            true => {
                info!("overlay status is 'applied'")
            }
            false => {
                return Err(FpgadError::OverlayStatus(format!(
                    "After writing to configfs, overlay status does not show 'applied'. Instead it is '{status}'"
                )));
            }
        }

        Ok(())
    }
}

/// Implementation of the OverlayHandler trait for UniversalOverlayHandler.
impl OverlayHandler for UniversalOverlayHandler {
    /// Apply a device tree overlay through configfs.
    ///
    /// Creates the overlay directory in configfs, writes the overlay source path to the
    /// `path` file, and verifies successful application. This may trigger automatic
    /// firmware loading if the overlay specifies a bitstream.
    ///
    /// # Arguments
    ///
    /// * `source_path_rel` - Path to the overlay file (can be absolute or relative to firmware path)
    ///
    /// # Returns: `Result<(), FpgadError>`
    /// * `Ok(())` - Overlay applied and verified successfully
    /// * `Err(FpgadError::Argument)` - Overlay with this handle already exists
    /// * `Err(FpgadError::IOCreate)` - Failed to create overlay directory
    /// * `Err(FpgadError::Internal)` - configfs didn't create `path` file (not mounted?)
    /// * `Err(FpgadError::IOWrite)` - Failed to write overlay path
    /// * `Err(FpgadError::OverlayStatus)` - Overlay didn't apply correctly
    ///
    /// # Notes
    ///
    /// - There are multiple ways device tree overlays can trigger firmware loading
    /// - This method is not valid if the dtbo doesn't contain firmware to load
    /// - The overlay directory must not already exist. [`OverlayHandler::remove_overlay`] can be called
    ///   to remove it first.
    fn apply_overlay(&self, source_path_rel: &Path) -> Result<(), FpgadError> {
        let overlay_fs_path = self.overlay_fs_path()?;
        if overlay_fs_path.exists() {
            return Err(FpgadError::Argument(format!(
                "Overlay with this handle already exists at {overlay_fs_path:?}. \
                 Remove the overlay and try again."
            )));
        }

        fs_create_dir(overlay_fs_path)?;
        trace!("Created dir {overlay_fs_path:?}");

        let overlay_path_file = overlay_fs_path.join("path");
        if !overlay_path_file.exists() {
            // TODO: consider different error type?
            return Err(FpgadError::Internal(format!(
                "Overlay at {overlay_fs_path:?} did not initialise a new overlay: \
                the `path` virtual file did not get created by the kernel. \
                Is the parent dir mounted as a configfs directory?"
            )));
        }

        match fs_write(&overlay_path_file, false, source_path_rel.to_string_lossy()) {
            Ok(_) => {
                trace!("'{source_path_rel:?}' successfully written to {overlay_path_file:?}");
            }
            Err(e) => return Err(e),
        }
        self.vfs_check_applied(source_path_rel)
    }

    /// Remove a device tree overlay from configfs.
    ///
    /// Removes the overlay directory from configfs, which deactivates the overlay and
    /// restores the original device tree state.
    ///
    /// # Returns: `Result<(), FpgadError>`
    /// * `Ok(())` - Overlay directory removed successfully
    /// * `Err(FpgadError::IODelete)` - Failed to remove directory (not empty, doesn't exist, etc.)
    fn remove_overlay(&self) -> Result<(), FpgadError> {
        let overlay_fs_path = self.overlay_fs_path()?;
        fs_remove_dir(overlay_fs_path)
    }

    /// Get the required FPGA programming flags from the overlay.
    ///
    /// # Warning: Not Implemented
    ///
    /// This method is intentionally unimplemented for the universal overlay handler.
    /// In a platform specific implementation (i.e. a softener) this would implement
    /// logic to parse the overlay/relevant package and determine any required FPGA
    /// programming flags.
    ///
    /// # Returns: `Result<isize, FpgadError>`
    /// * `Ok(0)` - Always returns 0 (no flags)
    fn required_flags(&self) -> Result<isize, FpgadError> {
        Ok(0)
    }

    /// Get the current status of the overlay.
    ///
    /// Reads both the `path` and `status` files from configfs and returns a combined
    /// status string. If the overlay directory doesn't exist, returns "not present".
    ///
    /// # Returns: `Result<String, FpgadError>`
    /// * `Ok(String)` - Status string (e.g., "/path/in/tree applied" or "not present")
    /// * `Err(FpgadError::IORead)` - Failed to read status or path file
    fn status(&self) -> Result<String, FpgadError> {
        if !self.overlay_fs_path()?.exists() {
            return Ok("not present".into());
        };
        let path = self.get_vfs_path()?;
        let status = self.get_vfs_status()?;
        Ok(format!("{path:?} {status}"))
    }

    /// Get the filesystem path to the overlay directory.
    ///
    /// Returns the stored overlay configfs path.
    ///
    /// # Returns: `Result<&Path, FpgadError>`
    /// * `Ok(&Path)` - Path to overlay directory in configfs
    fn overlay_fs_path(&self) -> Result<&Path, FpgadError> {
        Ok(self.overlay_fs_path.as_path())
    }
}

impl UniversalOverlayHandler {
    /// Create a new UniversalOverlayHandler for the specified overlay.
    ///
    /// Constructs the configfs path for the overlay but doesn't create the directory
    /// or validate its existence. Directory creation happens in [`apply_overlay`](OverlayHandler::apply_overlay).
    ///
    /// # Arguments
    ///
    /// * `overlay_handle` - The overlay handle (directory name in configfs)
    ///
    /// # Returns: `Self`
    /// * New UniversalOverlayHandler instance
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use daemon::platforms::platform::platform_for_known_platform;
    ///
    /// let handler = platform_for_known_platform("universal").overlay_handler("my_overlay")?;
    /// ```
    pub(crate) fn new(overlay_handle: &str) -> Self {
        UniversalOverlayHandler {
            overlay_fs_path: construct_overlay_fs_path(overlay_handle),
        }
    }
}
