// This file is part of fpgad, an application to manage FPGA subsystem together with devicetree and kernel modules.
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

use super::platform::{Fpga, OverlayHandler, Platform};
use crate::error::FpgadError::ArgumentError;
use crate::system_io::{fs_write, fs_read, fs_create_dir, fs_remove_dir};
use crate::error::FpgadError;
use log::trace;

use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct UniversalPlatform {
    name: &'static str,
    fpga: Option<UniversalFPGA>,
    overlay_handler: Option<UniversalOverlayHandler>,
}

impl UniversalPlatform {
    /// Creates a new [`UniversalPlatform`].
    pub(crate) fn new() -> Self {
        trace!("creating new UniversalPlatform");
        UniversalPlatform {
            name: "Universal",
            fpga: None,
            overlay_handler: None,
        }
    }

    pub(crate) fn load_package(
        &mut self,
        bitstream_path: &Path,
        overlay_source_path: &Path,
    ) -> Result<(), FpgadError> {
        let overlay_handler = self.overlay_handler.get_or_insert_with(|| {
            UniversalOverlayHandler::new(bitstream_path, overlay_source_path)
        });

        let fpga = self
            .fpga
            .as_mut()
            .ok_or(FpgadError::Internal("FPGA not initialized".into()))?;

        trace!("overlay handler: {:?}", overlay_handler);
        trace!("FPGA: {:?}", fpga);

        if let Ok(flags) = overlay_handler.get_required_flags() {
            fpga.set_flags(flags)?;
        }
        fpga.state()?;

        overlay_handler
            .prepare_for_load()?;
        overlay_handler
            .apply_overlay()?;
        Ok(())
    }

    /// Removes the overlay, undoing any extra steps, and then deletes the overlay_handler
    pub(crate) fn unload_package(&mut self) -> Result<(), FpgadError> {
        self.overlay_handler.take();
        Ok(())
    }
}

impl Drop for UniversalOverlayHandler {
    fn drop(&mut self) {
        if self.rollback_steps.delete_overlay_in_fw {
            self.remove_overlay().expect("Failed to remove overlay");
        }
        if self.rollback_steps.reset_firmware_path {
            println!("NOT IMPLEMENTED: UniversalOverlayHandler::reset_firmware_path()");
        }

        println!("Dropping MyStruct!");
        // TODO: Go through steps which are true in self.rollback_steps.
    }
}

impl Platform for UniversalPlatform {
    /// Returns the `name` of the [`UniversalPlatform`]
    fn name(&self) -> &str {
        self.name
    }
    /// Initialises or get the fpga object called `name`
    fn fpga(&mut self, name: &str) -> &impl Fpga {
        assert!(
            !name.is_empty() && name.is_ascii(),
            "fpga name must be compliant with sysfs rules."
        );

        // Create FPGA if not same or present
        if self.fpga.as_ref().is_none_or(|f| f.name != name) {
            self.fpga = Some(UniversalFPGA::new(name));
        }
        self.fpga.as_ref().unwrap()
    }

    /// Gets the `overlay_handler` associated with this device.
    fn overlay_handler(
        &mut self,
        bitstream_path: &Path,
        overlay_source_path: &Path,
    ) -> &impl OverlayHandler {
        // Create FPGA if not same or present
        if self.overlay_handler.as_ref().is_none() {
            self.overlay_handler = Some(UniversalOverlayHandler::new(
                bitstream_path,
                overlay_source_path,
            ));
        }
        self.overlay_handler.as_ref().unwrap()
    }
}

#[derive(Debug)]
struct OverlayRollbackSteps {
    delete_overlay_in_fw: bool,
    reset_firmware_path: bool,
}
#[derive(Debug)]
pub struct UniversalOverlayHandler {
    // we have the source directory, the overlayfs directory, and the name of the relevant files
    bitstream_source_path: PathBuf,
    overlay_source_path: PathBuf,
    overlay_fs_path: PathBuf,
    rollback_steps: OverlayRollbackSteps,
}

impl OverlayHandler for UniversalOverlayHandler {
    /// `/sys/module/firmware_class/parameters/path`
    fn prepare_for_load(&mut self) -> Result<(), FpgadError> {
        if !self.overlay_source_path.exists() | self.overlay_fs_path.is_dir() {
            return Err(ArgumentError(format!(
                "Overlay file '{:?}' has invalid path.",
                self.overlay_source_path
            )));
        }

        if !self.bitstream_source_path.exists() | self.overlay_fs_path.is_dir() {
            return Err(ArgumentError(format!(
                "Bitstream file '{:?}' has invalid path.",
                self.overlay_fs_path
            )));
        }

        if self.overlay_fs_path.exists() {
            fs_remove_dir(&self.overlay_fs_path)?
        }

        trace!("Checking configfs path exists.");
        if !self.overlay_fs_path.parent().unwrap().exists() {
            eprintln!("NOOOOO! no path {:?}", self.overlay_fs_path.parent())
        }

        trace!("Attempting to create '{:?}'", self.overlay_fs_path);
        fs_create_dir(&self.overlay_fs_path)?;
        trace!("Created dir {:?}", self.overlay_fs_path);

        self.rollback_steps.delete_overlay_in_fw = true;

        Ok(())
    }

