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

//! Universal FPGA device implementation.
//!
//! This module provides the [`UniversalFPGA`] struct, which implements the [`Fpga`] trait
//! for generic FPGA devices using the standard Linux FPGA subsystem. It provides direct
//! access to sysfs attributes without vendor-specific logic.
//!
//! # A sysfs map of an fpga in fpga_manager class.
//!
//! Below is an example sysfs layout for an FPGA device managed by the standard Linux FPGA subsystem for a xilinx kria board:
//! ```text
//! ubuntu@kria:~$ tree /sys/class/fpga_manager/fpga0
//! /sys/class/fpga_manager/fpga0
//! ├── device -> ../../../firmware:zynqmp-firmware:pcap
//! ├── firmware
//! ├── flags
//! ├── key
//! ├── name
//! ├── of_node -> ../../../../../../firmware/devicetree/base/firmware/zynqmp-firmware/pcap
//! ├── power
//! │   ├── async
//! │   ├── autosuspend_delay_ms
//! │   ├── control
//! │   ├── runtime_active_kids
//! │   ├── runtime_active_time
//! │   ├── runtime_enabled
//! │   ├── runtime_status
//! │   ├── runtime_suspended_time
//! │   └── runtime_usage
//! ├── state
//! ├── status
//! ├── subsystem -> ../../../../../../class/fpga_manager
//! └── uevent
//! ```
//! Of these files, only the following are interacted with by this implementation:
//! - `state` - Current FPGA state (operating, unknown, write error, etc.)
//! - `flags` - Programming flags (hexadecimal format: "0x...")
//! - `firmware` - Trigger bitstream loading by writing filename
//!
//! with any other files being controllable using the
//! [`write_property_bytes`](../../../../daemon/comm/dbus/control_interface/struct.ControlInterface.html#method.write_property_bytes)
//! and
//! [`write_property`](../../../../daemon/comm/dbus/control_interface/struct.ControlInterface.html#method.write_property)
//! DBus methods.
//! See the [`control_interface`](../../../../daemon/comm/dbus/control_interface/index.html) documentation for more details.
//!
//! # Examples
//!
//! ```rust,no_run
//! # use daemon::platforms::universal_components::universal_fpga::UniversalFPGA;
//! # use daemon::platforms::platform::platform_for_known_platform;
//! #
//! # fn example() -> Result<(), daemon::error::FpgadError> {
//! let fpga = platform_for_known_platform("universal").fpga("fpga0")?;
//!
//! // Check state
//! let state = fpga.state()?;
//! println!("FPGA state: {}", state);
//!
//! // Get flags
//! let flags = fpga.flags()?;
//! println!("Flags: 0x{:X}", flags);
//! # Ok(())
//! # }
//! ```

use crate::config;
use crate::error::FpgadError;
use crate::platforms::platform::Fpga;
use crate::system_io::{fs_read, fs_write};
use log::{error, info, trace, warn};
use std::path::Path;

/// Universal FPGA device implementation using standard Linux FPGA subsystem.
///
/// This struct represents a single FPGA device and provides methods to interact
/// with it through sysfs. It stores only the device handle (e.g., "fpga0") and
/// constructs paths to sysfs files on demand.
///
/// # Fields
///
/// * `device_handle` - The device identifier (e.g., "fpga0") used to locate the device in sysfs
///
#[derive(Debug)]
pub struct UniversalFPGA {
    pub(crate) device_handle: String,
}

impl UniversalFPGA {
    /// Create a new UniversalFPGA instance for the specified device.
    ///
    /// This constructor simply stores the device handle. It does not verify that
    /// the device exists in sysfs - validation occurs when methods are called.
    ///
    /// # Arguments
    ///
    /// * `device_handle` - The device handle (e.g., "fpga0")
    ///
    /// # Returns: `UniversalFPGA`
    /// * New UniversalFPGA instance
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use daemon::platforms::universal_components::universal_fpga::UniversalFPGA;
    ///
    /// let fpga = platform_for_known_platform("universal").fpga("fpga0")?;
    /// ```
    pub(crate) fn new(device_handle: &str) -> UniversalFPGA {
        UniversalFPGA {
            device_handle: device_handle.to_owned(),
        }
    }

