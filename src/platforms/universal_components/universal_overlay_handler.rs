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

use crate::error::FpgadError;
use crate::error::FpgadError::{ArgumentError, OverlayStatusError};
use crate::platforms::platform::OverlayHandler;
use crate::system_io::{fs_create_dir, fs_read, fs_remove_dir, fs_write};
use log::{error, info, trace};
use std::path::{Path, PathBuf};

/// Store the steps that need to be undone on delete/failure.
#[derive(Debug)]
struct OverlayRollbackSteps {
    delete_configfs_dir: bool,
}
/// Stores the three relevant paths: source files for dtbo/bitstream and the overlayfs dir to which
/// the dtbo path was written.
/// Also stores the steps that need to be undone
#[derive(Debug)]
pub struct UniversalOverlayHandler {
    overlay_source_path: PathBuf,
    overlay_fs_path: PathBuf,
    rollback_steps: OverlayRollbackSteps,
}

fn extract_filename(path: &Path) -> Result<&str, FpgadError> {
    path.file_name()
        .ok_or_else(|| FpgadError::Internal(format!("No filename in path: {:?}", path)))?
        .to_str()
        .ok_or_else(|| FpgadError::Internal(format!("Filename not UTF-8: {:?}", path)))
}

impl UniversalOverlayHandler {
    fn get_vfs_status(&self) -> Result<String, FpgadError> {
        let status_path = self.overlay_fs_path.join("status");

        trace!("Reading from {:?}", status_path);
        fs_read(&status_path).map(|s| s.trim_end_matches('\n').to_string())
    }
    /// Read path from <overlay_fs_path>/path file and verify that what was meant to be applied
    /// was applied.
    fn get_vfs_path(&self) -> Result<String, FpgadError> {
        let path_path = self.overlay_fs_path.join("path");

        trace!("Reading from {:?}", path_path);
        fs_read(&path_path).map(|s| s.trim_end_matches('\n').to_string())
    }

    /// When an overlay fails to be applied, it may show as "applied" status but the path will
    /// be empty. Therefore, this checks both match what is expected.
    fn vfs_check_applied(&self) -> Result<(), FpgadError> {
        let path_contents = self.get_vfs_path()?;
        let dtbo_file_name = extract_filename(&self.overlay_source_path)?;
        match path_contents.contains(dtbo_file_name) {
            true => {
                info!("overlay path contents is valid: '{}'", path_contents)
            }
            false => {
                return Err(OverlayStatusError(format!(
                    "When trying to apply overlay '{}', the resulting vfs path contained '{}'",
                    dtbo_file_name, path_contents
                )));
            }
        }

        let status = self.get_status()?;
        match status.contains("applied") {
            true => {
                info!("overlay status is 'applied'")
            }
            false => {
                return Err(OverlayStatusError(format!(
                    "After writing to configfs, overlay status does not show 'applied'. Instead it is '{}'",
                    status
                )));
            }
        }

        Ok(())
    }
}

impl OverlayHandler for UniversalOverlayHandler {
    /// Checks inputs are real files (doesn't yet check they are valid)
    /// Checks for `overlay_fs_path`.
    /// In future this may change the firmware location through
    /// `/sys/module/firmware_class/parameters/`.
    fn prepare_for_load(&mut self) -> Result<(), FpgadError> {
        if !self.overlay_source_path.exists() | self.overlay_source_path.is_dir() {
            return Err(ArgumentError(format!(
                "Overlay file '{:?}' has invalid path.",
                self.overlay_source_path
            )));
        }

        if self.overlay_fs_path.exists() {
            fs_remove_dir(&self.overlay_fs_path)?
        }

        trace!(
            "Checking configfs path exists at {:?}",
            self.overlay_fs_path
        );
        if let Some(parent_path) = self.overlay_fs_path.parent() {
            if !parent_path.exists() {
                return Err(ArgumentError(format!(
                    "The path {:?} doesn't seem to exist.",
                    parent_path
                )));
            }
        } else {
            return Err(ArgumentError(format!(
                "The path {:?} has no parent directory.",
                self.overlay_fs_path
            )));
        }

        trace!("Attempting to create '{:?}'", self.overlay_fs_path);
        fs_create_dir(&self.overlay_fs_path)?;
        trace!("Created dir {:?}", self.overlay_fs_path);

        self.rollback_steps.delete_configfs_dir = true;

        Ok(())
    }

    /// Attempts to apply a device tree overlay which should trigger a firmware load.
    /// There are multiple ways to trigger a firmware load so this is not valid if the
    /// dtbo doesn't contain a firmware to load.
    fn apply_overlay(&self) -> Result<(), FpgadError> {
        let dtbo_file_name = extract_filename(&self.overlay_source_path)?;
        let overlay_path_file = self.overlay_fs_path.join("path");
        match fs_write(&overlay_path_file, false, dtbo_file_name) {
            Ok(_) => {
                trace!(
                    "'{}' successfully written to {:?}",
                    dtbo_file_name, overlay_path_file
                );
            }
            Err(e) => {
                return Err(FpgadError::IO(format!(
                    "Failed to write overlay path '{}' to '{:?}' : '{}'",
                    dtbo_file_name, overlay_path_file, e
                )));
            }
        }
        self.vfs_check_applied()
    }

    /// Attempts to delete overlay_fs_path
    fn remove_overlay(&mut self) -> Result<(), FpgadError> {
        if self.rollback_steps.delete_configfs_dir {
            let removed = fs_remove_dir(&self.overlay_fs_path);
            self.rollback_steps.delete_configfs_dir = false;
            removed?
        }
        Ok(())
    }

    /// WARNING NOT IMPLEMENTED:
    /// This is where the required fpga flags will be determined from the dtbo,
    /// such as compressed or encrypted.
    fn get_required_flags(&self) -> Result<isize, FpgadError> {
        Ok(0)
    }

    /// Read status from <overlay_fs_path>/status file and verify that it is "applied"
    fn get_status(&self) -> Result<String, FpgadError> {
        self.get_vfs_status()
    }
}

impl UniversalOverlayHandler {
    /// Scans the package dir for required files
    pub(crate) fn new(overlay_source_path: &Path) -> Self {
        let overlay_fs = Path::new("/sys/kernel/config/device-tree/overlays/fpgad_overlay_0");

        UniversalOverlayHandler {
            overlay_source_path: overlay_source_path.to_owned(),
            overlay_fs_path: overlay_fs.to_owned(),
            rollback_steps: OverlayRollbackSteps {
                delete_configfs_dir: false,
            },
        }
    }
}
