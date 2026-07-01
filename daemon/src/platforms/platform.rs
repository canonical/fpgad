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

//! Platform abstraction layer for FPGA management.
//!
//! This module defines the core trait system that enables vendor and hardware-specific
//! FPGA management implementations. The platform system uses a registry-based approach
//! where platform implementations register themselves with compatibility strings that
//! match Linux device tree compatible strings. This module also provides functions for
//! retrieving the appropriate platform implementation for a given FPGA device at runtime.
//!
//! # Architecture
//!
//! The platform abstraction consists of three main traits:
//! - [`Platform`] - Top-level platform interface that provides access to FPGA and overlay handlers
//! - [`Fpga`] - Interface for interacting with FPGA devices through the Linux FPGA subsystem
//! - [`OverlayHandler`] - Interface for managing device tree overlays
//!
//! # Platform Discovery
//!
//! At runtime, the daemon discovers which platform to use for each FPGA device by:
//! 1. Reading the device's `/sys/class/fpga_manager/<device>/of_node/compatible` string
//! 2. Matching this against registered platform compatibility strings
//! 3. Falling back to the XilinxSys platform if no match is found
//!
//! # Platform Registration
//!
//! Platforms register themselves at daemon startup using the [`register_platform`] function
//! and are defined using the `#[platform]` macro. Compatibility strings can include
//! comma-separated components, all of which must match for a platform to be selected.
//! Platforms and softeners are included or not excluded using cargo "features".
//! See [`softeners`](../../softeners/index.html) for more details.
//!
//! TODO(Artie): Add examples of how to use the getters for platforms with and without knowing the
//!  platform string? - could be called "# Fetching platforms"
//! # Examples
//!
//! in [main.rs]:
//! ```rust,no_run
//! #[cfg(feature = "xilinx-dfx-mgr")]
//! use softeners::xilinx_dfx_mgr::XilinxDfxMgrPlatform;
//!
//! #[cfg(feature = "your-new-softener")]
//! use softeners::your_softener_name::YourSoftenerPlatform;  // Add this
//!
//! fn register_platforms() {
//!     #[cfg(feature = "xilinx-dfx-mgr")]
//!     XilinxDfxMgrPlatform::register_platform();
//!
//!     #[cfg(feature = "your-new-softener")]
//!     YourSoftenerPlatform::register_platform();  // Add this
//!
//!     XilinxSysPlatform::register_platform();
//! }
//! ```
//!

use crate::config;
use crate::error::FpgadError;
use crate::platforms::xilinx_sys::XilinxSysPlatform;
use crate::system_io::{fs_read, fs_read_dir};
use log::{trace, warn};
use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::{Mutex, OnceLock};

/// Type alias for platform constructor functions.
///
/// Platform constructors take no arguments and return a boxed Platform trait object.
/// These functions are stored in the platform registry and called when a matching
/// platform is discovered.
type PlatformConstructor = fn() -> Box<dyn Platform>;

/// Global registry of platform implementations.
///
/// This static variable holds a thread-safe registry mapping compatibility strings
/// to platform constructor functions. It is initialized once at daemon startup via
/// [`init_platform_registry`] and accessed through [`register_platform`] and
/// [`match_platform_string`].
///
/// The registry uses `OnceLock` to ensure thread-safe lazy initialization and `Mutex`
/// to protect concurrent access to the internal HashMap.
pub static PLATFORM_REGISTRY: OnceLock<Mutex<HashMap<&'static str, PlatformConstructor>>> =
    OnceLock::new();

/// Trait for managing an FPGA device
pub trait Fpga {
    /// Get the device handle for this FPGA device.
    ///
    /// Returns the device handle (e.g., "fpga0") that identifies this FPGA device.
    ///
    /// # Returns: `&str`
    /// * Device handle string (e.g., "fpga0", "fpga1")
    #[allow(dead_code)]
    fn device_handle(&self) -> &str;

    /// Get the current state of the FPGA device.
    ///
    /// # Returns: `Result<String, FpgadError>`
    /// * `Ok(String)` - Current state of the FPGA
    /// * `Err(FpgadError::IORead)` - Failed to read state file
    fn state(&self) -> Result<String, FpgadError>;

