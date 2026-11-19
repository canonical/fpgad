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

//! Xilinx DFX Manager platform implementation.
//!
//! This module provides the Xilinx DFX Manager (dfx-mgr) platform, which is a vendor-specific
//! "softener" implementation for managing Xilinx FPGA devices. It wraps the Xilinx dfx-mgr-client
//! command-line tool to provide enhanced functionality for Xilinx devices including:
//! - Dynamic function exchange (DFX) / partial reconfiguration
//! - Accelerator package management
//! - Multi-slot FPGA management
//! - UIO (User I/O) interface management
//! - Inter-region buffer management
//!
//! # Platform Support
//!
//! This platform registers itself for Xilinx device compatibility strings:
//! - `xlnx,zynqmp-pcap-fpga` - Zynq UltraScale+ MPSoC
//! - `versal-fpga` - Versal ACAP devices
//! - `zynq-devcfg-1.0` - Zynq-7000 devices
//!
//! # DFX Manager Integration
//!
//! The platform communicates with the dfx-mgrd daemon (started via snap daemon wrapper)
//! through the dfx-mgr-client CLI tool. The dfx-mgr-client binary must be available at
//! `$SNAP/usr/bin/dfx-mgr-client`.
//!
//! # Architecture
//!
//! The platform uses lazy initialization via `OnceLock` to create component instances:
//! - [`XilinxDfxMgrFPGA`] - Manages FPGA device operations via dfx-mgr
//! - [`XilinxDfxMgrOverlayHandler`] - Manages overlay operations with bitstream coordination
//!
//! # Feature Flag
//!
//! This module is only compiled when the `xilinx-dfx-mgr` feature is enabled.
//!
//! # Examples
//!
//! ```rust,no_run
//! # use daemon::platforms::platform::platform_for_known_platform;
//! # fn example() -> Result<(), daemon::error::FpgadError> {
//! let platform = platform_for_known_platform("xlnx,zynqmp-pcap-fpga")?;
//! let fpga = platform.fpga("fpga0")?;
//! # Ok(())
//! # }
//! ```

use std::path::Path;
use std::sync::OnceLock;

use crate::error::FpgadError;
use crate::platforms::platform::{Fpga, OverlayHandler, Platform};
use crate::softeners::error::FpgadSoftenerError;
use fpgad_macros::platform;
use log::{info, trace};
use xilinx_dfx_mgr_fpga::XilinxDfxMgrFPGA;
use xilinx_dfx_mgr_helpers::run_dfx_mgr;
use xilinx_dfx_mgr_overlay_handler::XilinxDfxMgrOverlayHandler;

// Marked as public so that the members are published in docs
pub mod xilinx_dfx_mgr_fpga;
pub mod xilinx_dfx_mgr_helpers;
pub mod xilinx_dfx_mgr_overlay_handler;

/// Xilinx DFX Manager platform implementation for managing Xilinx FPGA devices.
///
/// This struct provides the platform implementation for Xilinx devices using the
/// dfx-mgr backend. It uses lazy initialization to create FPGA and overlay handler
/// instances on first access.
///
/// The `#[platform]` macro automatically registers this platform with the Xilinx-specific
/// compatibility strings, making it available for matching against Xilinx device tree
/// compatible properties.
///
/// # Fields
///
/// * `fpga` - Lazily initialized Xilinx FPGA device instance
/// * `overlay_handler` - Lazily initialized Xilinx overlay handler instance
///
/// # Thread Safety
///
/// This struct is thread-safe thanks to `OnceLock`, which ensures that initialization
/// happens exactly once even with concurrent access.
#[platform(compat_string = "xlnx,zynqmp-pcap-fpga,versal-fpga,zynq-devcfg-1.0,dfx-mgr,softener")]
pub struct XilinxDfxMgrPlatform {
    fpga: OnceLock<XilinxDfxMgrFPGA>,
    overlay_handler: OnceLock<XilinxDfxMgrOverlayHandler>,
}

impl Default for XilinxDfxMgrPlatform {
    fn default() -> Self {
        Self::new()
    }
}

impl XilinxDfxMgrPlatform {
    /// Create a new Xilinx DFX Manager platform instance.
    ///
    /// Creates an empty platform with uninitialized FPGA and overlay handler instances.
    /// The actual components will be lazily initialized on first access through the
    /// [`Platform`] trait methods.
    ///
    /// # Returns: `Self`
    /// * New XilinxDfxMgrPlatform instance ready for use
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use daemon::softeners::xilinx_dfx_mgr::XilinxDfxMgrPlatform;
    ///
    /// let platform = XilinxDfxMgrPlatform::new();
    /// ```
    pub fn new() -> Self {
        trace!("creating new XilinxDfxMgrPlatform");
        XilinxDfxMgrPlatform {
            fpga: OnceLock::new(),
            overlay_handler: OnceLock::new(),
        }
    }
}

impl Platform for XilinxDfxMgrPlatform {
    fn fpga(&self, device_handle: &str) -> Result<&dyn Fpga, FpgadError> {
        Ok(self
            .fpga
            .get_or_init(|| XilinxDfxMgrFPGA::new(device_handle)))
    }

