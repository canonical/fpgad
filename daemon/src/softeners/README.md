# FPGAd Softeners

## Overview

Softeners are platform-specific extensions to FPGAd that provide integration with vendor-specific FPGA management tools
and frameworks. While FPGAd's core platform system handles standard Linux FPGA subsystem operations, softeners enable
fpgad to work with proprietary or vendor-specific tools that sit alongside or above the kernel FPGA subsystem.

> **Note on Naming**: FPGA devices are often referred to as "fabric" (as in "programmable logic fabric"), so naturally
> FPGAd provides "softeners" for the fabric 🧺.

### When to Use a Softener vs. a Platform

- **Use the XilinxSys Platform** when implementing direct interactions with the Linux FPGA subsystem (
  `/sys/class/fpga_manager`) and device tree overlays using standard kernel interfaces. The XilinxSys platform handles
  the standard kernel interface through generic sysfs operations.

- **Create a Softener Platform** when you need to integrate with vendor-specific userspace tools, SDKs, or frameworks
  that provide additional functionality beyond what the kernel exposes. Softeners typically wrap external binaries or
  libraries and implement custom `Fpga` and `OverlayHandler` types that use vendor-specific tools instead of generic
  sysfs operations.

## Architecture

Softeners extend platforms by providing vendor-specific functionality. A softener platform:

1. Implements the `Platform` trait (like any other platform)
2. Uses the `#[platform(compat_string = "...")]` macro to register itself
3. Provides custom `Fpga` and `OverlayHandler` implementations that wrap vendor tools
4. May provide additional vendor-specific functions beyond the standard `Platform` trait
5. Is conditionally compiled using Cargo feature flags

### Why Custom FPGA and OverlayHandler Implementations?

While you *could* reuse the XilinxSys platform's components (`XilinxSysFPGA` and `XilinxSysOverlayHandler`), softeners
exist specifically to provide vendor-specific functionality. Custom implementations allow you to:

- **Wrap vendor tools**: Instead of using generic sysfs operations, call vendor-specific binaries or libraries
- **Provide enhanced features**: Expose functionality not available through the standard Linux FPGA subsystem
- **Optimize performance**: Use vendor-optimized loading mechanisms or caching strategies
- **Handle vendor quirks**: Work around device-specific behaviors or limitations
- **Integrate with vendor ecosystems**: Connect with other vendor tools, licensing systems, or management frameworks

If you find yourself not needing any custom behavior, you probably don't need a softener platform at all - the XilinxSys
platform should suffice.

### Example: Xilinx DFX Manager

