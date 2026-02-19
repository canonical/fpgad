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

//! Universal platform implementation for standard Linux FPGA subsystem.
//!
//! This module provides the Universal platform, which is a generic implementation
//! that works with tested FPGA devices using one of the common Linux FPGA subsystems,
//! without requiring vendor-specific extensions. It serves as the fallback platform
//! when no more specific platform matches a device's compatibility string.
//!
//! # Features
//!
//! - **Generic FPGA Support** - Works with any FPGA manager driver in the Linux kernel
//! - **Device Tree Overlays** - Full support for overlay application and removal via configfs
//! - **No Vendor Dependencies** - Pure Linux kernel API usage without vendor SDKs
//! - **Automatic Fallback** - Used when device compatibility string doesn't match registered
//!   platforms, or no softeners are included.
//!
//! # Architecture
//!
//! The Universal platform uses lazy initialization via `OnceLock` to create component
//! instances on first access:
//! - [`UniversalFPGA`] - Manages FPGA device operations
//! - [`UniversalOverlayHandler`] - Manages overlay operations
//!
//! # Registration
//!
//! The platform automatically registers itself with the compatibility string "universal"
//! via the `#[platform]` procedural macro. This ensures it's always available as a fallback.
//!
//! # Examples
//!
//! ```rust,no_run
//! # use daemon::platforms::universal::UniversalPlatform;
//! # use daemon::platforms::platform::Platform;
//!
//! # fn example() -> Result<(), daemon::error::FpgadError> {
//! let platform = platform_for_known_platform("universal");
//! let fpga = platform.fpga("fpga0")?;
//! let state = fpga.state()?;
//! # Ok(())
//! # }
//! ```

use crate::error::FpgadError;
use crate::platforms::platform::{Fpga, OverlayHandler, Platform};
use crate::platforms::universal_components::universal_fpga::UniversalFPGA;
use crate::platforms::universal_components::universal_overlay_handler::UniversalOverlayHandler;
use fpgad_macros::platform;
use log::trace;
use std::sync::OnceLock;

/// Universal platform implementation for generic FPGA management.
///
/// This struct provides a platform implementation that works with any FPGA device
/// using the standard Linux FPGA subsystem. It uses lazy initialization to create
/// FPGA and overlay handler instances on first access, ensuring efficient resource
/// usage.
///
/// The `#[platform]` macro automatically registers this platform with the compatibility
/// string "universal", making it available as a fallback for devices without specific
/// platform support.
///
/// # Fields
///
/// * `fpga` - Lazily initialized FPGA device instance
/// * `overlay_handler` - Lazily initialized overlay handler instance
///
/// # Thread Safety
///
/// This struct is thread-safe thanks to `OnceLock`, which ensures that initialization
/// happens exactly once even with concurrent access.
#[platform(compat_string = "universal")]
#[derive(Debug)]
pub struct UniversalPlatform {
    fpga: OnceLock<UniversalFPGA>,
    overlay_handler: OnceLock<UniversalOverlayHandler>,
}

impl Default for UniversalPlatform {
    fn default() -> Self {
        Self::new()
    }
}

impl UniversalPlatform {
    /// Create a new Universal platform instance.
    ///
    /// Creates an empty platform with uninitialized FPGA and overlay handler instances.
    /// The actual components will be lazily initialized on first access through the
    /// [`Platform`] trait methods.
    ///
    /// # Returns: `Self`
    /// * New UniversalPlatform instance ready for use
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use daemon::platforms::universal::UniversalPlatform;
    ///
    /// let platform = platform_for_known_platform("universal");
    /// ```
    pub fn new() -> Self {
        trace!("creating new universal_platform");
        UniversalPlatform {
            fpga: OnceLock::new(),
            overlay_handler: OnceLock::new(),
        }
    }
}

impl Platform for UniversalPlatform {
    /// Get or initialize the FPGA device instance.
    ///
    /// Returns a reference to the [`UniversalFPGA`] instance for the specified device.
    /// On first call, this creates and initializes the FPGA instance. Subsequent calls
    /// return the same cached instance.
    ///
    /// # Arguments
    ///
    /// * `device_handle` - The device handle (e.g., "fpga0")
    ///
    /// # Returns: `Result<&dyn Fpga, FpgadError>`
    /// * `Ok(&dyn Fpga)` - Reference to the FPGA device instance
    ///
    /// # Note
    ///
    /// This implementation currently never returns an error, but the Result type
    /// is required by the Platform trait to support platform-specific validation.
    fn fpga(&self, device_handle: &str) -> Result<&dyn Fpga, FpgadError> {
        Ok(self.fpga.get_or_init(|| UniversalFPGA::new(device_handle)))
    }

    /// Get or initialize the overlay handler instance.
    ///
    /// Returns a reference to the [`UniversalOverlayHandler`] instance for the specified
    /// overlay. On first call, this creates and initializes the handler. This method also
    /// validates that the configfs overlay directory exists.
    ///
    /// # Arguments
    ///
    /// * `overlay_handle` - The overlay handle (directory name in configfs)
    ///
    /// # Returns: `Result<&dyn OverlayHandler, FpgadError>`
    /// * `Ok(&dyn OverlayHandler)` - Reference to the overlay handler instance
    /// * `Err(FpgadError::Argument)` - Overlay path has no parent or parent doesn't exist
    ///
    /// # Implementation Note
    ///
    /// This method includes a workaround for the unstable `get_or_try_init` feature.
    /// Once that feature is stable, the error handling can be improved. See:
    /// <https://github.com/rust-lang/rust/issues/121641>
    fn overlay_handler(&self, overlay_handle: &str) -> Result<&dyn OverlayHandler, FpgadError> {
        // TODO: replace the return type of UniversalOverlayHandler to Result and use
        // get_or_try_init instead here when stable:
        // https://github.com/rust-lang/rust/issues/121641
        let handler = self
            .overlay_handler
            .get_or_init(|| UniversalOverlayHandler::new(overlay_handle));

        // NOTE: This will fail if the constructor fails.
        let parent_path = handler.overlay_fs_path()?.parent().ok_or_else(|| {
            FpgadError::Argument(format!(
                "The path {:?} has no parent directory.",
                handler.overlay_fs_path()
            ))
        })?;

        if !parent_path.exists() {
            return Err(FpgadError::Argument(format!(
                "The overlayfs path {parent_path:?} doesn't seem to exist."
            )));
        }
        Ok(handler)
    }
}