    /// Verify that the FPGA is in the "operating" state.
    ///
    /// Reads the FPGA state and checks if it equals "operating". This method should
    /// be called after bitstream loading to ensure the FPGA successfully configured.
    ///
    /// # Returns: `Result<(), FpgadError>`
    /// * `Ok(())` - FPGA is in "operating" state
    /// * `Err(FpgadError::FPGAState)` - FPGA is in a different state
    /// * `Err(FpgadError::IORead)` - Failed to read state file
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use daemon::platforms::universal_components::universal_fpga::UniversalFPGA;
    /// # fn example(fpga: &UniversalFPGA) -> Result<(), daemon::error::FpgadError> {
    /// // After loading a bitstream
    /// fpga.assert_state()?;
    /// println!("FPGA is operating correctly");
    /// # Ok(())
    /// # }
    /// ```
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
    /// Get the device handle for this FPGA.
    ///
    /// Returns the stored device handle string.
    ///
    /// # Returns: `&str`
    /// * Device handle (e.g., "fpga0")
    fn device_handle(&self) -> &str {
        &self.device_handle
    }

    /// Read the current FPGA state from sysfs.
    ///
    /// Reads `/sys/class/fpga_manager/<device>/state` and returns the state string
    /// with trailing newlines removed. Common states include "operating", "unknown",
    /// or a string representing an error state.
    ///
    /// # Returns: `Result<String, FpgadError>`
    /// * `Ok(String)` - Current state (newlines trimmed)
    /// * `Err(FpgadError::IORead)` - Failed to read state file
    fn state(&self) -> Result<String, FpgadError> {
        let state_path = Path::new(config::FPGA_MANAGERS_DIR)
            .join(self.device_handle.clone())
            .join("state");
        trace!("reading {state_path:?}");
        fs_read(&state_path).map(|s| s.trim_end_matches('\n').to_string())
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

        match self.state() {
            Ok(state) => match state.as_str() {
                "operating" => {
                    info!(
                        "{}'s state is 'operating' after writing flags.",
                        self.device_handle
                    )
                }
                _ => {
                    warn!(
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

    /// Load a bitstream firmware file directly to the FPGA.
    ///
    /// Writes the firmware filename (relative to the kernel firmware search path) to
    /// `/sys/class/fpga_manager/<device>/firmware`. This triggers the kernel to load
    /// and program the bitstream. After writing, the method verifies the FPGA enters
    /// the "operating" state.
    ///
    /// # Arguments
    ///
    /// * `bitstream_path_rel` - Path to bitstream file relative to firmware search path
    ///
    /// # Returns: `Result<(), FpgadError>`
    /// * `Ok(())` - Bitstream loaded and FPGA is operating
    /// * `Err(FpgadError::IOWrite)` - Failed to write firmware file
    /// * `Err(FpgadError::FPGAState)` - FPGA not in "operating" state after loading
    ///
    /// # Note
    ///
    /// This method can be used to manually load firmware when an overlay doesn't
    /// trigger automatic loading. Always load firmware before applying overlays.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use daemon::platforms::universal_components::universal_fpga::UniversalFPGA;
    /// # use daemon::platforms::platform::Fpga;
    /// # use std::path::Path;
    /// # fn example(fpga: &UniversalFPGA) -> Result<(), daemon::error::FpgadError> {
    /// fpga.load_firmware(Path::new("design.bit.bin"))?;
    /// println!("Bitstream loaded successfully");
    /// # Ok(())
    /// # }
    /// ```
    fn load_firmware(&self, bitstream_path_rel: &Path) -> Result<(), FpgadError> {
        let control_path = Path::new(config::FPGA_MANAGERS_DIR)
            .join(self.device_handle())
            .join("firmware");
        fs_write(&control_path, false, bitstream_path_rel.to_string_lossy())?;
        self.assert_state()
    }
}
