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
use crate::platforms::platform::Fpga;
use crate::system_io::{fs_read, fs_write};
use log::{error, info, trace};
use std::path::Path;

#[derive(Debug)]
pub struct UniversalFPGA {
    pub(crate) device_handle: String,
}

impl UniversalFPGA {
    /// Constructor simply stores an owned version of the provided name.
    /// This should probably be where we actually check if the device exists in the sysfs
    pub(crate) fn new(device_handle: &str) -> UniversalFPGA {
        UniversalFPGA {
            device_handle: device_handle.to_owned(),
        }
    }

    /// Reads the current fpga state file.
    /// Only succeeds if the state is 'operating'.
    /// Should only be used after bitstream loading.
    pub(crate) fn assert_state(&self) -> Result<(), FpgadError> {
        match self.state() {
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
    fn state(&self) -> Result<String, FpgadError> {
        let state_path = Path::new(config::FPGA_MANAGERS_DIR)
            .join(self.device_handle.clone())
            .join("state");
        trace!("reading {state_path:?}");
        fs_read(&state_path).map(|s| s.trim_end_matches('\n').to_string())
    }

    /// Gets the flags from the hex string stored in the sysfs flags file
    /// e.g. sys/class/fpga_manager/fpga0/flags
    fn flags(&self) -> Result<u32, FpgadError> {
        let flag_path = Path::new(config::FPGA_MANAGERS_DIR)
            .join(self.device_handle.clone())
            .join("flags");
        let contents = fs_read(&flag_path)?;
        let trimmed = contents.trim().trim_start_matches("0x");
        u32::from_str_radix(trimmed, 16)
            .map_err(|_| FpgadError::Flag("Parsing flags failed".into()))
    }

    /// Sets the flags in the sysfs flags file (e.g. sys/class/fpga_manager/fpga0/flags)
    /// and verifies the write command stuck by reading it back.
    fn set_flags(&self, flags: u32) -> Result<(), FpgadError> {
        let flag_path = Path::new(config::FPGA_MANAGERS_DIR)
            .join(self.device_handle.clone())
            .join("flags");
        trace!("Writing '{flags}' to '{flag_path:?}");
        if let Err(e) = fs_write(&flag_path, false, flags.to_string()) {
            error!("Failed to read state.");
            return Err(e);
        }

        match self.state() {
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

        match self.flags() {
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
    fn load_firmware(&self, bitstream_path_rel: &str) -> Result<(), FpgadError> {
        let control_path = Path::new(config::FPGA_MANAGERS_DIR)
            .join(self.device_handle())
            .join("firmware");
        fs_write(&control_path, false, bitstream_path_rel)?;
        self.assert_state()
    }
}
