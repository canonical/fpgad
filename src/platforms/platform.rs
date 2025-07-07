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
use crate::platforms::universal::UniversalPlatform;
use crate::system_io::{fs_read, fs_read_dir};
use log::{error, info, trace, warn};
use std::path::Path;

#[derive(Clone, Copy)]
enum PlatformType {
    Universal,
    ZynqMP,
    Versal,
}

const PLATFORM_SUBSTRINGS: &[(&str, PlatformType)] = &[
    ("zynqmp", PlatformType::ZynqMP),
    ("versal", PlatformType::Versal),
];

/// Scans /sys/class/fpga_manager/ for all present device nodes and returns a Vec of their handles
pub fn list_fpga_managers() -> Result<Vec<String>, FpgadError> {
    let prefix = config::SYSFS_PREFIX;
    if prefix.is_empty() {
        return Ok(vec!["".to_string()]);
    };
    fs_read_dir(prefix.as_ref())
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
    #[allow(dead_code)]
    /// get the device handle for this fpga device
    fn device_handle(&self) -> &str;
    /// get the state of the fpga device
    fn state(&self) -> Result<String, FpgadError>;
    /// get the current flags of the fpga device
    fn flags(&self) -> Result<isize, FpgadError>;
    /// attempt to set the flags of an fpga device
    fn set_flags(&self, flags: isize) -> Result<(), FpgadError>;
    #[allow(dead_code)]
    /// Directly load the firmware stored in bitstream_path to the device
    fn load_firmware(&self, bitstream_path: &Path) -> Result<(), FpgadError>;
}

pub trait OverlayHandler {
    /// Applies an overlay to an already existing overlayfs dir,
    /// which may or may not also write a bitstream to an fpga device.
    fn apply_overlay(&self, source_path: &Path) -> Result<(), FpgadError>;
    /// Removes an overlayfs directory from the configfs.
    fn remove_overlay(&self) -> Result<(), FpgadError>;
    /// Gets the required fpga flags from an overlay file
    #[allow(dead_code)]
    fn required_flags(&self) -> Result<isize, FpgadError>;
    /// gets the overlay application status
    fn status(&self) -> Result<String, FpgadError>;
    fn overlay_fs_path(&self) -> Result<&Path, FpgadError>;
}

fn match_platform_string(platform_string: &str) -> PlatformType {
    for (substr, platform) in PLATFORM_SUBSTRINGS {
        if platform_string.contains(substr) {
            trace!("Found '{substr}'");
            return *platform;
        }
    }
    warn!(
        "FPGAd could not match {platform_string} to a known platform.\
    Using 'Universal'"
    );
    PlatformType::Universal
}

pub fn read_compatible_string(device_handle: &str) -> Result<String, FpgadError> {
    let compat_string = match fs_read(
        &Path::new(config::SYSFS_PREFIX)
            .join(device_handle)
            .join("of_node/compatible"),
    ) {
        Err(e) => {
            error!(
                "Failed to read platform from {device_handle:?}: {e}\n\
                Universal will be used as platform type.",
            );
            return Ok("universal".to_string());
        }
        Ok(s) => {
            trace!("{s}");
            // often driver virtual files contain null terminated strings instead of EOF terminated.
            s.trim_end_matches('\0').to_string()
        }
    };
    Ok(compat_string)
}

fn discover_platform_type(device_handle: &str) -> Result<PlatformType, FpgadError> {
    let compat_string = read_compatible_string(device_handle)?;
    trace!("Found compatibility string: '{compat_string}'");
    Ok(match_platform_string(&compat_string))
}

pub fn new_platform(platform_type: PlatformType) -> impl Platform {
    match platform_type {
        PlatformType::Universal => {
            info!("Using platform: Universal");
            UniversalPlatform::new()
        }
        PlatformType::ZynqMP => {
            warn!("ZynqMP not implemented yet: using Universal");
            UniversalPlatform::new()
        }
        PlatformType::Versal => {
            warn!("Versal not implemented yet: using Universal");
            UniversalPlatform::new()
        }
    }
}

pub fn platform_for_device(device_handle: &str) -> Result<impl Platform, FpgadError> {
    Ok(new_platform(discover_platform_type(device_handle)?))
}

pub fn platform_for_known_platform(platform_string: &str) -> impl Platform {
    new_platform(match_platform_string(platform_string))
}

pub trait Platform {
    #[allow(dead_code)]
    /// gets the name of the Platform type e.g. Universal or ZynqMP
    fn platform_type(&self) -> &str;
    /// creates and inits an Fpga if not present otherwise gets the instance
    fn fpga(&self, device_handle: &str) -> Result<&impl Fpga, FpgadError>;
    /// creates and inits an OverlayHandler if not present otherwise gets the instance
    fn overlay_handler(&self, overlay_handle: &str) -> Result<&impl OverlayHandler, FpgadError>;
}
