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
use crate::system_io::{fs_read, fs_write};
use log::trace;
use std::path::Path;

/// Write a specified path to the systems firmware search path.
/// See [these kernel docs](https://docs.kernel.org/driver-api/firmware/fw_search_path.html)
/// for more information on the process.
///
/// # Arguments
///
/// * `new_path`: path inside which firmware can be found
///
/// # Returns: `Result<(), FpgadError>`
/// * `()` on success
/// * `FpgadError::IOWrite` (or similar IO error) if writing fails for any reason.
///
/// # Examples
///
/// ```rust,no_run
/// assert!(write_firmware_source_dir("/lib/firmware/my_firmware_dir").is_ok());
/// ```
pub fn write_firmware_source_dir(new_path: &str) -> Result<(), FpgadError> {
    trace!(
        "Writing fw prefix {} to {}",
        new_path,
        config::FIRMWARE_LOC_CONTROL_PATH
    );
    let fw_lookup_override = Path::new(config::FIRMWARE_LOC_CONTROL_PATH);
    fs_write(fw_lookup_override, false, new_path)
}

#[allow(dead_code)]
/// Read the currently specified firmware search path.
/// See [these kernel docs](https://docs.kernel.org/driver-api/firmware/fw_search_path.html)
/// for more information on the process.
///
/// # Returns: `Result<String, FpgadError>`
/// * `String` - The contents of the firmware search path variable.
/// * `FpgadError::IOWrite` (or similar IO error) if writing fails for any reason.
///
/// # Examples
///
/// ```rust,no_run
/// let search_path_str = read_firmware_source_dir()?;
/// assert_eq!(search_path_str, "/lib/firmware/my_firmware_dir");
/// ```
pub fn read_firmware_source_dir() -> Result<String, FpgadError> {
    trace!(
        "Reading fw prefix from {}",
        config::FIRMWARE_LOC_CONTROL_PATH
    );
    let fw_lookup_override = Path::new(config::FIRMWARE_LOC_CONTROL_PATH);
    fs_read(fw_lookup_override)
}