    fn apply_overlay(&self) -> Result<(), FpgadError> {
        let dtbo_file_name = self
            .overlay_source_path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();
        let overlay_path_file = self.overlay_fs_path.join("path");
        match fs_write(&overlay_path_file, false, dtbo_file_name) {
            Ok(_) => {
                trace!(
                    "{} succesfully written to {:?}",
                    dtbo_file_name, overlay_path_file
                );
            }
            Err(e) => {
                return Err(FpgadError::IO(format!(
                    "Failed to write overlay path {} to {:?} : {}",
                    dtbo_file_name, overlay_path_file, e
                )));
            }
        }
        match self.status() {
            Ok(..) => Ok(()),
            Err(e) => Err(e),
        }
    }
    fn remove_overlay(&self) -> Result<(), FpgadError> {
        Ok(fs_remove_dir(&self.overlay_fs_path)?)
    }

    fn get_required_flags(&self) -> Result<isize, FpgadError> {
        Ok(0)
    }

    fn status(&self) -> Result<String, FpgadError> {
        let status_path = self.overlay_fs_path.join("status");

        trace!("Reading from {:?}", status_path);
        let state = match fs_read(&status_path) {
            Ok(val) => Ok(val),
            Err(e) => Err(e),
        };
        match state {
            Ok(val) => match val.as_str() {
                "applied\n" => Ok(val),
                _ => Err(FpgadError::OverlayStatusError(val)),
            },
            Err(e) => Err(e),
        }
    }
}

#[derive(Debug)]
pub struct UniversalFPGA {
    name: String,
}

impl UniversalFPGA {
    pub(crate) fn new(name: &str) -> Self {
        UniversalFPGA {
            name: name.to_owned(),
        }
    }
}

impl UniversalOverlayHandler {
    /// Scans the package dir for reqiured files
    pub(crate) fn new(bitstream_path: &Path, overlay_source_path: &Path) -> Self {
        let overlay_fs = Path::new("/sys/kernel/config/device-tree/overlays/fpgad_overlay_0");

        UniversalOverlayHandler {
            bitstream_source_path: bitstream_path.to_owned(),
            overlay_source_path: overlay_source_path.to_owned(),
            overlay_fs_path: overlay_fs.to_owned(),
            rollback_steps: OverlayRollbackSteps {
                delete_overlay_in_fw: false,
                reset_firmware_path: false,
            },
        }
    }
}

impl Fpga for UniversalFPGA {
    /// Name of this FPGA device e.g. fpga0.
    fn name(&self) -> &str {
        &self.name
    }

    /// Reads and returns contents of `/sys/class/fpga_manager/self.name/state` or FpgadError::IO.
    ///
    /// returns: Result<String, FpgadError>
    fn state(&self) -> Result<String, FpgadError> {
        trace!("reading /sys/class/fpga_manager/{}/state", self.name);
        let state = match fs_read(Path::new(&format!(
            "/sys/class/fpga_manager/{}/state",
            self.name
        ))) {
            Ok(val) => Ok(val),
            Err(e) => Err(e),
        };
        match state {
            Ok(val) => match val.as_str() {
                "operating\n" => Ok(val),
                "unknown\n" => {
                    trace!(
                        "fpga state shows an 'unknown'> Fine if this is first boot, sketchy otherwise."
                    );
                    Ok(val)
                }
                _ => Err(FpgadError::StateError(val)),
            },
            Err(e) => Err(e),
        }
    }

    fn get_flags(&self) -> Result<isize, FpgadError> {
        let path = format!("/sys/class/fpga_manager/{}/flags", self.name);
        let contents = fs_read(&PathBuf::from(&path))?;
        let trimmed = contents.trim().trim_start_matches("0x");
        isize::from_str_radix(trimmed, 16)
            .map_err(|_| FpgadError::FlagError("Parsing flags failed".into()))
    }

    fn set_flags(&self, flags: isize) -> Result<(), FpgadError> {
        trace!(
            "Writing {} to '/sys/class/fpga_manager/{}/flags'",
            flags, self.name
        );
        match fs_write(
            &PathBuf::from(&format!("/sys/class/fpga_manager/{}/flags", self.name)),
            false,
            flags.to_string(),
        ) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Failed to read state.");
                return Err(e);
            }
        };

        match self.state() {
            Ok(..) => Ok(()),
            Err(e) => Err(e),
        }
        .expect("TODO: panic message");

        match self.get_flags() {
            Ok(returned_flags) if returned_flags == flags => Ok(()),
            Ok(returned_flags) => Err(FpgadError::FlagError(format!(
                "Setting flags to {} failed. Returned flag was {}",
                flags, returned_flags
            ))),
            Err(e) => Err(FpgadError::FlagError(format!(
                "Failed to read flags after setting to {}: {}",
                flags, e
            ))),
        }
    }
}