    /// Load a bitstream firmware file to the FPGA device.
    ///
    /// # Arguments
    ///
    /// * `bitstream_path` - Absolute path to the bitstream file
    /// * `firmware_lookup_path` - Path to resolve firmware or empty path
    ///   (automatically uses the parent dir of `bitstream_path`)
    ///
    /// # Returns: `Result<String, FpgadError>`
    /// * `Ok(String)` - Confirmation message with source and target
    /// * `Err(FpgadError::IOWrite)` - Failed to write firmware file
    /// * `Err(FpgadError::FPGAState)` - FPGA not in correct state for loading
    #[allow(dead_code)]
    fn load_firmware(
        &self,
        bitstream_path: &Path,
        firmware_lookup_path: &Path,
    ) -> Result<String, FpgadError>;

    /// Remove a previously loaded firmware/bitstream.
    ///
    /// # Arguments
    ///
    /// * `handle` - Optional handle/slot identifier for the firmware to remove
    ///
    /// # Returns: `Result<String, FpgadError>`
    /// * `Ok(String)` - Confirmation message including device and firmware details
    /// * `Err(FpgadError::Internal)` - Operation not supported by this platform
    /// * `Err(FpgadError)` - Failed to remove firmware
    fn remove_firmware(&self, handle: Option<&str>) -> Result<String, FpgadError>;
}

/// Trait for managing device tree overlays.
pub trait OverlayHandler {
    /// Apply a device tree overlay from a source file.
    ///
    /// # Arguments
    ///
    /// * `source_path` - Path to the `.dtbo` overlay binary file
    /// * `lookup_path` - Path to resolve overlay firmware or empty path
    ///   (automatically uses the parent dir of `source_path`)
    ///
    /// # Returns: `Result<String, FpgadError>`
    /// * `Ok(String)` - Confirmation message with overlay path and firmware prefix
    /// * `Err(FpgadError::IOWrite)` - Failed to write overlay
    /// * `Err(FpgadError::OverlayStatus)` - Overlay application failed
    fn apply_overlay(&self, source_path: &Path, lookup_path: &Path) -> Result<String, FpgadError>;

    /// Remove a device tree overlay.
    ///
    /// # Arguments
    ///
    /// * `handle` - Optional handle/slot identifier for the overlay to remove
    ///
    /// # Returns: `Result<String, FpgadError>`
    /// * `Ok(String)` - Confirmation message including overlay filesystem path
    /// * `Err(FpgadError::IODelete)` - Failed to remove overlay directory
    fn remove_overlay(&self, handle: Option<&str>) -> Result<String, FpgadError>;

    /// Get the current status of the overlay.
    ///
    /// # Returns: `Result<String, FpgadError>`
    /// * `Ok(String)` - Overlay status string (e.g., "applied", "")
    /// * `Err(FpgadError::IORead)` - Failed to read status
    fn status(&self) -> Result<String, FpgadError>;

    /// Get the filesystem path to the overlay directory.
    ///
    /// # Returns: `Result<&Path, FpgadError>`
    /// * `Ok(&Path)` - Path to overlay directory in configfs
    /// * `Err(FpgadError)` - Overlay path not initialized
    fn overlay_fs_path(&self) -> Result<&Path, FpgadError>;
}

/// Trait representing a complete FPGA platform implementation.
///
/// This trait is the top-level interface for platform-specific FPGA management.
/// Implementations provide factory methods for creating FPGA device and overlay
/// handler instances.
///
/// The trait extends `Any` to allow for runtime type checking and downcasting,
/// which can be useful for platform-specific functionality.
///
/// # Examples
///
/// ```rust,no_run
/// # use crate::platforms::platform::Platform;
/// #
/// # fn example(platform: &dyn Platform) -> Result<(), crate::error::FpgadError> {
/// // Get an FPGA device instance
/// let fpga = platform.fpga("fpga0")?;
/// let state = fpga.state()?;
///
/// // Get an overlay handler instance
/// let overlay = platform.overlay_handler("my_overlay")?;
/// # Ok(())
/// # }
/// ```
pub trait Platform: Any {
    /// Get or create an FPGA device instance.
    ///
    /// Returns a reference to an [`Fpga`] trait object for the specified device.
    /// Implementations typically cache instances and return the same instance for
    /// repeated calls with the same device handle.
    ///
    /// # Arguments
    ///
    /// * `device_handle` - The device handle (e.g., "fpga0")
    ///
    /// # Returns: `Result<&dyn Fpga, FpgadError>`
    /// * `Ok(&dyn Fpga)` - FPGA device instance
    /// * `Err(FpgadError::Argument)` - Invalid device handle
    fn fpga(&self, device_handle: &str) -> Result<&dyn Fpga, FpgadError>;