    fn overlay_handler(&self, overlay_handle: &str) -> Result<&dyn OverlayHandler, FpgadError> {
        Ok(self
            .overlay_handler
            .get_or_init(|| XilinxDfxMgrOverlayHandler::new(overlay_handle)))
    }

    fn status_message(&self) -> Result<String, FpgadError> {
        Ok(list_package()?)
    }

    fn platform_compat_string(&self) -> String {
        Self::COMPAT_STRING.into()
    }

    /// Check if the Xilinx DFX Manager platform is available on this system.
    ///
    /// This function checks whether the dfx-mgr-client binary is available, either in the
    /// snap component location (`$SNAP_COMPONENTS/dfx-mgr/usr/bin/dfx-mgr-client`) or
    /// the system location (`/usr/bin/dfx-mgr-client`).
    ///
    /// # Returns
    /// * `true` - dfx-mgr-client binary is available and the platform can be used
    /// * `false` - dfx-mgr-client is not found, platform will not be available
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use daemon::softeners::xilinx_dfx_mgr::XilinxDfxMgrPlatform;
    ///
    /// if XilinxDfxMgrPlatform::is_available() {
    ///     println!("DFX Manager is available");
    /// } else {
    ///     println!("DFX Manager not found, using fallback platform");
    /// }
    /// ```
    fn is_available(&self) -> bool {
        let available = xilinx_dfx_mgr_helpers::get_dfx_mgr_client_path().is_ok();

        if !available {
            info!(
                "The dfx-mgr-client binary was not found and therefore Xilinx DFX Manager softener isn't available. \
                If you'd like to use DFX Manager features, you can install support via: \
                `sudo snap install fpgad+dfx-mgr.comp --dangerous` or `sudo apt install dfx-mgr`. \
                For now, this will probably use the xlnx-sys platform as fallback."
            );
        }

        available
    }
}

/// List locally downloaded accelerator packages
pub fn list_package() -> Result<String, FpgadSoftenerError> {
    run_dfx_mgr(&["-listPackage"])
}

/// Load the provided accelerator package
#[allow(dead_code)]
pub fn load(accel_name: &str) -> Result<String, FpgadSoftenerError> {
    run_dfx_mgr(&["-load", accel_name])
}

/// Unload package `slot_handle`, or every loaded package when it is `None`.
#[allow(dead_code)]
pub fn remove(slot_handle: Option<&str>) -> Result<String, FpgadSoftenerError> {
    match slot_handle {
        Some(slot_handle) => Ok(run_dfx_mgr(&["-unload", slot_handle])?),
        None => remove_all_loaded(),
    }
}

fn remove_all_loaded() -> Result<String, FpgadSoftenerError> {
    remove_loaded_with(list_package, |id| run_dfx_mgr(&["-unload", id]))
}

/// Re-lists before each unload, since package IDs can change after one.
fn remove_loaded_with(
    list: impl Fn() -> Result<String, FpgadSoftenerError>,
    unload: impl Fn(&str) -> Result<String, FpgadSoftenerError>,
) -> Result<String, FpgadSoftenerError> {
    let loaded_at_start = loaded_package_ids(&list()?).len();
    if loaded_at_start == 0 {
        return Ok(String::from("No loaded packages to unload."));
    }
    let mut output = String::new();
    for _ in 0..loaded_at_start {
        match loaded_package_ids(&list()?).first() {
            Some(id) => output.push_str(&unload(id)?),
            None => break,
        }
    }
    Ok(output)
}

fn loaded_package_ids(listing: &str) -> Vec<String> {
    listing
        .lines()
        .filter_map(|line| {
            let mut tokens = line.split_whitespace();
            let id = tokens.next()?;
            id.parse::<u32>().ok()?;
            let slot_loc = tokens.nth(2)?;
            let loaded = line.contains(".bit.bin") || line.contains(".dtbo") || slot_loc != "-1";
            loaded.then(|| id.to_string())
        })
        .collect()
}

/// Load a bitstream file using dfx-mgr
///
/// # Arguments
///
/// * `bitstream_path` - Path to the bitstream file to load
///
/// # Returns: `Result<String, FpgadSoftenerError>`
/// * `Ok(String)` - Output from dfx-mgr-client
/// * `Err(FpgadSoftenerError::DfxMgr)` - Path contains invalid UTF-8 or dfx-mgr-client failed
pub fn load_bitstream(bitstream_path: &Path) -> Result<String, FpgadSoftenerError> {
    let path_str = bitstream_path.to_str().ok_or_else(|| {
        FpgadSoftenerError::DfxMgr(format!(
            "Bitstream path contains invalid UTF-8: {}",
            bitstream_path.display()
        ))
    })?;
    run_dfx_mgr(&["-b", path_str, "-f", "Full"])
}

