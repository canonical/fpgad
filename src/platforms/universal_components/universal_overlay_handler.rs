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
use crate::error::FpgadError::{ArgumentError, InternalError, OverlayStatusError};
use crate::platforms::platform::OverlayHandler;
use crate::system_io::{extract_filename, fs_create_dir, fs_read, fs_remove_dir, fs_write};
use log::trace;
use std::path::{Path, PathBuf};

/// Stores the three relevant paths: source files for dtbo/bitstream and the overlayfs dir to which
/// the dtbo path was written.
#[derive(Debug)]
pub struct UniversalOverlayHandler {
    pub(crate) overlay_source_path: Option<PathBuf>,
    pub(crate) overlay_fs_path: Option<PathBuf>,
}

impl UniversalOverlayHandler {
    /// Checks inputs are real files (doesn't yet check they are valid)
    /// Checks for `overlay_fs_path`.
    /// In future this may change the firmware location through
    /// `/sys/module/firmware_class/parameters/`.
    fn prepare_for_load(&self) -> Result<(), FpgadError> {
        let source_path = self
            .overlay_source_path
            .clone()
            .ok_or(InternalError(format!(
                "Attempting to get vfs application state failed because UniversalOverlayHandler is \
                not initialised with an appropriate overlay source path: {:?}",
                self
            )))?;
        if !source_path.exists() | source_path.is_dir() {
            return Err(ArgumentError(format!(
                "Overlay file '{:?}' has invalid path.",
                self.overlay_source_path
            )));
        }

        let overlay_fs_path = self.overlay_fs_path.clone().ok_or(InternalError(format!(
            "Failed when preparing to load overlay because UniversalOverlayHandler is \
                not initialised with an appropriate overlayfs path: {:?}",
            self
        )))?;
        if overlay_fs_path.exists() {
            fs_remove_dir(&overlay_fs_path)?
        }

        trace!("Checking configfs path exists at {:?}", overlay_fs_path);
        if let Some(parent_path) = source_path.parent() {
            if !parent_path.exists() {
                return Err(ArgumentError(format!(
                    "The path {:?} doesn't seem to exist.",
                    parent_path
                )));
            }
        } else {
            return Err(ArgumentError(format!(
                "The path {:?} has no parent directory.",
                overlay_fs_path
            )));
        }

        trace!("Attempting to create '{:?}'", overlay_fs_path);
        fs_create_dir(&overlay_fs_path)?;
        trace!("Created dir {:?}", overlay_fs_path);

        Ok(())
    }

    fn get_vfs_status(&self) -> Result<String, FpgadError> {
        let status_path = self.overlay_fs_path.clone().map_or_else(
            || {
                Err(InternalError(format!(
                    "Attempting to get vfs path failed because UniversalOverlayHandler is not \
                    initialised with an appropriate overlayfs directory: {:?}",
                    self
                )))
            },
            |p| Ok(p.join("status")),
        )?;

        trace!("Reading from {:?}", status_path);
        fs_read(&status_path).map(|s| s.trim_end_matches('\n').to_string())
    }
    /// Read path from <overlay_fs_path>/path file and verify that what was meant to be applied
    /// was applied.
    fn get_vfs_path(&self) -> Result<String, FpgadError> {
        let path_path = self.overlay_fs_path.clone().map_or_else(
            || {
                Err(InternalError(format!(
                    "Attempting to get vfs path failed because UniversalOverlayHandler is not \
                    initialised with an appropriate overlayfs directory: {:?}",
                    self
                )))
            },
            |p| Ok(p.join("path")),
        )?;

        trace!("Reading from {:?}", path_path);
        fs_read(&path_path).map(|s| s.trim_end_matches('\n').to_string())
    }

    /// When an overlay fails to be applied, it may show as "applied" status but the path will
    /// be empty. Therefore, this checks both match what is expected.
    fn vfs_check_applied(&self) -> Result<(), FpgadError> {
        let path_contents = self.get_vfs_path()?;
        let source_path = self
            .overlay_source_path
            .clone()
            .ok_or(InternalError(format!(
                "Attempting to get vfs application state failed because UniversalOverlayHandler is \
                not initialised with an appropriate overlay source path: {:?}",
                self
            )))?;
        // TODO: these unwraps are unsafe.
        let dtbo_file_name = extract_filename(&source_path)?;
        match path_contents.contains(dtbo_file_name) {
            true => {
                println!("overlay path contents is valid: '{}'", path_contents)
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
                println!("overlay status is 'applied'")
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
    /// Attempts to apply a device tree overlay which should trigger a firmware load.
    /// There are multiple ways to trigger a firmware load so this is not valid if the
    /// dtbo doesn't contain a firmware to load.
    fn apply_overlay(&self) -> Result<(), FpgadError> {
        self.prepare_for_load()?;

        let source_path = self
            .overlay_source_path
            .clone()
            .ok_or(InternalError(format!(
                "Attempting to apply overlay failed because UniversalOverlayHandler is \
                not initialised with an appropriate overlay source path: {:?}",
                self
            )))?;
        let overlay_path_file = self.overlay_fs_path.clone().map_or_else(
            || {
                Err(InternalError(format!(
                    "Attempting to apply overlay failed because UniversalOverlayHandler is not \
                    initialised with an appropriate overlayfs directory: {:?}",
                    self
                )))
            },
            |p| Ok(p.join("path")),
        )?;
        let dtbo_file_name = extract_filename(&source_path)?;
        match fs_write(&overlay_path_file, false, dtbo_file_name) {
            Ok(_) => {
                trace!(
                    "'{}' successfully written to {:?}",
                    dtbo_file_name, overlay_path_file
                );
            }
            Err(e) => {
                return Err(FpgadError::IOError(format!(
                    "Failed to write overlay path '{}' to '{:?}' : '{}'",
                    dtbo_file_name, overlay_path_file, e
                )));
            }
        }
        self.vfs_check_applied()
    }

    /// Attempts to delete overlay_fs_path
    fn remove_overlay(&self) -> Result<(), FpgadError> {
        let overlay_fs_path = self.overlay_fs_path.clone().ok_or(InternalError(format!(
            "Attempting to remove overlay failed because UniversalOverlayHandler is \
                not initialised with an appropriate overlay fs path: {:?}",
            self
        )))?;

        let removed = fs_remove_dir(&overlay_fs_path);
        removed
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

    fn set_source_path(&mut self, source_path: &Path) -> Result<(), FpgadError> {
        Ok(self.overlay_source_path = Option::from(source_path.to_owned()))
    }

    fn set_overlay_fs_path(&mut self, overlay_handle: &str) {
        self.overlay_fs_path = Option::from(
            PathBuf::from("/sys/kernel/config/device-tree/overlays/").join(overlay_handle),
        );
    }
}

impl UniversalOverlayHandler {
    /// Scans the package dir for required files
    pub(crate) fn new() -> Self {
        UniversalOverlayHandler {
            overlay_source_path: None,
            overlay_fs_path: None,
        }
    }
}
