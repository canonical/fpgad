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
//! ```rust,ignore
//! let platform = platform_for_known_platform("universal")?;
//! let fpga = platform.fpga("fpga0")?;
//! let state = fpga.state()?;
//! ```

use crate::comm::dbus::validate_device_handle;
use crate::config;
use crate::error::FpgadError;
use crate::platforms::platform::{Fpga, OverlayHandler, Platform, list_fpga_managers};
use crate::platforms::universal_components::universal_fpga::UniversalFPGA;
use crate::platforms::universal_components::universal_overlay_handler::UniversalOverlayHandler;
use crate::system_io::{fs_read, fs_read_dir, fs_write, fs_write_bytes};
use fpgad_macros::platform;
use log::{error, info, trace, warn};
use std::path;
use std::path::{Component, Path, PathBuf};
use std::sync::OnceLock;
use zbus::fdo;

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
    /// use crate::platforms::universal::UniversalPlatform;
    ///
    /// let platform = platform_for_known_platform("universal")?;
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
        if overlay_handle.is_empty() {
            return Err(FpgadError::Argument(
                "An overlay handle is required. Provided overlay handle is empty.".into(),
            ));
        }

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

    fn status_message(&self) -> Result<String, FpgadError> {
        let mut ret_string = String::from(
            "---- DEVICES ----\n\
    | dev | platform | state |\n",
        );

        for device in list_fpga_managers()? {
            let state = self.fpga(&device)?.state()?;
            ret_string += format!(
                "| {} | {} | {} |\n",
                device,
                self.platform_compat_string(),
                state
            )
            .as_str();
        }
        ret_string += "\n---- OVERLAYS ----\n\
                   | overlay | status |\n";

        // If overlayfs not enabled, or interface not connected this will be an error.
        for overlay in fs_read_dir(config::OVERLAY_CONTROL_DIR.as_ref())? {
            let status = self.overlay_handler(&overlay)?.status()?;
            ret_string.push_str(format!("| {overlay} | {status} |\n").as_ref());
        }

        Ok(ret_string)
    }

    fn platform_compat_string(&self) -> String {
        "universal".into()
    }
}

/// Validates that a property path is constrained under the fpga manager directory and does not contain explicit parent traversal segments.
/// This is used to validate paths for all read/write property access methods in the control and status interfaces.
///
/// # Arguments
/// * `property_path` - The property path to validate as a Path.
///
/// # Returns: `Result<PathBuf, FpgadError>`
/// A `PathBuf` representing the validated property path if it is valid, or a `FpgadError` if the path is invalid.
///
/// # Examples
/// ```rust,no_run
/// let valid_path = validate_property_path("/sys/class/fpga_manager/fpga0/name")?;
/// assert_eq!(valid_path.to_string_lossy(), "/sys/class/fpga_manager/fpga0/name");
/// let invalid_path = validate_property_path("/sys/class/fpga_manager/../etc/passwd");
/// assert!(invalid_path.is_err());
/// ```
pub(crate) fn validate_property_path(property_path: &Path) -> Result<PathBuf, FpgadError> {
    validate_property_path_with_base(property_path, Path::new(config::FPGA_MANAGERS_DIR))
}

/// Validates that a property path is constrained under a specified base path and does not contain
/// explicit parent traversal segments. This is a more general version of `validate_property_path` which
/// can be used to validate paths under different base directories, such as the firmware lookup control path.
///
/// # Arguments
/// * `property_path` - The property path to validate as a Path.
/// * `base_path` - The base path under which the property path must be constrained.
///
/// # Returns: `Result<PathBuf, FpgadError>`
/// A `PathBuf` representing the validated property path if it is valid, or a `FpgadError` if the path is invalid.
///
/// # Examples
/// ```rust,no_run
/// let valid_path = validate_property_path_with_base("/sys/class/fpga_manager/fpga0/name", Path::new("/sys/class/fpga_manager/"))?;
/// assert_eq!(valid_path.to_string_lossy(), "/sys/class/fpga_manager/fpga0/name");
/// let invalid_path = validate_property_path_with_base("/sys/class/fpga_manager/../etc/passwd", Path::new("/sys/class/fpga_manager/"));
/// assert!(invalid_path.is_err());
/// ```
fn validate_property_path_with_base(
    property_path: &Path,
    base_path: &Path,
) -> Result<PathBuf, FpgadError> {
    let property_path = PathBuf::from(property_path);
    if property_path
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(FpgadError::Argument(format!(
            "Cannot access property {}: path traversal ('..') is not allowed",
            property_path.display()
        )));
    }

    let canonical_base = path::absolute(base_path).map_err(|e| {
        FpgadError::Argument(format!(
            "Cannot access property {}: failed to resolve base path {}: {}",
            property_path.display(),
            base_path.display(),
            e
        ))
    })?;
    let canonical_property = path::absolute(&property_path).map_err(|e| {
        FpgadError::Argument(format!(
            "Cannot access property {}: failed to resolve property path: {}",
            property_path.display(),
            e
        ))
    })?;

    if !canonical_property.starts_with(&canonical_base) {
        return Err(FpgadError::Argument(format!(
            "Cannot access property {}: resolved path {} is outside {}",
            property_path.display(),
            canonical_property.display(),
            canonical_base.display()
        )));
    }

    Ok(canonical_property)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ReadSubCommand {
    ReadProp,
    ReadFlags,
}

