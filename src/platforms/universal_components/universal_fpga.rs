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
use crate::platforms::platform::Fpga;
use crate::system_io::{fs_read, fs_write};
use log::{error, info, trace};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct UniversalFPGA {
    pub(crate) device_handle: String,
}

impl UniversalFPGA {
    /// Constructor simply stores an owned version of the provided name.
    /// This should probably be where we actually check if the device exists in the sysfs
    pub(crate) fn new(device_handle: &str) -> Self {
        UniversalFPGA {
            device_handle: device_handle.to_owned(),
        }
    }

    /// Reads the current fpga state file.
    /// Only succeeds if the state is 'operating'.
    /// Should only be used after bitstream loading.
    pub(crate) fn assert_state(&self) -> Result<(), FpgadError> {
        match self.get_state() {
            Ok(state) => match state.to_string().as_str() {
                "operating" => {
                    info!("{}'s state is 'operating'", self.device_handle);
                    Ok(())
                }
                _ => Err(FpgadError::FPGAState(format!(
                    "After loading bitstream, {}'s state should be should be 'operating' but it is '{}'",
                    self.device_handle, state
                ))),
            },
            Err(e) => Err(e),
        }
    }
}

impl Fpga for UniversalFPGA {
    /// Get the name of this FPGA device e.g. fpga0.
    fn device_handle(&self) -> &str {
        &self.device_handle
    }

    /// Reads and returns contents of `/sys/class/fpga_manager/self.name/state` or FpgadError::IO.
    ///
    /// returns: Result<String, FpgadError>
    fn get_state(&self) -> Result<String, FpgadError> {
        trace!(
            "reading /sys/class/fpga_manager/{}/state",
            self.device_handle
        );
        fs_read(Path::new(&format!(
            "/sys/class/fpga_manager/{}/state",
            self.device_handle
        )))
        .map(|s| s.trim_end_matches('\n').to_string())
    }

    /// Gets the flags from the hex string stored in the sysfs flags file
    /// e.g. sys/class/fpga_manager/fpga0/flags
    fn get_flags(&self) -> Result<isize, FpgadError> {
        let path = format!("/sys/class/fpga_manager/{}/flags", self.device_handle);
        let contents = fs_read(&PathBuf::from(&path))?;
        let trimmed = contents.trim().trim_start_matches("0x");
        isize::from_str_radix(trimmed, 16)
            .map_err(|_| FpgadError::Flag("Parsing flags failed".into()))
    }

    /// Sets the flags in the sysfs flags file (e.g. sys/class/fpga_manager/fpga0/flags)
    /// and verifies the write command stuck by reading it back.
    fn set_flags(&self, flags: isize) -> Result<(), FpgadError> {
        trace!(
            "Writing '{}' to '/sys/class/fpga_manager/{}/flags'",
            flags, self.device_handle
        );
        match fs_write(
            &PathBuf::from(&format!(
                "/sys/class/fpga_manager/{}/flags",
                self.device_handle
            )),
            false,
            flags.to_string(),
        ) {
            Ok(_) => {}
            Err(e) => {
                error!("Failed to read state.");
                return Err(e);
            }
        };

        match self.get_state() {
            Ok(state) => match state.as_str() {
                "operating" => {
                    info!(
                        "{}'s state is 'operating' after writing flags.",
                        self.device_handle
                    )
                }
                _ => {
                    error!(
                        "{}'s state is '{}' after writing flags.",
                        self.device_handle, state
                    );
                }
            },
            Err(e) => return Err(e),
        };

        match self.get_flags() {
            Ok(returned_flags) if returned_flags == flags => Ok(()),
            Ok(returned_flags) => Err(FpgadError::Flag(format!(
                "Setting {}'s flags to '{}' failed. Resulting flag was '{}'",
                self.device_handle, flags, returned_flags
            ))),
            Err(e) => Err(FpgadError::Flag(format!(
                "Failed to read {}'s  flags after setting to '{}': {}",
                self.device_handle, flags, e
            ))),
        }
    }

    /// This can be used to manually load a firmware if the overlay does not trigger the load.
    /// Note: always load firmware before overlay.
    fn load_firmware(&self, bitstream_path: &Path) -> Result<(), FpgadError> {
        fs_write(bitstream_path, false, "/sys/class/fpga_manager/{}/path")?;
        self.assert_state()
    }
}
