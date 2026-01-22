# FPGAd Softeners

## Overview

Softeners are platform-specific extensions to FPGAd that provide integration with vendor-specific FPGA management tools and frameworks. While FPGAd's core platform system handles standard Linux FPGA subsystem operations, softeners enable fpgad to work with proprietary or vendor-specific tools that sit alongside or above the kernel FPGA subsystem.

> **Note on Naming**: FPGA devices are often referred to as "fabric" (as in "programmable logic fabric"), so naturally FPGAd provides "softeners" for the fabric 🧺.

### When to Use a Softener vs. a Platform

- **Use a Platform** when implementing direct interactions with the Linux FPGA subsystem (`/sys/class/fpga_manager`) and device tree overlays. Platforms handle the standard kernel interface.
  
- **Use a Softener** when integrating with vendor-specific userspace tools, SDKs, or frameworks that provide additional functionality beyond what the kernel exposes. Softeners typically wrap external binaries or libraries.

## Architecture

Softeners extend platforms by providing vendor-specific functionality. A softener platform:
1. Implements the `Platform` trait (like any other platform)
2. Uses the `#[platform(compat_string = "...")]` macro to register itself
3. Provides additional vendor-specific functions beyond the standard `Platform` trait
4. Is conditionally compiled using Cargo feature flags

### Example: Xilinx DFX Manager