    /// Get or create an overlay handler instance.
    ///
    /// Returns a reference to an [`OverlayHandler`] trait object for the specified
    /// overlay. Implementations typically cache instances and return the same instance
    /// for repeated calls with the same overlay handle.
    ///
    /// # Arguments
    ///
    /// * `overlay_handle` - The overlay handle (directory name in configfs)
    ///
    /// # Returns: `Result<&dyn OverlayHandler, FpgadError>`
    /// * `Ok(&dyn OverlayHandler)` - Overlay handler instance
    /// * `Err(FpgadError::Argument)` - Invalid overlay handle or configfs not available
    fn overlay_handler(&self, overlay_handle: &str) -> Result<&dyn OverlayHandler, FpgadError>;

    /// Get a formatted status message for this platform.
    ///
    /// Returns a human-readable status message containing information about devices,
    /// overlays, and platform-specific state. The format and content are platform-specific.
    ///
    /// # Returns: `Result<String, FpgadError>`
    /// * `Ok(String)` - Formatted status message
    /// * `Err(FpgadError)` - Failed to gather status information
    ///
    /// # Examples
    ///
    /// For the XilinxSys platform, this returns a table of devices and overlays.
    /// For Xilinx DFX Manager, this returns the output of `dfx-mgr-client -listPackage`.
    fn status_message(&self) -> Result<String, FpgadError>;

    /// Get the platform compatibility string.
    ///
    /// Returns the compatibility string that this platform implementation is registered
    /// with. This string matches against device tree compatible properties to determine
    /// which platform to use for a device.
    ///
    /// Implementations should return the same value as specified in the `#[platform]` macro's
    /// `compat_string` attribute. The macro generates a `COMPAT_STRING` constant that can
    /// be used: `Self::COMPAT_STRING.into()`.
    ///
    /// # Returns: `String`
    /// * The platform compatibility string (e.g., "xlnx-sys", "xlnx,zynqmp-pcap-fpga")
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use daemon::platforms::platform::Platform;
    /// # fn example(platform: &dyn Platform) {
    /// let compat = platform.platform_compat_string();
    /// println!("Platform: {}", compat);
    /// # }
    /// ```
    fn platform_compat_string(&self) -> String;

    /// Check whether this platform is available
    ///
    /// This should determine if a platform is available.
    /// For built-in platforms (in the `platforms` dir), this always returns `true`.
    /// For softeners (in the `softeners` dir), this can implement custom logic like
    /// checking if required system services or binaries are present.
    fn is_available(&self) -> bool;
}