/// Load an overlay with bitstream using dfx-mgr
///
/// # Arguments
///
/// * `bitstream_path` - Path to the bitstream file
/// * `dtbo_path` - Path to the device tree overlay file
///
/// # Returns: `Result<String, FpgadSoftenerError>`
/// * `Ok(String)` - Output from dfx-mgr-client
/// * `Err(FpgadSoftenerError::DfxMgr)` - Path contains invalid UTF-8 or dfx-mgr-client failed
pub fn load_overlay(bitstream_path: &Path, dtbo_path: &Path) -> Result<String, FpgadSoftenerError> {
    let bitstream_str = bitstream_path.to_str().ok_or_else(|| {
        FpgadSoftenerError::DfxMgr(format!(
            "Bitstream path contains invalid UTF-8: {}",
            bitstream_path.display()
        ))
    })?;

    let dtbo_str = dtbo_path.to_str().ok_or_else(|| {
        FpgadSoftenerError::DfxMgr(format!(
            "DTBO path contains invalid UTF-8: {}",
            dtbo_path.display()
        ))
    })?;

    run_dfx_mgr(&["-b", bitstream_str, "-f", "Full", "-o", dtbo_str])
}

#[cfg(test)]
mod tests {
    use super::{loaded_package_ids, remove_loaded_with};
    use crate::softeners::error::FpgadSoftenerError;
    use std::cell::RefCell;

    fn fake_listing(state: &[(bool, &str)]) -> String {
        let mut listing = String::from("ID accelType   Base        slotLoc Accelerator\n");
        for (i, (loaded, name)) in state.iter().enumerate() {
            let slot = if *loaded { "0" } else { "-1" };
            listing.push_str(&format!(
                "{:>2} XRT_FLAT    -           {slot:<7} {name}\n",
                i + 1
            ));
        }
        listing
    }

    #[test]
    fn unloads_each_loaded_package_using_fresh_ids() {
        let state = RefCell::new(vec![
            (false, "k24-starter-kits"),
            (false, "k26-starter-kits"),
            (true, "a.bit.bin"),
            (true, "b.bit.bin"),
        ]);
        let unloaded = RefCell::new(Vec::new());
        let result = remove_loaded_with(
            || Ok(fake_listing(&state.borrow())),
            |id| {
                unloaded.borrow_mut().push(id.to_string());
                let index: usize = id.parse().expect("numeric id");
                let mut packages = state.borrow_mut();
                if index == 0 || index > packages.len() || !packages[index - 1].0 {
                    return Err(FpgadSoftenerError::DfxMgr(format!("ID {id} is not loaded")));
                }
                packages.remove(index - 1);
                Ok(format!("unload ID {id} returns: 0 (Ok)\n"))
            },
        );
        assert!(result.is_ok(), "unexpected error: {result:?}");
        assert_eq!(unloaded.into_inner(), vec!["3", "3"]);
        assert_eq!(state.borrow().len(), 2);
    }

    #[test]
    fn nothing_loaded_sends_no_unload() {
        let unloaded = RefCell::new(0);
        let result = remove_loaded_with(
            || Ok(fake_listing(&[(false, "k24-starter-kits")])),
            |_| {
                *unloaded.borrow_mut() += 1;
                Ok(String::new())
            },
        );
        assert_eq!(result.unwrap(), "No loaded packages to unload.");
        assert_eq!(unloaded.into_inner(), 0);
    }

    #[test]
    fn failed_unload_stops_and_returns_the_error() {
        let calls = RefCell::new(0);
        let result = remove_loaded_with(
            || Ok(fake_listing(&[(true, "a.bit.bin"), (true, "b.bit.bin")])),
            |_| {
                *calls.borrow_mut() += 1;
                Err(FpgadSoftenerError::DfxMgr("unload failed".to_string()))
            },
        );
        assert!(result.is_err());
        assert_eq!(calls.into_inner(), 1);
    }

    const LISTING: &str = "\
WARNING: Package IDs have changed since last -listPackage.\n\
ID accelType   Base        slotLoc Accelerator\n\
-- ----------- ----------- ------- -----------\n\
 1 XRT_FLAT    k24-star... -1      k24-starter-kits\n\
 2 XRT_FLAT    k26-star... -1      k26-starter-kits\n\
 3 -           -           -       k26-starter-kits.bit.bin\n";

    #[test]
    fn loaded_package_is_found_by_bit_bin_name() {
        assert_eq!(loaded_package_ids(LISTING), vec!["3".to_string()]);
    }

    #[test]
    fn package_with_a_slot_location_is_loaded() {
        let listing = " 4 SIHA_PL_DFX static      2       rp0rm0\n";
        assert_eq!(loaded_package_ids(listing), vec!["4".to_string()]);
    }

    #[test]
    fn dtbo_package_is_loaded() {
        let listing = " 5 -           -           -       overlay.dtbo\n";
        assert_eq!(loaded_package_ids(listing), vec!["5".to_string()]);
    }

    #[test]
    fn listing_with_nothing_loaded_returns_no_ids() {
        let listing = " 1 XRT_FLAT    k24-star... -1      k24-starter-kits\n";
        assert!(loaded_package_ids(listing).is_empty());
    }

    #[test]
    fn header_and_empty_input_return_no_ids() {
        assert!(loaded_package_ids("ID accelType Base slotLoc Accelerator\n").is_empty());
        assert!(loaded_package_ids("").is_empty());
    }
}
