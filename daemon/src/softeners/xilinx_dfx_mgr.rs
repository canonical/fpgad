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

use std::env;
use std::path::Path;
use std::sync::OnceLock;

use crate::error::FpgadError;
use crate::platforms::platform::{Fpga, OverlayHandler, Platform};
use crate::softeners::error::FpgadSoftenerError;
use crate::softeners::xilinx_dfx_mgr_fpga::XilinxDfxMgrFPGA;
use crate::softeners::xilinx_dfx_mgr_overlay_handler::XilinxDfxMgrOverlayHandler;
use fpgad_macros::platform;
use log::trace;

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
#[platform(compat_string = "xlnx,zynqmp-pcap-fpga,versal-fpga,zynq-devcfg-1.0")]
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
        // TODO(artie): perhaps this should warn the user that they are using a dfx-mgr capable
        // system but are not currently using that backend - warn that this is not suitable for
        // versal
        Ok(self
            .fpga
            .get_or_init(|| XilinxDfxMgrFPGA::new(device_handle)))
    }

    fn overlay_handler(&self, overlay_handle: &str) -> Result<&dyn OverlayHandler, FpgadError> {
        // TODO(artie): perhaps this should warn the user that they are using a dfx-mgr capable
        // system but are not currently using that backend - warn that this is not suitable for
        // versal
        Ok(self
            .overlay_handler
            .get_or_init(|| XilinxDfxMgrOverlayHandler::new(overlay_handle)))
    }

    fn status_message(&self) -> Result<String, FpgadError> {
        Ok(list_package()?)
    }

    fn platform_compat_string(&self) -> String {
        "xlnx,zynqmp-pcap-fpga,versal-fpga,zynq-devcfg-1.0".into()
    }
}

/// List locally downloaded accelerator packages
#[allow(dead_code)]
pub fn list_package() -> Result<String, FpgadSoftenerError> {
    run_dfx_mgr(&["-listPackage"])
}

/// Load the provided accelerator package
#[allow(dead_code)]
pub fn load(accel_name: &str) -> Result<String, FpgadSoftenerError> {
    run_dfx_mgr(&["-load", accel_name])
}

/// Unload package previously programmed
#[allow(dead_code)]
pub fn remove(slot_handle: Option<&str>) -> Result<String, FpgadSoftenerError> {
    match slot_handle {
        Some(slot_handle) => Ok(run_dfx_mgr(&["-remove", slot_handle])?),
        None => Ok(run_dfx_mgr(&["-remove"])?),
    }
}

/// List accelerator UIOs
#[allow(dead_code)]
pub fn list_uio(slot: Option<u32>, uio_name: Option<&str>) -> Result<String, FpgadSoftenerError> {
    let mut args = vec!["-listUIO"];
    if let Some(name) = uio_name {
        args.push(name);
    }
    if let Some(slot) = slot {
        let s_slot = slot.to_string();
        args.push(&s_slot);
        run_dfx_mgr(&args)
    } else {
        run_dfx_mgr(&args)
    }
}

/// List inter-RM buffer info
#[allow(dead_code)]
pub fn list_irbuf(slot: Option<u32>) -> Result<String, FpgadSoftenerError> {
    let mut args = vec!["-listIRbuf"];
    if let Some(slot) = slot {
        let s_slot = slot.to_string();
        args.push(s_slot.as_str());
        run_dfx_mgr(&args)
    } else {
        run_dfx_mgr(&args)
    }
}

/// Set RM stream from slot a to b
#[allow(dead_code)]
pub fn set_irbuf(a: u32, b: u32) -> Result<String, FpgadSoftenerError> {
    run_dfx_mgr(&["-setIRbuf", &format!("{a},{b}")])
}

/// Allocate buffer of size and return its DMA fd and pa
#[allow(dead_code)]
pub fn alloc_buffer(size: u64) -> Result<String, FpgadSoftenerError> {
    run_dfx_mgr(&["-allocBuffer", &size.to_string()])
}

/// Free buffer with physical address pa in decimal
#[allow(dead_code)]
pub fn free_buffer(pa: u64) -> Result<String, FpgadSoftenerError> {
    run_dfx_mgr(&["-freeBuffer", &pa.to_string()])
}

/// Send ip device FD's over socket
#[allow(dead_code)]
pub fn get_fds(slot: u32) -> Result<String, FpgadSoftenerError> {
    run_dfx_mgr(&["-getFDs", &slot.to_string()])
}

/// Get RM info
#[allow(dead_code)]
pub fn get_rm_info() -> Result<String, FpgadSoftenerError> {
    run_dfx_mgr(&["-getRMInfo"])
}

/// Get Shell FD
#[allow(dead_code)]
pub fn get_shell_fd() -> Result<String, FpgadSoftenerError> {
    run_dfx_mgr(&["-getShellFD"])
}

/// Get Clock FD
#[allow(dead_code)]
pub fn get_clock_fd() -> Result<String, FpgadSoftenerError> {
    run_dfx_mgr(&["-getClockFD"])
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
    run_dfx_mgr(&["-b", path_str])
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
#[allow(dead_code)]
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

    run_dfx_mgr(&["-o", dtbo_str, "-b", bitstream_str])
}

/// Helper to run the dfx-mgr-client binary with arguments
fn run_dfx_mgr(args: &[&str]) -> Result<String, FpgadSoftenerError> {
    let snap_env = env::var("SNAP").map_err(|e| {
        FpgadSoftenerError::DfxMgr(format!(
            "Cannot find dfx-mgr because $SNAP not specified in environment: {e}"
        ))
    })?;

    let dfx_mgr_client_path = format!("{}/usr/bin/dfx-mgr-client", snap_env);
    trace!("Calling dfx-mgr with args {:#?}", args);
    let output = std::process::Command::new(&dfx_mgr_client_path)
        .args(args)
        .output()
        .map_err(|e| {
            FpgadSoftenerError::DfxMgr(format!("dfx-mgr-client failed to produce output: {e}"))
        })?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(FpgadSoftenerError::DfxMgr(format!(
            "dfx-mgr-client failed. Exit status: {}\nStdout:\n{:#?}\nStderr:\n{:#?}",
            output.status, output.status, output.stderr
        )))
    }
}