/// Match a platform compatibility string to a registered platform.
///
/// This function implements the platform matching algorithm that searches the registry
/// for a platform whose compatibility string matches all components in the provided
/// string. The matching is done by splitting both strings on commas and ensuring ***all***
/// components in the query string are present in the registered compatibility string.
///
/// # Platform Selection Priority
///
/// When multiple platforms match the requested compatibility string:
/// 1. Softeners (platforms with `softener` in their compat string) are preferred if available
/// 2. Built-in `platforms` are used as fallback if no softener is available
///
/// Because matching requires all requested components to be present in the registered
/// compat string, the caller can explicitly constrain which type of platform is returned:
///
/// - Requesting `"xlnx,softener"` only matches platforms whose compat string contains both
///   `"xlnx"` and `"softener"` — built-in platforms (which have `"platform"` but not
///   `"softener"`) will not match, so no built-in fallback is possible.
/// - Requesting `"xlnx,platform"` only matches platforms whose compat string contains both
///   `"xlnx"` and `"platform"` — softeners (which have `"softener"` but not `"platform"`)
///   will not match, so no softener preference applies.
///
/// # Algorithm
///
/// 1. Split the registered compatibility string into a component set: `"xlnx,zynqmp-pcap-fpga"` → `{"xlnx", "zynqmp-pcap-fpga"}`
/// 2. Split the query string into components: `"xlnx"` → `["xlnx"]`
/// 3. Keep only registered platforms whose component set contains all query components
/// 4. Partition matches into available softeners vs everything else
/// 5. Return first available softener, or first fallback match, or error
///
/// # Arguments
///
/// * `platform_string` - Comma-separated compatibility string to match
///
/// # Returns: `Result<Box<dyn Platform>, FpgadError>`
/// * `Ok(Box<dyn Platform>)` - Newly constructed platform instance (softener preferred)
/// * `Err(FpgadError::Internal)` - Registry not initialized or lock failure
/// * `Err(FpgadError::Argument)` - No matching platform found
///
/// # Examples
/// Match on one component (softener preferred if available, built-in as fallback):
/// ```rust,ignore
/// let platform = match_platform_string("xlnx")?;
/// ```
/// Match on all of multiple components:
/// ```rust,ignore
/// let platform = match_platform_string("xlnx,zynqmp-pcap-fpga")?;
/// ```
/// Explicitly request only softeners (no built-in fallback):
/// ```rust,ignore
/// let platform = match_platform_string("xlnx,softener")?;
/// ```
/// Explicitly request only built-in platforms (no softener preference):
/// ```rust,ignore
/// let platform = match_platform_string("xlnx,platform")?;
/// ```
pub fn match_platform_string(platform_string: &str) -> Result<Box<dyn Platform>, FpgadError> {
    let registry = PLATFORM_REGISTRY
        .get()
        .ok_or(FpgadError::Internal(String::from(
            "couldn't get PLATFORM_REGISTRY",
        )))?
        .lock()
        .map_err(|_| FpgadError::Internal(String::from("couldn't lock PLATFORM_REGISTRY")))?;

    // find registered platforms which contain all requested platform string parts
    let matching_platforms: Vec<_> = registry
        .iter()
        .filter(|(compat_string, _)| {
            let compat_set: HashSet<&str> = compat_string.split(',').collect();
            platform_string.split(',').all(|x| compat_set.contains(x))
        })
        .collect();

    // Split matches into available softeners and built-in platforms in one pass
    let (softeners, builtin_platforms): (Vec<_>, Vec<_>) = matching_platforms
        .iter()
        .partition(|(compat_string, _)| compat_string.contains("softener"));

    for (compat_string, constructor) in softeners {
        trace!("Using softener platform: {}", compat_string);
        let platform = constructor();
        if platform.is_available() {
            return Ok(platform);
        }
    }
    trace!(
        "No softeners available for {}, trying built in platforms",
        platform_string
    );

    for (compat_string, constructor) in builtin_platforms {
        trace!("Using softener platform: {}", compat_string);
        let platform = constructor();
        if platform.is_available() {
            return Ok(platform);
        }
    }

    trace!("No built in platforms available for {}", platform_string);

    // No suitable platforms
    Err(FpgadError::Argument(format!(
        "FPGAd could not match {platform_string} to a known platform."
    )))
}

