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
use std::path::Path;

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
    fn name(&self) -> &str;
    fn assert_state(&self) -> Result<(), FpgadError>;
    fn get_state(&self) -> Result<String, FpgadError>;
    fn get_flags(&self) -> Result<isize, FpgadError>;
    fn set_flags(&self, flags: isize) -> Result<(), FpgadError>;
    #[allow(dead_code)]
    fn load_firmware(&self, bitstream_path: &Path) -> Result<(), FpgadError>;
}

pub trait OverlayHandler {
    fn prepare_for_load(&mut self) -> Result<(), FpgadError>;
    fn apply_overlay(&self) -> Result<(), FpgadError>;
    fn remove_overlay(&mut self) -> Result<(), FpgadError>;
    fn get_required_flags(&self) -> Result<isize, FpgadError>;
    fn get_status(&self) -> Result<String, FpgadError>;
}

pub trait Platform {
    #[allow(dead_code)]
    fn name(&self) -> &str;
    fn fpga(&mut self, name: &str) -> &impl Fpga;
    #[allow(dead_code)]
    fn overlay_handler(&mut self, overlay_source_path: &Path) -> &impl OverlayHandler;
}
