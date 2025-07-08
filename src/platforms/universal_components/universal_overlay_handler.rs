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

use crate::config;
use crate::error::FpgadError;
use crate::platforms::platform::OverlayHandler;
use crate::system_io::{fs_create_dir, fs_read, fs_remove_dir, fs_write};
use log::{info, trace};
use std::path::{Path, PathBuf};

/// Takes a handle and creates and stores an appropriate overlay_fs_path in this object.
/// The overlay_fs_path is static apart from the handle associated with each
/// device, overlay or bitstream, and so the handle is specified by the user here and the rest
/// is fixed.
fn construct_overlay_fs_path(overlay_handle: &str) -> PathBuf {
    let overlay_fs_path = PathBuf::from(config::OVERLAY_CONTROL_DIR).join(overlay_handle);
    trace!("overlay_fs_path will be {overlay_fs_path:?}");
    overlay_fs_path
}

/// Stores the three relevant paths: source files for dtbo/bitstream and the overlayfs dir to which
/// the dtbo path was written.
#[derive(Debug)]
pub struct UniversalOverlayHandler {
    /// The path which points to the overlay virtual filesystem's dir which contains
    /// `path`, `status` and `dtbo` virtual files for overlay control. `dtbo` appears unused?
    overlay_fs_path: PathBuf,
}

impl UniversalOverlayHandler {
    fn get_vfs_status(&self) -> Result<String, FpgadError> {
        let status_path = self.overlay_fs_path()?.join("status");
        trace!("Reading from {status_path:?}");
        fs_read(&status_path).map(|s| s.trim_end_matches('\n').to_string())
    }
    /// Read path from <overlay_fs_path>/path file and verify that what was meant to be applied
    /// was applied.
    fn get_vfs_path(&self) -> Result<String, FpgadError> {
        let path_path = self.overlay_fs_path()?.join("path");
        trace!("Reading from {path_path:?}");
        fs_read(&path_path).map(|s| s.trim_end_matches('\n').to_string())
    }

    /// When an overlay fails to be applied, it may show as "applied" status but the path will
    /// be empty. Therefore, this checks both match what is expected.
    fn vfs_check_applied(&self, source_path_rel: &str) -> Result<(), FpgadError> {
        let path_file_contents = self.get_vfs_path()?;
        if path_file_contents.contains(source_path_rel) {
            info!("overlay path contents is valid: '{path_file_contents}'");
        } else {
            return Err(FpgadError::OverlayStatus(format!(
                "When trying to apply overlay '{source_path_rel}', the resulting vfs path contained '{path_file_contents}'"
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

impl OverlayHandler for UniversalOverlayHandler {
    /// Attempts to apply a device tree overlay which should trigger a firmware load.
    /// There are multiple ways to trigger a firmware load so this is not valid if the
    /// dtbo doesn't contain a firmware to load.
    /// Calls prepare_for_load to ensure paths are valid etc. beforehand.
    fn apply_overlay(&self, source_path_rel: &str) -> Result<(), FpgadError> {
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

        match fs_write(&overlay_path_file, false, source_path_rel) {
            Ok(_) => {
                trace!("'{source_path_rel}' successfully written to {overlay_path_file:?}");
            }
            Err(e) => return Err(e),
        }
        self.vfs_check_applied(source_path_rel)
    }

    /// Attempts to delete overlay_fs_path
    fn remove_overlay(&self) -> Result<(), FpgadError> {
        let overlay_fs_path = self.overlay_fs_path()?;
        fs_remove_dir(overlay_fs_path)
    }

    /// WARNING NOT IMPLEMENTED:
    /// This is where the required fpga flags will be determined from the dtbo,
    /// such as compressed or encrypted.
    fn required_flags(&self) -> Result<isize, FpgadError> {
        Ok(0)
    }

    /// Read status from <overlay_fs_path>/status file and verify that it is "applied"
    fn status(&self) -> Result<String, FpgadError> {
        if !self.overlay_fs_path()?.exists() {
            return Ok("not present".into());
        };
        let path = self.get_vfs_path()?;
        let status = self.get_vfs_status()?;
        Ok(format!("{path:?} {status}"))
    }

    /// Checks that the overlay_fs_path is stored at time of call and returns it if so (unwraps Option into Result)
    fn overlay_fs_path(&self) -> Result<&Path, FpgadError> {
        Ok(self.overlay_fs_path.as_path())
    }
}

impl UniversalOverlayHandler {
    /// Scans the package dir for required files
    pub(crate) fn new(overlay_handle: &str) -> Self {
        UniversalOverlayHandler {
            overlay_fs_path: construct_overlay_fs_path(overlay_handle),
        }
    }
}