The `xilinx_dfx_mgr` softener integrates with AMD/Xilinx's `dfx-mgr` tool, which provides vendor specific optimizations
on top of the standard FPGA subsystem and, e.g., enables "partial reconfiguration" of the device, which isn't supported
by the XilinxSys platform. See [dfx-mgr on GitHub](https://github.com/Xilinx/dfx-mgr) for more information on dfx-mgr.

## Adding a New Softener Platform

### Step 1: Add a Cargo Feature

Edit `daemon/Cargo.toml` and add your softener as a feature:

```toml
[features]
default = ["softeners-all"]
softeners-all = ["softeners", "xilinx-dfx-mgr", "your-new-softener"]
softeners = []
xilinx-dfx-mgr = ["softeners"]
your-new-softener = ["softeners"]  # Add this line
```

**Important**: Each softener should depend on the `softeners` base feature.

### Step 2: Create the Softener Module Structure

Create your softener module directory and files:

1. Create directory: `daemon/src/softeners/your_softener_name/`
2. Create these files in that directory:
   - `your_softener_name_fpga.rs` - Custom FPGA implementation
   - `your_softener_name_overlay_handler.rs` - Custom overlay handler implementation
   - `your_softener_name_helpers.rs` (optional) - Helper functions
3. Create the main platform file: `daemon/src/softeners/your_softener_name.rs`

**Important**: While you *could* reuse the XilinxSys platform's components, softeners exist to provide vendor-specific
functionality, so you should implement your own FPGA and OverlayHandler types that wrap or integrate with your vendor's
tools.

#### Main Platform File: `daemon/src/softeners/your_softener_name.rs`

```rust
use std::sync::OnceLock;

use crate::platforms::platform::Platform;
use crate::softeners::error::FpgadSoftenerError;
use fpgad_macros::platform;
use your_softener_name_fpga::YourSoftenerFPGA;
use your_softener_name_overlay_handler::YourSoftenerOverlayHandler;

// Declare your sub-modules here (marked as public so they appear in documentation)
pub mod your_softener_name_fpga;
pub mod your_softener_name_overlay_handler;
pub mod your_softener_name_helpers;  // Optional

/// Your softener platform implementation.
///
/// This platform integrates with [vendor tool name] to provide [brief description].
///
/// # Compatibility
///
/// Compatible with devices matching: `your,compat-string`
#[platform(compat_string = "your,compat-string")]
pub struct YourSoftenerPlatform {
    fpga: OnceLock<YourSoftenerFPGA>,
    overlay_handler: OnceLock<YourSoftenerOverlayHandler>,
}

impl Default for YourSoftenerPlatform {
    fn default() -> Self {
        Self::new()
    }
}

impl YourSoftenerPlatform {
    pub fn new() -> Self {
        YourSoftenerPlatform {
            fpga: OnceLock::new(),
            overlay_handler: OnceLock::new(),
        }
    }
}

impl Platform for YourSoftenerPlatform {
    fn fpga(
        &self,
        device_handle: &str,
    ) -> Result<&dyn crate::platforms::platform::Fpga, crate::error::FpgadError> {
        Ok(self.fpga.get_or_init(|| YourSoftenerFPGA::new(device_handle)))
    }

    fn overlay_handler(
        &self,
        overlay_handle: &str,
    ) -> Result<&dyn crate::platforms::platform::OverlayHandler, crate::error::FpgadError> {
        Ok(self.overlay_handler.get_or_init(|| YourSoftenerOverlayHandler::new(overlay_handle)))
    }
}

// Add your vendor-specific functions below
// These are public functions that can be called by the daemon or exposed via DBus

/// Example vendor-specific function
pub fn do_vendor_thing(param: &str) -> Result<String, FpgadSoftenerError> {
    // vendor specific functionality, example later  
    todo!()
}
```

#### FPGA Implementation: `daemon/src/softeners/your_softener_name/your_softener_name_fpga.rs`

```rust
use crate::platforms::platform::Fpga;
use crate::error::FpgadError;
use std::process::Command;
use log::{debug, trace};

/// Your vendor-specific FPGA device implementation.
///
/// This struct wraps interactions with [vendor tool] for FPGA operations.
pub struct YourSoftenerFPGA {
    device_handle: String,
}

impl YourSoftenerFPGA {
    /// Create a new FPGA device instance.
    ///
    /// # Arguments
    ///
    /// * `device_handle` - The device handle (e.g., "fpga0")
    ///
    /// # Returns: `Self`
    ///
    /// New FPGA instance
    pub fn new(device_handle: &str) -> Self {
        Self {
            device_handle: device_handle.to_string(),
        }
    }
}

impl Fpga for YourSoftenerFPGA {
    fn device_handle(&self) -> &str {
        &self.device_handle
    }

    fn state(&self) -> Result<String, FpgadError> {
        trace!("Getting state for device '{}'", self.device_handle);
        // Custom implementation - call your vendor tool
        let output = Command::new("vendor-tool").arg("status").output().map_err(|e| FpgadError::Io(e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(FpgadError::Io(std::io::Error::other(
                String::from_utf8_lossy(&output.stderr).to_string()
            )))
        }
    }

    fn flags(&self) -> Result<u32, FpgadError> {
        // You can read from sysfs or use vendor tool
        // For standard flags, sysfs is fine:
        crate::system_io::fs_read_u32(&format!(
            "/sys/class/fpga_manager/{}/flags",
            self.device_handle
        ))
    }

    fn set_flags(&self, flags: u32) -> Result<(), FpgadError> {
        crate::system_io::fs_write(
            &format!("/sys/class/fpga_manager/{}/flags", self.device_handle),
            &flags.to_string()
        )
    }

    fn load(&self, bitstream_path: &str) -> Result<(), FpgadError> {
        debug!("Loading bitstream '{}' via vendor tool", bitstream_path);
        // Use your vendor's tool for loading
        let output = Command::new("vendor-tool").arg("load").arg(bitstream_path).output().map_err(|e| FpgadError::Io(e))?;

        if output.status.success() {
            Ok(())
        } else {
            Err(FpgadError::Io(std::io::Error::other(
                String::from_utf8_lossy(&output.stderr).to_string()
            )))
        }
    }

    fn name(&self) -> Result<String, FpgadError> {
        crate::system_io::fs_read(&format!(
            "/sys/class/fpga_manager/{}/name",
            self.device_handle
        ))
    }
}
```

#### Overlay Handler: `daemon/src/softeners/your_softener_name/your_softener_name_overlay_handler.rs`

```rust
use crate::platforms::platform::OverlayHandler;
use crate::error::FpgadError;
use std::process::Command;
use log::{debug, trace};

/// Your vendor-specific overlay handler implementation.
///
/// This struct manages device tree overlays using [vendor approach].
pub struct YourSoftenerOverlayHandler {
    overlay_handle: String,
}

impl YourSoftenerOverlayHandler {
    /// Create a new overlay handler instance.
    ///
    /// # Arguments
    ///
    /// * `overlay_handle` - The overlay identifier
    ///
    /// # Returns: `Self`
    ///
    /// New overlay handler instance
    pub fn new(overlay_handle: &str) -> Self {
        Self {
            overlay_handle: overlay_handle.to_string(),
        }
    }
}

impl OverlayHandler for YourSoftenerOverlayHandler {
    fn overlay_handle(&self) -> &str {
        &self.overlay_handle
    }

    fn load(&self, overlay_path: &str) -> Result<(), FpgadError> {
        debug!("Loading overlay '{}' via vendor tool", overlay_path);
        // Use your vendor's method for applying overlays
        let output = Command::new("vendor-tool").arg("apply-overlay").arg(overlay_path).output().map_err(|e| FpgadError::Io(e))?;

        if output.status.success() {
            Ok(())
        } else {
            Err(FpgadError::Io(std::io::Error::other(
                String::from_utf8_lossy(&output.stderr).to_string()
            )))
        }
    }

    fn remove(&self) -> Result<(), FpgadError> {
        debug!("Removing overlay '{}'", self.overlay_handle);
        // Use your vendor's method for removing overlays
        let output = Command::new("vendor-tool").arg("remove-overlay").arg(&self.overlay_handle).output().map_err(|e| FpgadError::Io(e))?;

        if output.status.success() {
            Ok(())
        } else {
            Err(FpgadError::Io(std::io::Error::other(
                String::from_utf8_lossy(&output.stderr).to_string()
            )))
        }
    }

    fn status(&self) -> Result<String, FpgadError> {
        trace!("Getting overlay status for '{}'", self.overlay_handle);
        // Query overlay status via vendor tool
        let output = Command::new("vendor-tool").arg("overlay-status").arg(&self.overlay_handle).output().map_err(|e| FpgadError::Io(e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(FpgadError::Io(std::io::Error::other(
                String::from_utf8_lossy(&output.stderr).to_string()
            )))
        }
    }
}
```

### Step 3: Add Your Softener Error Type

Add a custom error variant to the `FpgadSoftenerError` enum for your softener. This is used for vendor-specific errors
that occur when calling your vendor tools.

Edit `daemon/src/softeners/error.rs` and add your error variant:

```rust
// In daemon/src/softeners/error.rs
#[derive(Debug, thiserror::Error)]
pub enum FpgadSoftenerError {
    #[error("FpgadSoftenerError::DfxMgr: {0}")]
    DfxMgr(String),

    #[error("FpgadSoftenerError::YourSoftenerName: {0}")]
    YourSoftenerName(String),  // Add your error variant
}
```

**Naming Convention**: Use PascalCase matching your softener's name (e.g., `DfxMgr`).

**Error Message Format**: Follow the pattern `FpgadSoftenerError::VariantName: {0}` to maintain consistency.

**Error Type**: Use `String` to contain detailed error messages from your vendor tools.

You'll also need to update the `From<FpgadSoftenerError>` implementations to handle your new variant:

```rust
impl From<FpgadSoftenerError> for fdo::Error {
    fn from(err: FpgadSoftenerError) -> Self {
        error!("{err}");
        match err {
            FpgadSoftenerError::DfxMgr(..) => fdo::Error::Failed(err.to_string()),
            FpgadSoftenerError::YourSoftenerName(..) => fdo::Error::Failed(err.to_string()),
        }
    }
}

impl From<FpgadSoftenerError> for FpgadError {
    fn from(err: FpgadSoftenerError) -> Self {
        error!("{err}");
        match err {
            FpgadSoftenerError::DfxMgr(..) => FpgadError::Softener(err),
            FpgadSoftenerError::YourSoftenerName(..) => FpgadError::Softener(err),
        }
    }
}
```

This error type will be used throughout your softener implementation when vendor tool operations fail.

### Step 4: Register the Module

Edit `daemon/src/softeners.rs` to include your new softener module:

```rust
pub mod error;

pub mod softeners_thread;
#[cfg(feature = "xilinx-dfx-mgr")]
pub mod xilinx_dfx_mgr;

#[cfg(feature = "your-new-softener")]
pub mod your_softener_name;  // Add this line
```

**Note**: Only register the main platform module here. The sub-modules (FPGA, overlay handler, helpers) are declared
within `your_softener_name.rs` itself and don't need to be listed in this file.

### Step 5: Register the Platform at Startup

Edit `daemon/src/lib.rs` to import and register your platform in the `register_platforms()` function:

```rust
#[cfg(feature = "xilinx-dfx-mgr")]
use crate::softeners::xilinx_dfx_mgr::XilinxDfxMgrPlatform;

#[cfg(feature = "your-new-softener")]
use crate::softeners::your_softener_name::YourSoftenerPlatform;  // Add this

pub fn register_platforms() {
    #[cfg(feature = "xilinx-dfx-mgr")]
    XilinxDfxMgrPlatform::register_platform();

    #[cfg(feature = "your-new-softener")]
    YourSoftenerPlatform::register_platform();  // Add this

    XilinxSysPlatform::register_platform();
}
```

**Important**: The XilinxSys platform should always be registered last as it serves as the fallback.

### Step 6: Implement Vendor-Specific Functions (optional)

Add functions that wrap your vendor's tools or APIs.
These might use `std::process::Command` to call external binaries.
These will return `Result<T, FpgadSoftenerError>` where `T` is typically `String`, and must be convertable to `String`
if not.

Example pattern:

```rust
use std::process::Command;
use crate::softeners::error::FpgadSoftenerError;

/// Brief description of what this does
pub fn vendor_operation(arg: &str) -> Result<String, FpgadSoftenerError> {
    // This example assumes an external binary `vendor-tool` is used
    let output = Command::new("vendor-tool")
        .arg("--option")
        .arg(arg)
        .output()
        .map_err(|e| FpgadSoftenerError::YourSoftenerName(format!("vendor-tool failed to execute: {}", e)))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(FpgadSoftenerError::YourSoftenerName(format!(
            "vendor-tool failed.\n{}\nStdout:\n{}\nStderr:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        )))
    }
}
```

## Key Concepts

### Compatibility Strings

The `compat_string` parameter in the `#[platform]` macro can contain multiple comma-separated values:

```rust
#[platform(compat_string = "xlnx,zynqmp-pcap-fpga,versal-fpga,zynq-devcfg-1.0")]
```

This means the platform will be selected if the device's `/sys/class/fpga_manager/<device>/of_node/compatible` file
contains ANY of these strings.

### OnceLock for Lazy Initialization

Use `OnceLock` to lazily initialize components only when they're first accessed:

```rust
pub struct YourSoftenerPlatform {
    fpga: OnceLock<YourSoftenerFPGA>,
    overlay_handler: OnceLock<YourSoftenerOverlayHandler>,
}

impl Platform for YourSoftenerPlatform {
    fn fpga(&self, device_handle: &str) -> Result<&dyn Fpga, FpgadError> {
        Ok(self.fpga.get_or_init(|| YourSoftenerFPGA::new(device_handle)))
    }

    fn overlay_handler(&self, overlay_handle: &str) -> Result<&dyn OverlayHandler, FpgadError> {
        Ok(self.overlay_handler.get_or_init(|| YourSoftenerOverlayHandler::new(overlay_handle)))
    }
}
```

This pattern ensures that the FPGA and OverlayHandler instances are created only once, when first requested, and then
reused for all subsequent calls.

## Testing Your Softener

### Unit Testing

If possible, add unit tests which can be run on any host to the relevant files:

```no_run
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        // Your tests here
    }
}
```

### Integration Testing

Add your integration tests to `daemon/tests/<your_platform>` to test FPGAd and your softener on your actual hardware.

[//]: # (TODO -artie-: describe testing steps)

### Manual testing

Do some manual testing to ensure that everything is working as expected. Don't forget to build with the appropriate
feature flag:

```bash
cargo build --features your-new-softener
```

## Best Practices

1. **Document Everything**: Add comprehensive doc comments to all public functions
2. **Use Feature Gates**: Always guard your softener code with `#[cfg(feature = "...")]`
3. **Error Handling**: Use `FpgadSoftenerError` for all softener-specific errors. Do not allow fpgad to panic.
4. **Logging**: Use the `log` crate (`trace!`, `debug!`, `info!`, `warn!`, `error!`) liberally
5. **Fail Gracefully**: If vendor tools aren't installed, provide helpful error messages. Do not allow fpgad to panic.

[//]: # (## Example: Xilinx DFX Manager Structure)

[//]: # (TODO -artie-: describe all files edited by dfx-mgr approach, and how they were edited.)
