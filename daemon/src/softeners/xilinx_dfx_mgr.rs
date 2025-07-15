use std::io;
use std::process::Command;
use std::sync::OnceLock;

use log::trace;

use crate::platforms::platform::{Platform, PlatformType};
use crate::platforms::universal_components::universal_fpga::UniversalFPGA;
use crate::platforms::universal_components::universal_overlay_handler::UniversalOverlayHandler;
use crate::softeners::error::FpgadSoftenerError;

pub struct XilinxDfxMgrPlatform {
    fpga: OnceLock<UniversalFPGA>,
    overlay_handler: OnceLock<UniversalOverlayHandler>,
}

impl XilinxDfxMgrPlatform {
    pub fn new() -> Self {
        trace!("creating new XilinxDfxMgrPlatform");
        XilinxDfxMgrPlatform {
            fpga: OnceLock::new(),
            overlay_handler: OnceLock::new(),
        }
    }
}
impl Platform for XilinxDfxMgrPlatform {
    fn platform_type(&self) -> PlatformType {
        PlatformType::Xilinx
    }

    fn fpga(
        &self,
        device_handle: &str,
    ) -> Result<&dyn crate::platforms::platform::Fpga, crate::error::FpgadError> {
        Ok(self.fpga.get_or_init(|| UniversalFPGA::new(device_handle)))
    }

    fn overlay_handler(
        &self,
        overlay_handle: &str,
    ) -> Result<&(dyn crate::platforms::platform::OverlayHandler), crate::error::FpgadError> {
        Ok(self
            .overlay_handler
            .get_or_init(|| UniversalOverlayHandler::new(overlay_handle)))
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
pub fn remove(slot: u32) -> Result<String, FpgadSoftenerError> {
    run_dfx_mgr(&["-remove", &slot.to_string()])
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

/// Helper to run the dfx-mgr-client binary with arguments
fn run_dfx_mgr(args: &[&str]) -> Result<String, FpgadSoftenerError> {
    let output = Command::new("sudo")
        .arg("dfx-mgr-client")
        .args(args)
        .output()
        .map_err(FpgadSoftenerError::DfxMgr)?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(FpgadSoftenerError::DfxMgr(io::Error::other(
            String::from_utf8_lossy(&output.stderr).to_string(),
        )))
    }
}
