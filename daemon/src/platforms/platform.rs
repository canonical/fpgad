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
use log::{trace, warn};
use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::{Mutex, OnceLock};

type PlatformConstructor = fn() -> Box<dyn Platform>;

pub static PLATFORM_REGISTRY: OnceLock<Mutex<HashMap<&'static str, PlatformConstructor>>> =
    OnceLock::new();

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
    fn flags(&self) -> Result<u32, FpgadError>;
    /// attempt to set the flags of an fpga device
    fn set_flags(&self, flags: u32) -> Result<(), FpgadError>;
    #[allow(dead_code)]
    /// Directly load the firmware stored in bitstream_path to the device
    fn load_firmware(&self, bitstream_path_rel: &Path) -> Result<(), FpgadError>;
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

pub trait Platform: Any {
    /// creates and inits an Fpga if not present otherwise gets the instance
    fn fpga(&self, device_handle: &str) -> Result<&dyn Fpga, FpgadError>;
    /// creates and inits an OverlayHandler if not present otherwise gets the instance
    fn overlay_handler(&self, overlay_handle: &str) -> Result<&dyn OverlayHandler, FpgadError>;
}

fn match_platform_string(platform_string: &str) -> Result<Box<dyn Platform>, FpgadError> {
    let registry = PLATFORM_REGISTRY
        .get()
        .ok_or(FpgadError::Internal(String::from(
            "couldn't get PLATFORM_REGISTRY",
        )))?
        .lock()
        .map_err(|_| FpgadError::Internal(String::from("couldn't lock PLATFORM_REGISTRY")))?;

    for (compat_string, platform_constructor) in registry.iter() {
        let compat_set: HashSet<&str> = compat_string.split(',').collect();
        let compat_found = platform_string.split(',').all(|x| compat_set.contains(x));
        if compat_found {
            return Ok(platform_constructor());
        }
    }

    Err(FpgadError::Argument(format!(
        "FPGAd could not match {platform_string} to a known platform."
    )))
}

fn discover_platform(device_handle: &str) -> Result<Box<dyn Platform>, FpgadError> {
    let compat_string = read_compatible_string(device_handle)?;
    trace!("Found compatibility string: '{compat_string}'");

    Ok(match_platform_string(&compat_string).unwrap_or({
        warn!("{compat_string} not supported. Defaulting to Universal platform.");
        Box::new(UniversalPlatform::new())
    }))
}

pub fn read_compatible_string(device_handle: &str) -> Result<String, FpgadError> {
    let compat_string = match fs_read(
        &Path::new(config::FPGA_MANAGERS_DIR)
            .join(device_handle)
            .join("of_node/compatible"),
    ) {
        Err(e) => {
            return Err(FpgadError::Argument(format!(
                "Failed to read platform from {device_handle:?}: {e}"
            )));
        }
        Ok(s) => {
            // often driver virtual files contain null terminated strings instead of EOF terminated.
            s.trim_end_matches('\0').to_string()
        }
    };
    Ok(compat_string)
}

pub fn platform_from_compat_or_device(
    platform_string: &str,
    device_handle: &str,
) -> Result<Box<dyn Platform>, FpgadError> {
    match platform_string.is_empty() {
        true => discover_platform(device_handle),
        false => platform_for_known_platform(platform_string),
    }
}

pub fn platform_for_known_platform(platform_string: &str) -> Result<Box<dyn Platform>, FpgadError> {
    match_platform_string(platform_string)
}

pub fn init_platform_registry() -> Mutex<HashMap<&'static str, PlatformConstructor>> {
    Mutex::new(HashMap::new())
}

pub fn register_platform(compatible: &'static str, constructor: PlatformConstructor) {
    let mut registry = PLATFORM_REGISTRY
        .get_or_init(init_platform_registry)
        .lock()
        .expect("couldnt get PLATFORM_REGISTRY");

    registry.insert(compatible, constructor);
}

