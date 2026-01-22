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
//! 3. Falling back to the Universal platform if no match is found
//!
//! # Platform Registration
//!
//! Platforms register themselves at daemon startup using the [`register_platform`] function
//! and are defined using the `#[platform]` macro. Compatibility strings can include
//! comma-separated components, all of which must match for a platform to be selected.
//! Platforms and softeners are included or not excluded using cargo "features".
//! See [`softeners`](../../softeners/index.html) for more details.
//!
//! TODO(Artie): Add examples of how to use the getters for platforms with and without knowing the platform string? - could be called "# Fetching platforms"
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
//!     UniversalPlatform::register_platform();
//! }
//! ```
//!

use crate::config;
use crate::error::FpgadError;
use crate::platforms::universal::UniversalPlatform;
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

    /// Get the current programming flags for the FPGA device.
    ///
    /// # Returns: `Result<u32, FpgadError>`
    /// * `Ok(u32)` - Current flags value
    /// * `Err(FpgadError::IORead)` - Failed to read flags file
    /// * `Err(FpgadError::Flag)` - Failed to parse flags value
    fn flags(&self) -> Result<u32, FpgadError>;

    /// Set the programming flags for the FPGA device.
    ///
    /// # Arguments
    ///
    /// * `flags` - The flags value to set
    ///
    /// # Returns: `Result<(), FpgadError>`
    /// * `Ok(())` - Flags set successfully
    /// * `Err(FpgadError::IOWrite)` - Failed to write flags file
    fn set_flags(&self, flags: u32) -> Result<(), FpgadError>;

    /// Load a bitstream firmware file to the FPGA device.
    ///
    /// # Arguments
    ///
    /// * `bitstream_path_rel` - Path to the bitstream file relative to whatever path the lookup starts in. For universal, this is the firmware search path
    ///
    /// # Returns: `Result<(), FpgadError>`
    /// * `Ok(())` - Bitstream loaded successfully
    /// * `Err(FpgadError::IOWrite)` - Failed to write firmware file
    /// * `Err(FpgadError::FPGAState)` - FPGA not in correct state for loading
    #[allow(dead_code)]
    fn load_firmware(&self, bitstream_path_rel: &Path) -> Result<(), FpgadError>;
}

/// Trait for managing device tree overlays.
pub trait OverlayHandler {
    /// Apply a device tree overlay from a source file.
    ///
    /// # Arguments
    ///
    /// * `source_path` - Path to the `.dtbo` overlay binary file
    ///
    /// # Returns: `Result<(), FpgadError>`
    /// * `Ok(())` - Overlay applied successfully
    /// * `Err(FpgadError::IOWrite)` - Failed to write overlay
    /// * `Err(FpgadError::OverlayStatus)` - Overlay application failed
    fn apply_overlay(&self, source_path: &Path) -> Result<(), FpgadError>;

    /// Remove a device tree overlay.
    ///
    /// # Returns: `Result<(), FpgadError>`
    /// * `Ok(())` - Overlay removed successfully
    /// * `Err(FpgadError::IODelete)` - Failed to remove overlay directory
    fn remove_overlay(&self) -> Result<(), FpgadError>;

    /// Get the required FPGA flags, however they may be provided.
    ///
    /// # Returns: `Result<isize, FpgadError>`
    /// * `Ok(isize)` - Required flags value
    /// * `Err(FpgadError)` - Failed to parse overlay or extract flags
    #[allow(dead_code)]
    fn required_flags(&self) -> Result<isize, FpgadError>;

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
/// # use daemon::platforms::platform::Platform;
/// #
/// # fn example(platform: &dyn Platform) -> Result<(), daemon::error::FpgadError> {
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
}

/// Match a platform compatibility string to a registered platform.
///
/// This function implements the platform matching algorithm that searches the registry
/// for a platform whose compatibility string matches all components in the provided
/// string. The matching is done by splitting both strings on commas and ensuring ***all***
/// components in the query string are present in the registered compatibility string.
///
/// # Algorithm
///
/// 1. Split the registered compatibility string into components: `"xlnx,zynqmp-pcap-fpga"` → `["xlnx", "zynqmp-pcap-fpga"]`
/// 2. Split the query string into components: `"xlnx"` → `["xlnx"]`
/// 3. Check if all query components exist in the registered components
/// 4. Return the first matching platform constructor
///
/// # Arguments
///
/// * `platform_string` - Comma-separated compatibility string to match
///
/// # Returns: `Result<Box<dyn Platform>, FpgadError>`
/// * `Ok(Box<dyn Platform>)` - Newly constructed platform instance
/// * `Err(FpgadError::Internal)` - Registry not initialized or lock failure
/// * `Err(FpgadError::Argument)` - No matching platform found
///
/// # Examples
///
/// ```rust,no_run
/// # use daemon::platforms::platform::match_platform_string;
/// # fn example() -> Result<(), daemon::error::FpgadError> {
/// // Match a single component
/// let platform = match_platform_string("xlnx")?;
///
/// // Match multiple components
/// let platform = match_platform_string("xlnx,zynqmp-pcap-fpga")?;
/// # Ok(())
/// # }
/// ```
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

