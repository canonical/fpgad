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

use crate::error::Error;

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
    fn state(&self) -> Result<String, Error>;
    fn load_bitstream(&self) -> bool;
    fn unload_bitstream(&self) -> bool;
}

pub trait OverlayHandler {
    fn apply_devicetree(&self) -> bool;
    fn unapply_devicetree(&self) -> bool;
}

pub trait Platform {
    fn name(&self) -> &str;
    fn fpga(&mut self, name: &str) -> &impl Fpga;
    fn overlay_handler(&self) -> &impl OverlayHandler;
}