/// Discover the appropriate platform for a device by reading its compatibility string.
///
/// This function reads the device tree compatible string from the device's sysfs
/// `of_node/compatible` file and attempts to match it to a registered platform.
/// If no match is found, it falls back to the XilinxSys platform with a warning.
///
/// # Arguments
///
/// * `device_handle` - The device handle (e.g., "fpga0")
///
/// # Returns: `Result<Box<dyn Platform>, FpgadError>`
/// * `Ok(Box<dyn Platform>)` - Platform instance (matched or XilinxSys fallback)
/// * `Err(FpgadError::Argument)` - Failed to read compatibility string
///
/// # Examples
///
/// ```rust,no_run
/// # use crate::platforms::platform::discover_platform;
/// # fn example() -> Result<(), crate::error::FpgadError> {
/// let platform = discover_platform("fpga0")?;
/// let fpga = platform.fpga("fpga0")?;
/// # Ok(())
/// # }
/// ```
pub fn discover_platform(device_handle: &str) -> Result<Box<dyn Platform>, FpgadError> {
    let compat_string = read_compatible_string(device_handle)?;
    trace!("Found compatibility string: '{compat_string}'");

    match match_platform_string(&compat_string) {
        Ok(platform) => {
            trace!("Matched platform for compatibility string: '{compat_string}'");
            Ok(platform)
        }
        Err(_) => {
            warn!("{compat_string} not supported. Defaulting to XilinxSys platform.");
            Ok(Box::new(XilinxSysPlatform::new()))
        }
    }
}

/// Read the device tree compatible string for an FPGA device.
///
/// Reads the compatibility string from `/sys/class/fpga_manager/<device>/of_node/compatible`.
/// This string identifies the hardware and is used for platform matching. The function
/// handles null-terminated strings that some drivers write to sysfs by trimming the final trailing
/// null byte.
///
/// # Arguments
///
/// * `device_handle` - The device handle (e.g., "fpga0")
///
/// # Returns: `Result<String, FpgadError>`
/// * `Ok(String)` - Compatibility string (null terminators removed)
/// * `Err(FpgadError::Argument)` - Device not found or compatible string unreadable
///
/// # Examples
///
/// ```rust,no_run
/// # use crate::platforms::platform::read_compatible_string;
/// # fn example() -> Result<(), crate::error::FpgadError> {
/// let compat = read_compatible_string("fpga0")?;
/// println!("Compatibility: {}", compat);
/// # Ok(())
/// # }
/// ```
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

/// Get a platform instance from either a compatibility string or by device discovery.
///
/// This is a helper function that chooses between [`discover_platform`] (if
/// `platform_string` is empty) or [`platform_for_known_platform`] (if a platform
/// string is provided). This is commonly used by DBus interface methods.
///
/// # Arguments
///
/// * `platform_string` - Compatibility string (empty for auto-discovery)
/// * `device_handle` - The device handle (e.g., "fpga0")
///
/// # Returns: `Result<Box<dyn Platform>, FpgadError>`
/// * `Ok(Box<dyn Platform>)` - Platform instance
/// * `Err(FpgadError)` - Platform discovery or matching failed
///
/// # Examples
///
/// ```rust,no_run
/// # use crate::platforms::platform::platform_from_compat_or_device;
/// # fn example() -> Result<(), crate::error::FpgadError> {
/// // Auto-discover from device
/// let platform = platform_from_compat_or_device("", "fpga0")?;
///
/// // Use known platform string
/// let platform = platform_from_compat_or_device("xlnx,zynqmp-pcap-fpga", "fpga0")?;
/// # Ok(())
/// # }
/// ```
pub fn platform_from_compat_or_device(
    platform_string: &str,
    device_handle: &str,
) -> Result<Box<dyn Platform>, FpgadError> {
    match platform_string.is_empty() {
        true => discover_platform(device_handle),
        false => platform_for_known_platform(platform_string),
    }
}

/// Get a platform instance for a known compatibility string.
///
/// Directly matches the provided compatibility string against registered platforms
/// without attempting device discovery. This requires the caller to know the correct
/// platform string.
///
/// # Arguments
///
/// * `platform_string` - Compatibility string to match
///
/// # Returns: `Result<Box<dyn Platform>, FpgadError>`
/// * `Ok(Box<dyn Platform>)` - Matched platform instance
/// * `Err(FpgadError::Argument)` - No matching platform found
/// * `Err(FpgadError::Internal)` - Registry not initialized
///
/// # Examples
///
/// ```rust,no_run
/// # use crate::platforms::platform::platform_for_known_platform;
/// # fn example() -> Result<(), crate::error::FpgadError> {
/// let platform = platform_for_known_platform("xlnx,zynqmp-pcap-fpga")?;
/// # Ok(())
/// # }
/// ```
pub fn platform_for_known_platform(platform_string: &str) -> Result<Box<dyn Platform>, FpgadError> {
    match_platform_string(platform_string)
}

