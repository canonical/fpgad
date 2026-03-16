// This file is part of fpgad, an application to manage FPGA subsystem together with device-tree and kernel modules.
//
// Copyright 2026 Canonical Ltd.
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
use crate::softeners::xilinx_dfx_mgr;
use crate::system_io::{fs_read, fs_write};
use log::{error, trace};
use std::path::Path;

pub struct XilinxDfxMgrFPGA {
    device_handle: String,
}

impl XilinxDfxMgrFPGA {
    pub(crate) fn new(device_handle: &str) -> Self {
        XilinxDfxMgrFPGA {
            device_handle: device_handle.to_owned(),
        }
    }
}

impl Fpga for XilinxDfxMgrFPGA {
    fn device_handle(&self) -> &str {
        self.device_handle.as_str()
    }

    fn state(&self) -> Result<String, FpgadError> {
        Ok(xilinx_dfx_mgr::list_package()?)
    }
    /// Read the current programming flags from sysfs.
    ///
    /// Reads `/sys/class/fpga_manager/<device>/flags`, parses the hexadecimal string
    /// (format: "0x...", or undecorated), and returns the flags as u32.
    ///
    /// # Returns: `Result<u32, FpgadError>`
    /// * `Ok(u32)` - Current flags value
    /// * `Err(FpgadError::IORead)` - Failed to read flags file
    /// * `Err(FpgadError::Flag)` - Failed to parse hexadecimal value
    fn flags(&self) -> Result<u32, FpgadError> {
        let flag_path = Path::new(config::FPGA_MANAGERS_DIR)
            .join(self.device_handle.clone())
            .join("flags");
        let contents = fs_read(&flag_path)?;
        let trimmed = contents.trim().trim_start_matches("0x");
        u32::from_str_radix(trimmed, 16)
            .map_err(|_| FpgadError::Flag("Parsing flags failed".into()))
    }

    /// Set the programming flags in sysfs.
    ///
    /// Writes the flags to `/sys/class/fpga_manager/<device>/flags` in undecorated
    /// hexadecimal (decimal `32` -> undecorated hex `20`) and verifies that the write
    /// succeeded by reading the value back.
    /// Also checks and logs the FPGA state after setting flags.
    ///
    /// # Arguments
    ///
    /// * `flags` - The flags value to set
    ///
    /// # Returns: `Result<(), FpgadError>`
    /// * `Ok(())` - Flags set and verified successfully
    /// * `Err(FpgadError::IOWrite)` - Failed to write flags file
    /// * `Err(FpgadError::IORead)` - Failed to read back flags or state
    /// * `Err(FpgadError::Flag)` - Read-back value doesn't match written value
    fn set_flags(&self, flags: u32) -> Result<(), FpgadError> {
        let flag_path = Path::new(config::FPGA_MANAGERS_DIR)
            .join(self.device_handle.clone())
            .join("flags");
        trace!("Writing 0x'{flags:X}' to '{flag_path:?}");
        if let Err(e) = fs_write(&flag_path, false, format!("0x{flags:X}")) {
            error!("Failed to read state.");
            return Err(e);
        }
        //  TODO(Artie): this logic will not work because state is calling -listPackage
        // match self.state() {
        //     Ok(state) => match state.as_str() {
        //         "operating" => {
        //             info!(
        //                 "{}'s state is 'operating' after writing flags.",
        //                 self.device_handle
        //             )
        //         }
        //         _ => {
        //             warn!(
        //                 "{}'s state is '{}' after writing flags.",
        //                 self.device_handle, state
        //             );
        //         }
        //     },
        //     Err(e) => return Err(e),
        // };

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

    fn load_firmware(&self, firmware_path: &Path) -> Result<(), FpgadError> {
        xilinx_dfx_mgr::load_bitstream(firmware_path)?;
        Ok(())
    }

    fn remove_firmware(&self, slot_handle: Option<&str>) -> Result<String, FpgadError> {
        Ok(xilinx_dfx_mgr::remove(slot_handle)?)
    }
}
