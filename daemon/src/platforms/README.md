# Platform Abstractions for FPGA Hardware Management

This module provides a platform registry system that allows fpgad to support multiple
FPGA platforms through a common interface. Each platform implementation handles the
specific details of interacting with different FPGA hardware and toolchains.

## Architecture

The platform system uses a registration pattern where platform implementations register
themselves with a global registry using device tree compatibility strings. When fpgad
discovers FPGA devices during initialization, it queries the registry to find the
appropriate platform implementation based on the device's compatibility string.

Once a platform is selected, it is instantiated and used to manage the FPGA device.
Platforms handle operations like loading firmware, removing overlays, and querying device status.

## Workflow

1. Platform implementations register themselves with the platform registry during initialization
2. fpgad discovers FPGA devices by scanning the system
3. For each device, fpgad looks up the matching platform by compatibility string
4. The platform is instantiated and used to manage the device
5. Vendor-specific "softener" platforms can provide enhanced functionality beyond the standard interface

## Built-in Platforms

- **Universal Platform** ([`universal`]): A generic platform that provides basic FPGA management
  functionality for devices that don't require platform-specific handling.

## Optional Softener Platforms (using Features)

These platforms are vendor-specific implementations called "softeners" that provide enhanced
functionality beyond the universal platform. They are enabled via Cargo feature flags.

- **Xilinx DFX Manager** ([`softeners::xilinx_dfx_mgr`](crate::softeners::xilinx_dfx_mgr)) -
  **Feature** `xilinx-dfx-mgr`

  Custom support for AMD-AECG (formerly Xilinx) FPGA devices that integrates with the Xilinx
  dfx-mgr daemon. Implements dfx-mgr as a backend to support advanced features like partial
  reconfiguration.


  **Supported devices:**
  - `xlnx,zynqmp-pcap-fpga` - Zynq UltraScale+ MPSoC
  - `versal-fpga` - Versal ACAP devices
  - `zynq-devcfg-1.0` - Zynq-7000 devices

  **Requirements:** The dfx-mgrd daemon must be running (automatically started in snap
  environment).


## Platform Registration

Platform implementations use the `#[platform]` macro from `fpgad_macros` to automatically
generate registration code:

```rust,ignore
use fpgad_macros::platform;

#[platform(compat_string = "xlnx,zynqmp-pcap-fpga")]
pub struct XilinxPlatform {
    // ...
}
```

## See Also

- [`softeners`](crate::softeners) - Vendor-specific platform extensions (feature-gated)
- [`platform`] - Core platform registry and trait definitions