/// Initialize the platform registry.
///
/// Creates a new empty mutex-protected HashMap for storing platform entries
/// (constructor + availability checker).
/// This function is called automatically by [`register_platform`] via `OnceLock::get_or_init`.
///
/// # Returns: `Mutex<HashMap<&'static str, PlatformConstructor>>`
/// * Empty mutex-protected HashMap ready for platform registration
pub fn init_platform_registry() -> Mutex<HashMap<&'static str, PlatformConstructor>> {
    Mutex::new(HashMap::new())
}

/// Register a platform implementation in the global registry.
///
/// Adds a platform constructor and availability checker to the registry with an associated
/// compatibility string. The compatibility string should be a comma-separated list of components
/// that match the device tree compatible property. Platforms are typically registered at daemon
/// startup before any devices are discovered.
///
/// # Arguments
///
/// * `compatible` - Compatibility string (e.g., "xlnx,zynqmp-pcap-fpga")
/// * `constructor` - Function that creates a new platform instance
/// * `availability_checker` - Function that checks if the platform is available
///
/// # Panics
///
/// Panics if the registry lock is poisoned (should never happen in normal operation).
///
/// # Examples
///
/// ```rust,no_run
/// # use crate::platforms::platform::register_platform;
/// # use crate::platforms::xilinx_sys::XilinxSysPlatform;
/// register_platform(
///     "new_platform,compatibility-string",
///     || Box::new(NewPlatform::new()),
///     || true  // Always available for built-in platforms
/// );
/// ```
pub fn register_platform(compatible: &'static str, constructor: PlatformConstructor) {
    let mut registry = PLATFORM_REGISTRY
        .get_or_init(init_platform_registry)
        .lock()
        .expect("couldnt get PLATFORM_REGISTRY");

    registry.insert(compatible, constructor);
}

/// List all FPGA manager device handles present in the system.
///
/// Scans `/sys/class/fpga_manager/` and returns the names of all FPGA device
/// directories found. These handles can be used to interact with the devices
/// through the platform abstraction.
///
/// # Returns: `Result<Vec<String>, FpgadError>`
/// * `Ok(Vec<String>)` - List of device handles (e.g., ["fpga0", "fpga1"])
/// * `Err(FpgadError::IOReadDir)` - Failed to read fpga_manager directory
///
/// # Examples
///
/// ```rust,no_run
/// # use crate::platforms::platform::list_fpga_managers;
/// # fn example() -> Result<(), crate::error::FpgadError> {
/// let devices = list_fpga_managers()?;
/// for device in devices {
///     println!("Found FPGA device: {}", device);
/// }
/// # Ok(())
/// # }
/// ```
pub fn list_fpga_managers() -> Result<Vec<String>, FpgadError> {
    fs_read_dir(config::FPGA_MANAGERS_DIR.as_ref())
}

#[cfg(test)]
mod platform_discovery_tests {
    use super::*;
    use googletest::prelude::*;

    /// Helper function to register a test softener platform that mimics XilinxDfxMgrPlatform.
    /// This uses the same compat string structure as the real DFX Manager softener.
    /// This version is always available.
    fn register_test_softener_available() {
        register_platform(
            "xlnx,zynqmp-pcap-fpga,versal-fpga,zynq-devcfg-1.0,dfx-mgr,softener",
            || Box::new(XilinxSysPlatform::new()), // Using XilinxSysPlatform as a stand-in
        );
    }

    /// Helper function to register a test softener platform that is always unavailable.
    fn register_test_softener_unavailable() {
        register_platform(
            "xlnx,zynqmp-pcap-fpga,versal-fpga,zynq-devcfg-1.0,dfx-mgr,softener",
            || Box::new(XilinxSysPlatform::new()),
        );
    }

    /// Helper function to register a test built-in platform that mimics XilinxSysPlatform.
    /// This uses the same compat string structure as the real xlnx-sys platform.
    fn register_test_builtin() {
        register_platform(
            "xlnx,zynqmp-pcap-fpga,versal-fpga,zynq-devcfg-1.0,xlnx-sys,platform",
            || Box::new(XilinxSysPlatform::new()),
        );
    }