impl ReadSubCommand {
    pub fn as_str(self) -> &'static str {
        match self {
            ReadSubCommand::ReadFlags => "read_flags",
            ReadSubCommand::ReadProp => "read_property",
        }
    }
}

impl std::str::FromStr for ReadSubCommand {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "read_flags" => Ok(ReadSubCommand::ReadFlags),
            "read_property" => Ok(ReadSubCommand::ReadProp),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum WriteSubCommand {
    WriteFlags,
    WriteProp,
    WriteByte,
}

impl WriteSubCommand {
    pub fn as_str(self) -> &'static str {
        match self {
            WriteSubCommand::WriteFlags => "write_flags",
            WriteSubCommand::WriteProp => "write_property",
            WriteSubCommand::WriteByte => "write_property_bytes",
        }
    }
}

impl std::str::FromStr for WriteSubCommand {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "write_flags" => Ok(WriteSubCommand::WriteFlags),
            "write_property" => Ok(WriteSubCommand::WriteProp),
            "write_property_bytes" => Ok(WriteSubCommand::WriteByte),
            _ => Err(()),
        }
    }
}

/// Read the current contents of an FPGA device property, e.g. "name". The property path must be a subdirectory of the fpga manager directory (typically, /sys/class/fpga_manager/)
///
/// # Arguments
///
/// * `property_path_str`: path to the variable to read e.g. /sys/class/fpga_manager/fpga0/name
///
/// # Returns: `Result<String, FpgadError>`
/// * `String` - the contents of the property path
///
/// * `FpgadError::Argument` if the path is not found within the compile time [config::FPGA_MANAGERS_DIR]
///
/// # Examples
///
/// ```rust,no_run
/// let device_name = fs_read_property("/sys/class/fpga_manager/fpga0/name")?;
/// assert_eq!(device_name, "Xilinx ZynqMP FPGA Manager\n")
/// ```
pub fn fs_read_property(property_path_str: &str) -> Result<String, FpgadError> {
    let property_path = validate_property_path(Path::new(property_path_str))?;
    fs_read(&property_path)
}

/// Read an arbitrary FPGA device property from `/sys/class/fpga_manager/<device>/`.
///
/// # Arguments
///
/// * `property_path_str`: Full path to the property file.
///
/// # Returns: `Result<String, Error>`
/// * `Ok(String)` – Contents of the property file.
/// * `Err(fdo::Error)` If the property cannot be read.
///
/// # Examples
///
/// ```rust,no_run
/// let name = status_interface
///     .universal("read_property", "/sys/class/fpga_manager/fpga0/name")
///     .await?;
/// assert_eq!(name, "Xilinx ZynqMP FPGA Manager\n");
/// ```
///
pub fn read_property(property_path_str: &str) -> Result<String, fdo::Error> {
    info!("read_property called with property_path_str: {property_path_str}");
    Ok(fs_read_property(property_path_str)?)
}