/// Scans /sys/class/fpga_manager/ for all present device nodes and returns a Vec of their handles
pub fn list_fpga_managers() -> Result<Vec<String>, FpgadError> {
    fs_read_dir(config::FPGA_MANAGERS_DIR.as_ref())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::softeners::xilinx_dfx_mgr::XilinxDfxMgrPlatform;
    use std::any::Any;

    fn setup_test_registry() {
        register_platform("xlnx,versal-fpga,zynqmp-pcap-fpga,zynq-devcfg-1.0", || {
            Box::new(XilinxDfxMgrPlatform::new())
        });
    }

    fn assert_is_xilinx_platform(platform: &dyn Platform) {
        let as_xilinx_platform = (platform as &dyn Any).downcast_ref::<XilinxDfxMgrPlatform>();
        assert!(
            as_xilinx_platform.is_some(),
            "The platform should be of type XilinxDfxMgrPlatform"
        );
    }

    #[test]
    fn test_match_platform_string_empty_string_fails() {
        setup_test_registry();
        let result = match_platform_string("");

        assert!(
            result.is_err(),
            "Empty string should fail to match any platform"
        );
    }

    #[test]
    fn test_match_platform_string_xlnx_succeeds() {
        setup_test_registry();
        let result = match_platform_string("xlnx");

        assert!(result.is_ok(), "xlnx should match successfully");
        let platform = result.unwrap();
        assert_is_xilinx_platform(platform.as_ref());
    }

    #[test]
    fn test_match_platform_string_partial_match_fails() {
        setup_test_registry();
        let result = match_platform_string("xlnx,pcap-");

        assert!(result.is_err(), "Partial match 'xlnx,pcap-' should fail");
    }

    #[test]
    fn test_match_platform_string_invalid_platform_fails() {
        setup_test_registry();
        let result = match_platform_string("invalid-platform");
        assert!(result.is_err(), "Invalid platform should fail to match");
    }

    #[test]
    fn test_match_platform_string_full_match_succeeds() {
        setup_test_registry();
        let result = match_platform_string("xlnx,zynqmp-pcap-fpga");

        assert!(result.is_ok(), "Full match should succeed");
        let platform = result.unwrap();
        assert_is_xilinx_platform(platform.as_ref());
    }

    #[test]
    fn test_match_platform_string_single_component_succeeds() {
        setup_test_registry();
        let result = match_platform_string("versal-fpga");

        assert!(
            result.is_ok(),
            "Single component 'versal-fpga' should succeed"
        );
        let platform = result.unwrap();
        assert_is_xilinx_platform(platform.as_ref());
    }

    #[test]
    fn test_match_platform_string_multiple_components_succeeds() {
        setup_test_registry();
        let result = match_platform_string("xlnx,versal-fpga,zynq-devcfg-1.0");

        assert!(result.is_ok(), "Multiple valid components should succeed");
        let platform = result.unwrap();
        assert_is_xilinx_platform(platform.as_ref());
    }

    #[test]
    fn test_match_platform_string_mixed_valid_invalid_fails() {
        setup_test_registry();
        let result = match_platform_string("xlnx,invalid-component");

        assert!(
            result.is_err(),
            "Mix of valid and invalid components should fail"
        );
    }

    #[test]
    fn test_match_platform_string_case_sensitive() {
        setup_test_registry();
        let result = match_platform_string("XLNX");

        assert!(
            result.is_err(),
            "Case sensitive matching should fail for uppercase"
        );
    }

    #[test]
    fn test_platform_type_assertion_methods() {
        setup_test_registry();
        let platform = match_platform_string("xlnx").unwrap();

        // Method 1: Using downcast_ref
        let as_xilinx = (platform.as_ref() as &dyn Any).downcast_ref::<XilinxDfxMgrPlatform>();
        assert!(as_xilinx.is_some(), "Downcast should succeed");

        // Method 2: Using type_id comparison
        let platform_any = platform.as_ref() as &dyn Any;
        assert!(
            platform_any.is::<XilinxDfxMgrPlatform>(),
            "Type ID check should succeed"
        );

        // Method 3: Using type_name (for debugging)
        let type_name = std::any::type_name_of_val(as_xilinx.unwrap());
        assert!(
            type_name.contains("XilinxDfxMgrPlatform"),
            "Type name({type_name}) should contain XilinxDfxMgrPlatform"
        );
    }
}