/// Discover the appropriate platform for a device by reading its compatibility string.
///
/// This function reads the device tree compatible string from the device's sysfs
/// `of_node/compatible` file and attempts to match it to a registered platform.
/// If no match is found, it falls back to the Universal platform with a warning.
///
/// # Arguments
///
/// * `device_handle` - The device handle (e.g., "fpga0")
///
/// # Returns: `Result<Box<dyn Platform>, FpgadError>`
/// * `Ok(Box<dyn Platform>)` - Platform instance (matched or Universal fallback)
/// * `Err(FpgadError::Argument)` - Failed to read compatibility string
///
/// # Examples
///
/// ```rust,no_run
/// # use daemon::platforms::platform::discover_platform;
/// # fn example() -> Result<(), daemon::error::FpgadError> {
/// let platform = discover_platform("fpga0")?;
/// let fpga = platform.fpga("fpga0")?;
/// # Ok(())
/// # }
/// ```
fn discover_platform(device_handle: &str) -> Result<Box<dyn Platform>, FpgadError> {
    let compat_string = read_compatible_string(device_handle)?;
    trace!("Found compatibility string: '{compat_string}'");

    Ok(match_platform_string(&compat_string).unwrap_or({
        warn!("{compat_string} not supported. Defaulting to Universal platform.");
        Box::new(UniversalPlatform::new())
    }))
}

/// Read the device tree compatible string for an FPGA device.
///
/// Reads the compatibility string from `/sys/class/fpga_manager/<device>/of_node/compatible`.
/// This string identifies the hardware and is used for platform matching. The function
/// handles null-terminated strings that some drivers write to sysfs.
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
/// # use daemon::platforms::platform::read_compatible_string;
/// # fn example() -> Result<(), daemon::error::FpgadError> {
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
/// # use daemon::platforms::platform::platform_from_compat_or_device;
/// # fn example() -> Result<(), daemon::error::FpgadError> {
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
/// # use daemon::platforms::platform::platform_for_known_platform;
/// # fn example() -> Result<(), daemon::error::FpgadError> {
/// let platform = platform_for_known_platform("xlnx,zynqmp-pcap-fpga")?;
/// # Ok(())
/// # }
/// ```
pub fn platform_for_known_platform(platform_string: &str) -> Result<Box<dyn Platform>, FpgadError> {
    match_platform_string(platform_string)
}

/// Initialize the platform registry.
///
/// Creates a new empty mutex-protected HashMap for storing platform constructors.
/// This function is called automatically by [`register_platform`] via `OnceLock::get_or_init`.
///
/// # Returns: `Mutex<HashMap<&'static str, PlatformConstructor>>`
/// * Empty mutex-protected HashMap ready for platform registration
pub fn init_platform_registry() -> Mutex<HashMap<&'static str, PlatformConstructor>> {
    Mutex::new(HashMap::new())
}

/// Register a platform implementation in the global registry.
///
/// Adds a platform constructor to the registry with an associated compatibility string.
/// The compatibility string should be a comma-separated list of components that match
/// the device tree compatible property. Platforms are typically registered at daemon
/// startup before any devices are discovered.
///
/// # Arguments
///
/// * `compatible` - Compatibility string (e.g., "xlnx,zynqmp-pcap-fpga")
/// * `constructor` - Function that creates a new platform instance
///
/// # Panics
///
/// Panics if the registry lock is poisoned (should never happen in normal operation).
///
/// # Examples
///
/// ```rust,no_run
/// # use daemon::platforms::platform::register_platform;
/// # use daemon::platforms::universal::UniversalPlatform;
/// register_platform("new_platform,compatibility-string", || {
///     Box::new(NewPlatform::new())
/// });
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
/// # use daemon::platforms::platform::list_fpga_managers;
/// # fn example() -> Result<(), daemon::error::FpgadError> {
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