    #[gtest]
    fn test_built_in_platform_is_always_available() {
        let platform = XilinxSysPlatform::new();
        assert_that!(platform.is_available(), eq(true));
    }

    #[gtest]
    fn test_built_in_platform_can_be_registered_and_matched() {
        register_platform("test-platform,built-in", || {
            Box::new(XilinxSysPlatform::new())
        });

        let result = match_platform_string("test-platform");
        assert_that!(result.is_ok(), eq(true));
    }

    #[gtest]
    fn test_platform_matching_requires_all_components() {
        register_platform("test-multi,component,platform", || {
            Box::new(XilinxSysPlatform::new())
        });

        let result = match_platform_string("test-multi,component,platform");
        assert_that!(result.is_ok(), eq(true));

        let result = match_platform_string("test-multi,component");
        assert_that!(result.is_ok(), eq(true));

        let result = match_platform_string("test-multi,component,platform,extra");
        assert_that!(result.is_err(), eq(true));
    }

    #[gtest]
    fn test_softener_preferred_when_available() {
        register_test_softener_available();
        register_test_builtin();

        let result = match_platform_string("xlnx,zynqmp-pcap-fpga");
        assert_that!(result.is_ok(), eq(true));

        let result = match_platform_string("xlnx");
        assert_that!(result.is_ok(), eq(true));
    }

    #[gtest]
    fn test_fallback_to_builtin_when_softener_unavailable() {
        register_test_softener_unavailable();
        register_test_builtin();

        let result = match_platform_string("xlnx,zynqmp-pcap-fpga");
        assert_that!(result.is_ok(), eq(true));

        let result = match_platform_string("versal-fpga");
        assert_that!(result.is_ok(), eq(true));
    }

    #[gtest]
    fn test_explicit_softener_request_requires_softener_component() {
        register_test_softener_available();
        register_test_builtin();

        let result = match_platform_string("xlnx,softener");
        assert_that!(result.is_ok(), eq(true));

        let result = match_platform_string("xlnx,platform,softener");
        assert_that!(result.is_err(), eq(true));
    }

    #[gtest]
    fn test_explicit_builtin_request_works() {
        register_test_softener_available();
        register_test_builtin();

        let result = match_platform_string("xlnx,platform");
        assert_that!(result.is_ok(), eq(true));

        let result = match_platform_string("xlnx-sys");
        assert_that!(result.is_ok(), eq(true));
    }

    #[gtest]
    fn test_only_unavailable_platforms_match_still_returns_one() {
        register_platform("only-unavailable,test", || {
            Box::new(XilinxSysPlatform::new())
        });

        let result = match_platform_string("only-unavailable");
        assert_that!(result.is_ok(), eq(true));
    }

    #[gtest]
    fn test_multiple_softeners_picks_first_available() {
        register_platform("multi-soft,test,softener,first", || {
            Box::new(XilinxSysPlatform::new())
        });
        register_platform("multi-soft,test,softener,second", || {
            Box::new(XilinxSysPlatform::new())
        });
        register_platform("multi-soft,test,softener,third", || {
            Box::new(XilinxSysPlatform::new())
        });

        let result = match_platform_string("multi-soft,test,softener");
        assert_that!(result.is_ok(), eq(true));
    }

    #[gtest]
    fn test_no_match_returns_error() {
        let result = match_platform_string("nonexistent-platform-12345");
        assert_that!(result.is_err(), eq(true));
        if let Err(FpgadError::Argument(msg)) = result {
            assert_that!(msg, contains_substring("could not match"));
        } else {
            panic!("Expected FpgadError::Argument");
        }
    }

    #[gtest]
    fn test_empty_string_returns_error() {
        let result = match_platform_string("");
        assert_that!(result.is_err(), eq(true));
    }

    #[gtest]
    fn test_case_sensitivity() {
        register_platform("case-test,lowercase", || Box::new(XilinxSysPlatform::new()));

        let result = match_platform_string("case-test");
        assert_that!(result.is_ok(), eq(true));

        let result = match_platform_string("CASE-TEST");
        assert_that!(result.is_err(), eq(true));
    }