/// Write a string value to an arbitrary FPGA device property.
///
/// # Arguments
///
/// * `property_path_str`: Full path under [crate::config::FPGA_MANAGERS_DIR].
/// * `data`: String data to write.
///
/// # Returns: `Result<String, Error>`
///
/// * `Ok(String)` – Confirmation of written data.
/// * `Err(fdo::Error)` if path is outside FPGA managers, or if the writing failed for any
///   other reason
///
/// **Notes:**
///
/// * Path must be under [crate::config::FPGA_MANAGERS_DIR] - determined at compile time.
///
/// # Examples
///
/// ```
/// let result = control_interface
///     .universal(
///         "write_property",
///         "/sys/class/fpga_manager/fpga0/key",
///         "BADBADBADBAD")
///     .await?;
/// assert_eq!(result, "BADBADBADBAD written to /sys/class/fpga_manager/fpga0/key");
/// ```
pub fn write_property(property_path_str: &str, data: &str) -> Result<String, fdo::Error> {
    info!("write_property called with property_path_str: {property_path_str} and data: {data}");
    let property_path = validate_property_path(Path::new(property_path_str))?;
    fs_write(&property_path, false, data)?;
    Ok(format!("{data} written to {property_path_str}"))
}

/// Write raw bytes to an arbitrary FPGA device property.
///
/// # Arguments
///
/// * `property_path_str`: Full path under [crate::config::FPGA_MANAGERS_DIR].
/// * `data`: Byte array to write.
///
/// # Returns: `Result<String, Error>`
///
/// * `Ok(String)` – Confirmation of written data.
/// * `Err(fdo::Error)` if path is outside FPGA managers, or if the writing failed for any
///   other reason
///
/// **Notes:**
///
/// * Path must be under [crate::config::FPGA_MANAGERS_DIR] - determined at compile time.
///
/// # Examples
///
/// ```
/// let result = control_interface
///     .universal(
///         "write_property_bytes",
///         "/sys/class/fpga_manager/fpga0/key",
///         "BADBAD")
///     .await?;
/// assert_eq!(result, "Byte string successfully written to /sys/class/fpga_manager/fpga0/key");
/// ```
fn write_property_bytes(property_path_str: &str, data: &[u8]) -> Result<String, fdo::Error> {
    info!("write_property called with property_path_str: {property_path_str} and data: {data:?}");
    let property_path = validate_property_path(Path::new(property_path_str))?;
    fs_write_bytes(&property_path, false, data)?;
    Ok(format!(
        "Byte string successfully written to {property_path_str}"
    ))
}

/// Read the current programming flags from sysfs.
///
/// Reads `/sys/class/fpga_manager/<device>/flags`, parses the hexadecimal string
/// (format: "0x...", or undecorated), and returns the flags as u32.
///
/// # Returns: `Result<u32, FpgadError>`
/// * `Ok(u32)` - Current flags value
/// * `Err(FpgadError::IORead)` - Failed to read flags file
/// * `Err(FpgadError::Flag)` - Failed to parse hexadecimal value
fn flags(fpga: &UniversalFPGA) -> Result<u32, FpgadError> {
    let flag_path = Path::new(config::FPGA_MANAGERS_DIR)
        .join(fpga.device_handle())
        .join("flags");
    let contents = fs_read(&flag_path)?;
    let trimmed = contents.trim().trim_start_matches("0x");
    u32::from_str_radix(trimmed, 16).map_err(|_| FpgadError::Flag("Parsing flags failed".into()))
}

/// Set the programming flags in sysfs.
///
/// Writes the flags to `/sys/class/fpga_manager/<device>/flags` in undecorated
/// hexadecimal (decimal `32` -> undecorated hex `20`) and verifies that the write
/// succeeded by reading the value back.
/// Also checks and logs the FPGA state after setting flags.
///
/// # Arguments
///
/// * `flags` - The flags value to set
///
/// # Returns: `Result<(), FpgadError>`
/// * `Ok(())` - Flags set and verified successfully
/// * `Err(FpgadError::IOWrite)` - Failed to write flags file
/// * `Err(FpgadError::IORead)` - Failed to read back flags or state
/// * `Err(FpgadError::Flag)` - Read-back value doesn't match written value
fn set_flags(fpga: &UniversalFPGA, new_flags: u32) -> Result<String, FpgadError> {
    let device_handle = fpga.device_handle();
    let flag_path = Path::new(config::FPGA_MANAGERS_DIR)
        .join(device_handle)
        .join("flags");

    trace!("Writing '0x{new_flags:X}' to '{flag_path:?}'");
    if let Err(e) = fs_write(&flag_path, false, format!("0x{new_flags:X}")) {
        error!("Failed to read state.");
        return Err(e);
    }

    match fpga.state() {
        Ok(state) => match state.as_str() {
            "operating" => {
                info!(
                    "{}'s state is 'operating' after writing flags.",
                    device_handle
                )
            }
            _ => {
                warn!(
                    "{}'s state is '{}' after writing flags.",
                    device_handle, state
                );
            }
        },
        Err(e) => return Err(e),
    };

    let returned_flags = flags(fpga)?;
    if returned_flags == new_flags {
        Ok(format!(
            "Flags set to '0x{:X}' for '{}'",
            new_flags, device_handle
        ))
    } else {
        Err(FpgadError::Flag(format!(
            "Setting flags of '{}' to '0x{:X}' failed. Resulting flag was '0x{:X}'",
            device_handle, new_flags, returned_flags
        )))
    }
}

