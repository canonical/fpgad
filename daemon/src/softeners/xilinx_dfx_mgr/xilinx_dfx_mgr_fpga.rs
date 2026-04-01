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

//! Xilinx DFX Manager FPGA device implementation.
//!
//! This module provides the [`XilinxDfxMgrFPGA`] struct, which implements the [`Fpga`] trait
//! for Xilinx FPGA devices using the dfx-mgr backend. It provides a hybrid approach that:
//! - Uses standard sysfs for reading flags
//! - Uses dfx-mgr-client for bitstream loading and package management
//! - Supports dfx-mgr's slot-based management
//!
//! # Key Differences from Universal Platform
//!
//! - **State Query**: Returns dfx-mgr package listing instead of simple sysfs state
//! - **Bitstream Loading**: Uses `dfx-mgr-client -b` instead of direct firmware loading due to snap confinement
//!   limitations (temporary?)
//! - **Firmware Removal**: Supports slot-based removal via `dfx-mgr-client -remove`
//!
//! # Examples
//!
//! ```rust,no_run
//! # use daemon::platforms::platform::platform_for_known_platform;
//! # fn example() -> Result<(), daemon::error::FpgadError> {
//! let platform = platform_for_known_platform("xlnx,zynqmp-pcap-fpga")?;
//! let fpga = platform.fpga("fpga0")?;
//! let state = fpga.state()?;  // Returns dfx-mgr package listing
//! # Ok(())
//! # }
//! ```

use crate::config;
use crate::error::FpgadError;
use crate::platforms::platform::Fpga;
use crate::softeners::xilinx_dfx_mgr;
use crate::system_io::{fs_read, fs_write};
use log::{error, trace};
use std::path::Path;

/// Xilinx DFX Manager FPGA device implementation.
///
/// This struct represents a Xilinx FPGA device and provides methods to interact
/// with it through the dfx-mgr backend. It stores only the device handle and
/// uses dfx-mgr-client for most operations.
///
/// # Fields
///
/// * `device_handle` - The device identifier (e.g., "fpga0") used to locate the device in sysfs.
///   Only used for sysfs backed functions e.g. flags
///
/// # Implementation Notes
///
/// While this implementation uses the dfx-mgr backend, it still reads flags directly
/// from sysfs for consistency with the standard FPGA subsystem interface.
pub struct XilinxDfxMgrFPGA {
    device_handle: String,
}

impl XilinxDfxMgrFPGA {
    /// Create a new XilinxDfxMgrFPGA instance for the specified device.
    ///
    /// This constructor simply stores the device handle. It does not verify that
    /// the device exists or that dfx-mgrd is running - validation occurs when
    /// methods are called.
    ///
    /// # Arguments
    ///
    /// * `device_handle` - The device handle (e.g., "fpga0")
    ///
    /// # Returns: `Self`
    /// * New XilinxDfxMgrFPGA instance
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use daemon::softeners::xilinx_dfx_mgr_fpga::XilinxDfxMgrFPGA;
    ///
    /// let fpga = XilinxDfxMgrFPGA::new("fpga0");
    /// ```
    pub(crate) fn new(device_handle: &str) -> Self {
        XilinxDfxMgrFPGA {
            device_handle: device_handle.to_owned(),
        }
    }
}

impl Fpga for XilinxDfxMgrFPGA {
    /// Get the device handle for this FPGA.
    ///
    /// Returns the device handle (e.g., "fpga0") that identifies this FPGA in sysfs.
    ///
    /// # Returns: `&str`
    /// * The device handle string
    fn device_handle(&self) -> &str {
        self.device_handle.as_str()
    }

    /// Get the current state of the FPGA via dfx-mgr package listing.
    ///
    /// Returns the output of `dfx-mgr-client -listPackage`, which provides a formatted
    /// table showing all available accelerator packages, their types, load status, and
    /// slot assignments.
    ///
    /// # Returns: `Result<String, FpgadError>`
    /// * `Ok(String)` - Formatted package listing from dfx-mgr
    /// * `Err(FpgadSoftenerError)` - Failed to communicate with dfx-mgrd or parse output
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
    fn set_flags(&self, flags: u32) -> Result<String, FpgadError> {
        let flag_path = Path::new(config::FPGA_MANAGERS_DIR)
            .join(self.device_handle.clone())
            .join("flags");
        trace!("Writing '0x{flags:X}' to '{flag_path:#?}'");
        if let Err(e) = fs_write(&flag_path, false, format!("0x{flags:X}")) {
            error!("Failed to read state.");
            return Err(e);
        }
        //  TODO(Artie): how to check success when doing -listPackage

        match self.flags() {
            Ok(returned_flags) if returned_flags == flags => Ok(format!(
                "Flags set to '0x{:X}' for '{}'",
                flags, self.device_handle
            )),
            Ok(returned_flags) => Err(FpgadError::Flag(format!(
                "Setting '{}'s flags to '{}' failed. Resulting flag was '{}'",
                self.device_handle, flags, returned_flags
            ))),
            Err(e) => Err(FpgadError::Flag(format!(
                "Failed to read '{}'s flags after setting to '{}': {}",
                self.device_handle, flags, e
            ))),
        }
    }

    fn load_firmware(&self, firmware_path: &Path, _: &Path) -> Result<String, FpgadError> {
        Ok(xilinx_dfx_mgr::load_bitstream(firmware_path)?)
    }

    fn remove_firmware(&self, slot_handle: Option<&str>) -> Result<String, FpgadError> {
        Ok(xilinx_dfx_mgr::remove(slot_handle)?)
    }
}