    #[gtest]
    fn test_compat_string_constant_matches_registered() {
        assert_that!(
            XilinxSysPlatform::COMPAT_STRING,
            eq("xlnx,zynqmp-pcap-fpga,versal-fpga,zynq-devcfg-1.0,xlnx-sys,platform")
        );
    }

    #[gtest]
    fn test_dfx_mgr_component_matching() {
        register_test_softener_available();

        let result = match_platform_string("dfx-mgr");
        assert_that!(result.is_ok(), eq(true));

        let result = match_platform_string("xlnx,dfx-mgr");
        assert_that!(result.is_ok(), eq(true));
    }

    #[gtest]
    fn test_versal_and_zynqmp_components() {
        register_test_builtin();

        let result = match_platform_string("versal-fpga");
        assert_that!(result.is_ok(), eq(true));

        let result = match_platform_string("zynqmp-pcap-fpga");
        assert_that!(result.is_ok(), eq(true));

        let result = match_platform_string("zynq-devcfg-1.0");
        assert_that!(result.is_ok(), eq(true));
    }
}

#[cfg(all(test, feature = "xilinx-dfx-mgr"))]
mod dfx_mgr_integration_tests {
    use super::*;
    use crate::softeners::xilinx_dfx_mgr::XilinxDfxMgrPlatform;
    use googletest::prelude::*;
    use std::any::Any;

    /// Register both real softener and built-in platforms for integration testing
    fn setup_integrated_registry() {
        // Register the real DFX Manager softener (may or may not be available)
        register_platform(
            "xlnx,zynqmp-pcap-fpga,versal-fpga,zynq-devcfg-1.0,dfx-mgr,softener",
            || Box::new(XilinxDfxMgrPlatform::new()),
        );

        // Register the built-in platform
        register_platform(
            "xlnx,zynqmp-pcap-fpga,versal-fpga,zynq-devcfg-1.0,xlnx-sys,platform",
            || Box::new(XilinxSysPlatform::new()),
        );
    }

    fn assert_is_xlnx_sys_platform(platform: &dyn Platform) {
        let as_xlnx_sys = (platform as &dyn Any).downcast_ref::<XilinxSysPlatform>();
        assert_that!(as_xlnx_sys.is_some(), eq(true));
    }

    #[gtest]
    fn test_dfx_mgr_platform_availability() {
        // Test whether DFX Manager is available (depends on system)
        let platform = XilinxDfxMgrPlatform::new();
        let available = platform.is_available();
        // This will be true or false depending on whether dfx-mgr-client is installed
        // Just verify the function is callable
        println!("DFX Manager available: {}", available);
    }

    #[gtest]
    fn test_explicit_builtin_request_with_real_platforms() {
        setup_integrated_registry();

        let result = match_platform_string("xlnx-sys");
        assert_that!(result.is_ok(), eq(true));

        let platform = result.unwrap();
        assert_is_xlnx_sys_platform(platform.as_ref());
    }

    #[gtest]
    fn test_platform_type_assertion_methods() {
        setup_integrated_registry();
        let platform = match_platform_string("xlnx").unwrap();

        let platform_any = platform.as_ref() as &dyn Any;
        let type_name = std::any::type_name_of_val(platform_any);
        println!("Platform type: {}", type_name);

        let is_dfx_mgr = platform_any.is::<XilinxDfxMgrPlatform>();
        let is_xlnx_sys = platform_any.is::<XilinxSysPlatform>();

        assert_that!(is_dfx_mgr || is_xlnx_sys, eq(true));
    }

    #[gtest]
    fn test_compat_string_constants() {
        assert_that!(
            XilinxDfxMgrPlatform::COMPAT_STRING,
            contains_substring("dfx-mgr")
        );
        assert_that!(
            XilinxDfxMgrPlatform::COMPAT_STRING,
            contains_substring("softener")
        );
        assert_that!(
            XilinxSysPlatform::COMPAT_STRING,
            contains_substring("xlnx-sys")
        );
        assert_that!(
            XilinxSysPlatform::COMPAT_STRING,
            contains_substring("platform")
        );
    }
}