/// Parse a hexadecimal string into raw bytes.
///
/// Accepts an ASCII string representing a hexadecimal byte stream. The input may
/// contain optional whitespace separators and may optionally include `0x` prefixes.
///
/// All whitespace is removed and all occurrences of 0x are removed from the input prior to parsing.
/// The remaining string is treated as a continuous hexadecimal stream.
///
/// If the resulting stream has an odd number of hexadecimal digits, a leading `0`
/// is implicitly prepended to allow nibble-aligned parsing.
///
/// Each pair of hexadecimal digits is then converted into a single `u8`.
///
/// Supported input formats:
/// - `"00 04 02 20 20"` → `[0x00, 0x04, 0x02, 0x20, 0x20]`
/// - `"0x00 0x04 0x02 0x20 0x20"` → `[0x00, 0x04, 0x02, 0x20, 0x20]`
/// - `"0004022020"` → `[0x00, 0x04, 0x02, 0x20, 0x20]`
/// - `"AA"` → `[0xAA]`
/// - `"aa"` → `[0xAA]`
/// - `"1FF"` → `[0x01, 0xFF]`
/// - `"0x1FF"` → `[0x01, 0xFF]`
/// - `"0x0x10"` → `[0x10]`
///
/// Note:
/// - Whitespace is ignored and only acts as a separator.
/// - Optional `0x` prefixes are stripped before parsing.
/// - Input is treated as a continuous hex stream after normalization.
/// - Odd-length hex streams are left-padded with a single `0` before decoding.
/// - Each resulting byte must be within `0x00..=0xFF`.
///
/// # Arguments
///
/// * `value_str` - A hexadecimal-formatted ASCII string representing a byte stream
///
/// # Returns: `Result<Vec<u8>, FpgadError>`
///
/// * `Ok(Vec<u8>)` - Successfully parsed byte vector from hex stream
/// * `Err(FpgadError::Argument)` - Input contains invalid hexadecimal characters
///   or malformed byte values that cannot be parsed into `u8`
fn hex_from_string(value_str: &str) -> Result<Vec<u8>, FpgadError> {
    let clean: String = value_str
        .split_whitespace()
        .collect::<String>()
        .to_lowercase()
        .replace("0x", "");

    let mut chars = clean.chars().collect::<Vec<_>>();

    // If odd number of nibbles, left-pad with '0'
    if chars.len() % 2 != 0 {
        chars.insert(0, '0');
    }

    chars
        .chunks(2)
        .map(|chunk| {
            let s: String = chunk.iter().collect();

            u8::from_str_radix(&s, 16)
                .map_err(|e| FpgadError::Argument(format!("Invalid hex byte '{s}': {e}")))
        })
        .collect()
}

