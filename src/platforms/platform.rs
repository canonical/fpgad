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
use std::path::{Path, PathBuf};

/// Scans /sys/class/fpga_manager/ for all present device nodes and returns a Vec of their handles
pub fn list_fpga_managers() -> Vec<String> {
    std::fs::read_dir("/sys/class/fpga_manager")
        .map(|iter| {
            iter.filter_map(Result::ok)
                .map(|entry| entry.file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default()
}

/// A sysfs map of an fpga in fpga_manager class.
/// See the example below (not all sysfs files are implemented as methods):
/// ubuntu@kria:~$ tree /sys/class/fpga_manager/fpga0
/// /sys/class/fpga_manager/fpga0
/// ├── device -> ../../../firmware:zynqmp-firmware:pcap
/// ├── firmware
/// ├── flags
/// ├── key
/// ├── name
/// ├── of_node -> ../../../../../../firmware/devicetree/base/firmware/zynqmp-firmware/pcap
/// ├── power
/// │   ├── async
/// │   ├── autosuspend_delay_ms
/// │   ├── control
/// │   ├── runtime_active_kids
/// │   ├── runtime_active_time
/// │   ├── runtime_enabled
/// │   ├── runtime_status
/// │   ├── runtime_suspended_time
/// │   └── runtime_usage
/// ├── state
/// ├── status
/// ├── subsystem -> ../../../../../../class/fpga_manager
/// └── uevent
///
pub trait Fpga {
    /// get the device handle for this fpga device
    fn device_handle(&self) -> &str;
    /// get the state of the fpga device
    fn get_state(&self) -> Result<String, FpgadError>;
    /// get the current flags of the fpga device
    fn get_flags(&self) -> Result<isize, FpgadError>;
    /// attempt to set the flags of an fpga device
    fn set_flags(&self, flags: isize) -> Result<(), FpgadError>;
    #[allow(dead_code)]
    /// Directly load the firmware stored in bitstream_path to the device
    fn load_firmware(&self, bitstream_path: &Path) -> Result<(), FpgadError>;
}

pub trait OverlayHandler {
    /// Applies an overlay, which may or may not also write a bitstream to an fpga device.
    fn apply_overlay(&self) -> Result<(), FpgadError>;
    /// Removes an overlay
    fn remove_overlay(&self) -> Result<(), FpgadError>;
    /// Gets the required fpga flags from an overlay file
    #[allow(dead_code)]
    fn get_required_flags(&self) -> Result<isize, FpgadError>;
    /// gets the overlay application status
    fn get_status(&self) -> Result<String, FpgadError>;
    /// internally stores the source path for the overlay to be applied/inspected
    fn set_source_path(&mut self, source_path: &Path) -> Result<(), FpgadError>;
    /// constructs the internal overlayfs path for a given overlay handle e.g. my_overlay_0
    fn set_overlay_fs_path(&mut self, overlay_handle: &str) -> Result<(), FpgadError>;
    fn get_overlay_fs_path(&self) -> Result<&PathBuf, FpgadError>;
    fn get_overlay_source_path(&self) -> Result<&PathBuf, FpgadError>;
}

pub trait Platform {
    #[allow(dead_code)]
    /// gets the name of the Platform type e.g. Universal or ZynqMP
    fn platform_type(&self) -> &str;
    /// creates and inits an Fpga if not present otherwise gets the instance
    fn fpga(&mut self, device_handle: &str) -> &impl Fpga;
    /// creates and inits an OverlayHandler if not present otherwise gets the instance
    fn overlay_handler(&mut self) -> &mut dyn OverlayHandler;
}