The `xilinx_dfx_mgr` softener integrates with AMD/Xilinx's `dfx-mgr` tool, which provides vendor specific optimizations on top of the standard FPGA subsystem and, e.g., enables "partial reconfiguration" of the device, which isn't supported by the Universal platform. See [dfx-mgr on GitHub](https://github.com/Xilinx/dfx-mgr) for more information on dfx-mgr.

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

### Step 2: Create the Softener Module

Create a new file at `daemon/src/softeners/your_softener_name.rs`:

```rust
use std::sync::OnceLock;

use crate::platforms::platform::Platform;
use crate::platforms::universal_components::universal_fpga::UniversalFPGA;
use crate::platforms::universal_components::universal_overlay_handler::UniversalOverlayHandler;
use crate::softeners::error::FpgadSoftenerError;
use fpgad_macros::platform;

/// Your softener platform implementation.
///
/// This platform integrates with [vendor tool name] to provide [brief description].
/// 
/// # Compatibility
/// 
/// Compatible with devices matching: `your,compat-string`
#[platform(compat_string = "your,compat-string")]
pub struct YourSoftenerPlatform {
    fpga: OnceLock<UniversalFPGA>,
    overlay_handler: OnceLock<UniversalOverlayHandler>,
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
        Ok(self.fpga.get_or_init(|| YourPlatformFPGA::new(device_handle)))
        // See step (7) about implementing custom Fpga
    }

    fn overlay_handler(
        &self,
        overlay_handle: &str,
    ) -> Result<&dyn crate::platforms::platform::OverlayHandler, crate::error::FpgadError> {
        Ok(self
            .overlay_handler
            .get_or_init(|| YourPlatformOverlayHandler::new(overlay_handle)))
        // See step (7) about implementing custom OverlayHandler
    }
}

// Add your vendor-specific functions below
// These are public functions that can be called by the daemon or exposed via DBus

/// Example vendor-specific function
pub fn do_vendor_thing(param: &str) -> Result<String, FpgadSoftenerError> {
    // Implementation here
    todo!()
}
```

### Step 3: Register the Module

Edit `daemon/src/softeners.rs` to include your new softener:

```rust
pub mod error;

#[cfg(feature = "xilinx-dfx-mgr")]
pub mod xilinx_dfx_mgr;

#[cfg(feature = "your-new-softener")]
pub mod your_softener_name;  // Add this line
```

### Step 4: Register the Platform at Startup

Edit `daemon/src/main.rs` to register your platform:

```rust
#[cfg(feature = "xilinx-dfx-mgr")]
use softeners::xilinx_dfx_mgr::XilinxDfxMgrPlatform;

#[cfg(feature = "your-new-softener")]
use softeners::your_softener_name::YourSoftenerPlatform;  // Add this

fn register_platforms() {
    #[cfg(feature = "xilinx-dfx-mgr")]
    XilinxDfxMgrPlatform::register_platform();
    
    #[cfg(feature = "your-new-softener")]
    YourSoftenerPlatform::register_platform();  // Add this
    
    UniversalPlatform::register_platform();
}
```

**Important**: The Universal platform should always be registered last as it serves as the fallback.

### Step 5: Implement Vendor-Specific Functions

Add functions that wrap your vendor's tools or APIs. These typically:
- Use `std::process::Command` to call external binaries
- Return `Result<T, FpgadSoftenerError>` for error handling
- Are marked with `#[allow(dead_code)]` if not yet exposed via DBus

Example pattern:

```rust
use std::process::Command;
use crate::softeners::error::FpgadSoftenerError;

/// Brief description of what this does
pub fn vendor_operation(arg: &str) -> Result<String, FpgadSoftenerError> {
    let output = Command::new("vendor-tool")
        .arg("--option")
        .arg(arg)
        .output()
        .map_err(FpgadSoftenerError::YourError)?;
        
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(FpgadSoftenerError::YourError(std::io::Error::other(
            String::from_utf8_lossy(&output.stderr).to_string(),
        )))
    }
}
```

### Step 6: Add Error Types (If Needed)

If your softener needs custom error types beyond what's in `error.rs`, add them:

```rust
// In daemon/src/softeners/error.rs
#[derive(Debug, thiserror::Error)]
pub enum FpgadSoftenerError {
    #[error("FpgadSoftenerError::DfxMgr: {0}")]
    DfxMgr(std::io::Error),
    
    #[error("FpgadSoftenerError::YourError: {0}")]
    YourError(std::io::Error),  // Add your error variant
}
```

### Step 7: Implement Custom FPGA/OverlayHandler (Advanced, Optional)

If your vendor-specific platform requires custom behavior for FPGA or overlay operations, you can implement your own, otherwise just use the Universal platform's `UniversalFPGA` and `UniversalOverlayHandler` implementations.


#### Creating Custom FPGA Implementation

Create a struct that implements the `Fpga` trait:

```rust
use crate::platforms::platform::Fpga;
use crate::error::FpgadError;

pub struct YourPlatformFPGA {
    device_handle: String,
}

impl YourPlatformFPGA {
    pub fn new(device_handle: &str) -> Self {
        Self {
            device_handle: device_handle.to_string(),
        }
    }
}

impl Fpga for YourPlatformFPGA {
    fn device_handle(&self) -> &str {
        &self.device_handle
    }
    
    fn state(&self) -> Result<String, FpgadError> {
        // Custom implementation - maybe call vendor tool
        // or fall back to reading sysfs directly
        todo!()
    }
   
    fn flags(&self) -> Result<u32, FpgadError> {
        // Custom implementation
        todo!()
    }
    
    fn set_flags(&self, flags: u32) -> Result<(), FpgadError> {
        // Custom implementation
        todo!()
    }

    fn load(&self, bitstream_path: &str) -> Result<(), FpgadError> {
        // Custom implementation - perhaps using vendor-specific loading
        todo!()
    }

    fn name(&self) -> Result<String, FpgadError> {
        // Custom implementation
        todo!()
    }
}
```

#### Creating Custom OverlayHandler Implementation

Create a struct that implements the `OverlayHandler` trait:

```rust
use crate::platforms::platform::OverlayHandler;
use crate::error::FpgadError;

pub struct YourPlatformOverlayHandler {
    overlay_handle: String,
}

impl YourPlatformOverlayHandler {
    pub fn new(overlay_handle: &str) -> Self {
        Self {
            overlay_handle: overlay_handle.to_string(),
        }
    }
}

impl OverlayHandler for YourPlatformOverlayHandler {
    fn overlay_handle(&self) -> &str {
        &self.overlay_handle
    }

    fn load(&self, overlay_path: &str) -> Result<(), FpgadError> {
        // Custom implementation - maybe use vendor-specific overlay mechanism
        todo!()
    }

    fn remove(&self) -> Result<(), FpgadError> {
        // Custom implementation
        todo!()
    }

    fn status(&self) -> Result<String, FpgadError> {
        // Custom implementation
        todo!()
    }
   
}
```

#### Using Your Custom Implementations

Update your platform's `fpga()` and `overlay_handler()` methods to use your custom implementations - note that this was
already included in the above example code.:

```rust
impl Platform for YourSoftenerPlatform {
    fn fpga(
        &self,
        device_handle: &str,
    ) -> Result<&dyn crate::platforms::platform::Fpga, crate::error::FpgadError> {
        Ok(self.fpga.get_or_init(|| YourPlatformFPGA::new(device_handle)))
    }

    fn overlay_handler(
        &self,
        overlay_handle: &str,
    ) -> Result<&dyn crate::platforms::platform::OverlayHandler, crate::error::FpgadError> {
        Ok(self
            .overlay_handler
            .get_or_init(|| YourPlatformOverlayHandler::new(overlay_handle)))
    }
}
```

And update your struct fields to use the custom types:

```rust
pub struct YourSoftenerPlatform {
    fpga: OnceLock<YourPlatformFPGA>,
    overlay_handler: OnceLock<YourPlatformOverlayHandler>,
}
```

#### When to Implement Custom Components

Consider implementing custom FPGA or OverlayHandler when:
- The vendor tooling provides optimized loading mechanisms
- The standard sysfs interface doesn't expose all necessary functionality
- The sysfs interface is different on your platform (instead of changing [config.rs](../config.rs))

For most cases, the Universal components are sufficient and recommended for simplicity.

## Key Concepts

### Compatibility Strings

The `compat_string` parameter in the `#[platform]` macro can contain multiple comma-separated values:

```rust
#[platform(compat_string = "xlnx,zynqmp-pcap-fpga,versal-fpga,zynq-devcfg-1.0")]
```

This means the platform will be selected if the device's `/sys/class/fpga_manager/<device>/of_node/compatible` file contains ANY of these strings.

### OnceLock for Lazy Initialization

Use `OnceLock` to lazily initialize components only when they're first accessed:

```rust
pub struct YourPlatform {
    fpga: OnceLock<UniversalFPGA>,
    overlay_handler: OnceLock<UniversalOverlayHandler>,
}

impl Platform for YourPlatform {
    fn fpga(&self, device_handle: &str) -> Result<&dyn Fpga, FpgadError> {
        Ok(self.fpga.get_or_init(|| UniversalFPGA::new(device_handle)))
    }
}
```

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

Do some manual testing to ensure that everything is working as expected. Don't forget to build with the appropriate feature flag:

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