/// Dispatches write subcommands for the universal DBus control API.
///
/// Supported `sub_cmd_str` values:
/// - `write_flags` to set FPGA flags for a given device handle or `property_path`.
///   Uses the validated device handle to create an FPGA object
/// - `write_property` to write string data to a validated sysfs property path
/// - `write_property_bytes` to write the raw bytes of `value_str` to a validated sysfs property path
pub fn universal_write_handler(
    sub_cmd_str: &str,
    property_path: &str,
    value_str: &str,
) -> Result<String, fdo::Error> {
    match sub_cmd_str.parse::<WriteSubCommand>() {
        Ok(WriteSubCommand::WriteFlags) => {
            // extracts device handle e.g. "fpga0" from "/sys/class/fpga_manager/fpga0/flags" or
            // just treats input as a device handle
            let device_handle = if let Some(rest) =
                property_path.strip_prefix(config::FPGA_MANAGERS_DIR)
            {
                let handle = rest
                        .split('/')
                        .next()
                        .filter(|s| !s.is_empty())
                        .ok_or_else(|| {
                            FpgadError::Argument(format!(
                                "Invalid FPGA manager path '{property_path}', could not extract device handle"
                            ))
                        })?;
                // todo(artie): decide whether to check that "flags" is the final part or just allow
                validate_device_handle(handle)?;
                handle
            } else {
                validate_device_handle(property_path)?;
                property_path
            };

            let fpga = UniversalFPGA::new(device_handle);

            let trimmed = value_str.trim();
            let parsed_flags = if let Some(hex) = trimmed
                .strip_prefix("0x")
                .or_else(|| trimmed.strip_prefix("0X"))
            {
                u32::from_str_radix(hex, 16)
            } else {
                trimmed
                    .parse::<u32>()
                    .or_else(|_| u32::from_str_radix(trimmed, 16))
            }
            .map_err(|e| {
                FpgadError::Argument(format!(
                    "Invalid flags value '{value_str}': expected decimal or hexadecimal u32 ({e})"
                ))
            })?;

            set_flags(&fpga, parsed_flags).map_err(Into::into)
        }
        Ok(WriteSubCommand::WriteProp) => {
            validate_property_path(Path::new(property_path))?;
            write_property(property_path, value_str)
        }
        Ok(WriteSubCommand::WriteByte) => {
            validate_property_path(Path::new(property_path))?;
            // todo: consider using "hex" which is available from cargo as v0.4.0
            let hex_data = hex_from_string(value_str)?;
            write_property_bytes(property_path, &hex_data)
        }
        Err(()) => {
            Err(FpgadError::Argument(format!("Unknown write subcommand '{sub_cmd_str}'")).into())
        }
    }
}

/// Dispatches read subcommands for the universal DBus status API.
///
/// Supported `sub_cmd_str` values:
/// - `read_property` to read a validated sysfs property path
/// - `read_flags` to read FPGA flags for a validated device handle
pub fn universal_read_handler(
    sub_cmd_str: &str,
    property_path: &str,
) -> Result<String, fdo::Error> {
    match sub_cmd_str.parse::<ReadSubCommand>() {
        Ok(ReadSubCommand::ReadFlags) => {
            validate_device_handle(property_path)?;
            let fpga = UniversalFPGA::new(property_path);
            Ok(flags(&fpga)?.to_string())
        }
        Ok(ReadSubCommand::ReadProp) => {
            validate_property_path(Path::new(property_path))?;
            read_property(property_path)
        }
        Err(()) => {
            Err(FpgadError::Argument(format!("Unknown read subcommand '{sub_cmd_str}'")).into())
        }
    }
}

