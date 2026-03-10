# fpgad_macros

Procedural macros for the FPGAd project.

## Overview

This crate provides procedural macros used by the FPGAd daemon to simplify platform registration and other compile-time code generation tasks.

## Macros

### `#[platform]`

The `platform` attribute macro is used to register FPGA platform implementations with the daemon.

**Usage:**

```rust
use fpgad_macros::platform;

#[platform(compat_string = "xlnx,zynqmp")]
pub struct ZynqMPPlatform {
    // Platform-specific fields
}
```

This macro automatically implements the necessary registration code for the platform, making it discoverable by the FPGAd daemon at runtime.

## License

GPL-3.0