#[cfg(test)]
mod test_validate_property_path {
    use crate::platforms::universal::validate_property_path_with_base;
    use googletest::prelude::*;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_test_dir(test_name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("fpgad_validate_property_path_{test_name}_{nanos}"))
    }

    #[gtest]
    fn should_pass_valid_path() {
        let root = unique_test_dir("valid_path");
        let base = root.join("fpga_manager");
        let property = base.join("fpga0").join("name");

        fs::create_dir_all(property.parent().expect("property should have parent"))
            .expect("create parent dirs");
        fs::write(&property, "name\n").expect("create property file");

        let expected = fs::canonicalize(&property).expect("canonicalize property");
        let result = validate_property_path_with_base(&property, &base);

        fs::remove_dir_all(root).expect("cleanup temp dirs");
        assert_that!(&result, ok(eq(&expected)));
    }

    #[gtest]
    fn should_fail_for_path_outside_fpga_dir() {
        let root = unique_test_dir("outside_base");
        let base = root.join("fpga_manager");
        let outside = root.join("outside").join("evil_file.sh");

        fs::create_dir_all(&base).expect("create base dir");
        fs::create_dir_all(outside.parent().expect("outside should have parent"))
            .expect("create outside dir");
        fs::write(&outside, "evil\n").expect("create outside file");

        let result = validate_property_path_with_base(&outside, &base);

        fs::remove_dir_all(root).expect("cleanup temp dirs");
        assert_that!(&result, err(displays_as(contains_substring("is outside"))));
    }

    #[gtest]
    fn should_fail_for_root_path_traversal() {
        let root = unique_test_dir("root_traversal");
        let base = root.join("fpga_manager");
        fs::create_dir_all(&base).expect("create base dir");

        let traversal = base.join("..").join("outside").join("evil_file.sh");
        let result = validate_property_path_with_base(&traversal, &base);

        fs::remove_dir_all(root).expect("cleanup temp dirs");
        assert_that!(
            &result,
            err(displays_as(contains_substring("path traversal")))
        );
    }

    #[gtest]
    fn should_fail_for_device_path_traversal() {
        let root = unique_test_dir("device_traversal");
        let base = root.join("fpga_manager");
        fs::create_dir_all(base.join("fpga0")).expect("create fpga0 dir");

        let traversal = base.join("fpga0").join("..").join("name");
        let result = validate_property_path_with_base(&traversal, &base);

        fs::remove_dir_all(root).expect("cleanup temp dirs");
        assert_that!(
            &result,
            err(displays_as(contains_substring("path traversal")))
        );
    }

    #[cfg(unix)]
    #[gtest]
    fn should_allow_symlink_path_without_resolution() {
        use std::os::unix::fs::symlink;
        use std::path::absolute;

        let root = unique_test_dir("symlink_escape");
        let base = root.join("fpga_manager");
        let outside = root.join("outside");
        let link_target_file = outside.join("escaped_name");
        let fpga0_dir = base.join("fpga0");
        let link_in_base = fpga0_dir.join("link_outside");

        fs::create_dir_all(&fpga0_dir).expect("create fpga0 dir");
        fs::create_dir_all(&outside).expect("create outside dir");
        fs::write(&link_target_file, "evil\n").expect("create outside target file");
        symlink(&outside, &link_in_base).expect("create symlink escaping base");

        let escaped_path = link_in_base.join("escaped_name");
        let expected = absolute(&escaped_path).expect("resolve absolute escaped path");
        let result = validate_property_path_with_base(&escaped_path, &base);

        fs::remove_dir_all(root).expect("cleanup temp dirs");
        assert_that!(&result, ok(eq(&expected)));
    }
}

#[cfg(test)]
mod test_hex_from_string {
    use super::*;
    use googletest::prelude::*;

    #[test]
    fn parses_plain_hex_bytes() {
        let result = hex_from_string("00 04 02 20 20");
        let expected: Vec<u8> = vec![0, 4, 2, 32, 32];
        assert_that!(result, ok(eq(&expected)));
    }

    #[test]
    fn parses_continuous_single_byte() {
        let result = hex_from_string("AA");
        let expected: Vec<u8> = vec![0xAA];
        assert_that!(result, ok(eq(&expected)));
    }

    #[test]
    fn parses_lowercase_hex() {
        let result = hex_from_string("aa bb cc");
        let expected: Vec<u8> = vec![0xAA, 0xBB, 0xCC];
        assert_that!(result, ok(eq(&expected)));
    }

    #[test]
    fn parses_with_0x_prefix() {
        let result = hex_from_string("0x00 0x04 0x02 0x20 0x20");
        let expected: Vec<u8> = vec![0, 4, 2, 32, 32];
        assert_that!(result, ok(eq(&expected)));
    }

    #[test]
    fn parses_mixed_prefix_and_plain_tokens() {
        let result = hex_from_string("0x00 04 0x02 20");
        let expected: Vec<u8> = vec![0, 4, 2, 32];
        assert_that!(result, ok(eq(&expected)));
    }

    #[test]
    fn ignores_extra_whitespace() {
        let result = hex_from_string("   00   04   02   20   ");
        let expected: Vec<u8> = vec![0, 4, 2, 32];
        assert_that!(result, ok(eq(&expected)));
    }

    #[test]
    fn rejects_invalid_hex_characters() {
        let result = hex_from_string("00 GG 02");
        assert_that!(
            result,
            err(pat!(FpgadError::Argument(contains_substring(
                "Invalid hex"
            ))))
        );
    }

    #[test]
    fn parses_odd_length_as_stream() {
        let result = hex_from_string("1FF");
        let expected: Vec<u8> = vec![0x01, 0xFF];
        assert_that!(result, ok(eq(&expected)));
    }

    #[test]
    fn parses_no_spaces() {
        let result = hex_from_string("DEADBEEF");
        let expected: Vec<u8> = vec![0xDE, 0xAD, 0xBE, 0xEF];
        assert_that!(result, ok(eq(&expected)));
    }

    #[test]
    fn empty_input_returns_empty_vec() {
        let result = hex_from_string("");
        let expected: Vec<u8> = vec![];
        assert_that!(result, ok(eq(&expected)));
    }
}
