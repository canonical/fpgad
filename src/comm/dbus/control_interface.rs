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

use crate::config::FPGA_MANAGERS_DIR;
use crate::error::FpgadError;
use crate::platforms::platform::{platform_for_known_platform, platform_from_compat_or_device};
use crate::system_io::{
    extract_path_and_filename, fs_write, validate_device_handle, write_firmware_source_dir,
};
use log::trace;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard, OnceCell};
use zbus::{fdo, interface};

static WRITE_LOCK: OnceCell<Arc<Mutex<()>>> = OnceCell::const_new();

async fn get_write_lock_guard() -> MutexGuard<'static, ()> {
    let lock = WRITE_LOCK
        .get_or_init(|| async { Arc::new(Mutex::new(())) })
        .await;
    lock.lock().await
}

fn make_firmware_pair(
    source_path: &Path,
    firmware_path: &Path,
) -> Result<(PathBuf, PathBuf), FpgadError> {
    if firmware_path.as_os_str().is_empty() {
        return extract_path_and_filename(source_path);
    }

    if let Ok(suffix) = source_path.strip_prefix(firmware_path) {
        // Remove leading '/' if present
        let cleaned_suffix_path = suffix
            .components()
            .skip_while(|c| matches!(c, Component::RootDir))
            .collect::<PathBuf>();
        Ok((firmware_path.to_path_buf(), cleaned_suffix_path))
    } else {
        Err(FpgadError::Argument(format!(
            "Could not find {source_path:?} inside {firmware_path:?}"
        )))
    }
}

pub struct ControlInterface {}
#[interface(name = "com.canonical.fpgad.control")]
impl ControlInterface {
    async fn set_fpga_flags(
        &self,
        platform_string: &str,
        device_handle: &str,
        flags: u32,
    ) -> Result<String, fdo::Error> {
        trace!("set_fpga_flags called with name: {device_handle} and flags: {flags}");
        validate_device_handle(device_handle)?;
        let platform = platform_from_compat_or_device(platform_string, device_handle)?;
        platform.fpga(device_handle)?.set_flags(flags)?;
        Ok(format!("Flags set to {flags} for {device_handle}"))
    }

    async fn write_bitstream_direct(
        &self,
        platform_string: &str,
        device_handle: &str,
        bitstream_path_str: &str,
        firmware_lookup_path: &str,
    ) -> Result<String, fdo::Error> {
        trace!(
            "load_firmware called with name: {device_handle} and path_str: {bitstream_path_str}"
        );
        validate_device_handle(device_handle)?;
        let path = Path::new(bitstream_path_str);
        if !path.exists() || path.is_dir() {
            return Err(fdo::Error::InvalidArgs(format!(
                "{bitstream_path_str} is not a valid path to a bitstream file."
            )));
        }
        let platform = platform_from_compat_or_device(platform_string, device_handle)?;
        let (prefix, suffix) = make_firmware_pair(path, Path::new(firmware_lookup_path))?;

        let _guard = get_write_lock_guard();
        trace!("Got write lock.");
        write_firmware_source_dir(&prefix.to_string_lossy())?;
        platform.fpga(device_handle)?.load_firmware(&suffix)?;
        Ok(format!(
            "{bitstream_path_str} loaded to {device_handle} using firmware lookup path: '\
         {firmware_lookup_path}'"
        ))
    }

    async fn apply_overlay(
        &self,
        platform_compat_str: &str,
        overlay_handle: &str,
        overlay_source_path: &str,
        firmware_lookup_path: &str,
    ) -> Result<String, fdo::Error> {
        trace!(
            "apply_overlay called with platform_compat_str: {platform_compat_str}, overlay_handle: \
            {overlay_handle} and overlay_path: {overlay_source_path}",
        );
        let platform = platform_for_known_platform(platform_compat_str)?;
        let overlay_handler = platform.overlay_handler(overlay_handle)?;
        let overlay_fs_path = overlay_handler.overlay_fs_path()?;
        let (prefix, suffix) = make_firmware_pair(
            Path::new(overlay_source_path),
            Path::new(firmware_lookup_path),
        )?;

        let _guard = get_write_lock_guard();
        trace!("Got write lock.");
        write_firmware_source_dir(&prefix.to_string_lossy())?;
        overlay_handler.apply_overlay(&suffix)?;
        Ok(format!(
            "{overlay_source_path} loaded via {overlay_fs_path:?} using firmware lookup path: '\
         {firmware_lookup_path}'"
        ))
    }

    async fn remove_overlay(
        &self,
        platform_compat_str: &str,
        overlay_handle: &str,
    ) -> Result<String, fdo::Error> {
        trace!(
            "remove_overlay called with platform_compat_str: {platform_compat_str} and overlay_handle:\
             {overlay_handle}"
        );
        let platform = platform_for_known_platform(platform_compat_str)?;
        let overlay_handler = platform.overlay_handler(overlay_handle)?;
        let overlay_fs_path = overlay_handler.overlay_fs_path()?;
        overlay_handler.remove_overlay()?;
        Ok(format!(
            "{overlay_handle} removed by deleting {overlay_fs_path:?}"
        ))
    }

    /// use to write to a device property from /sys/class/fpga_manager/<device>/** that does not have a specific interface
    async fn write_property(&self, property_path_str: &str, data: &str) -> Result<(), fdo::Error> {
        trace!(
            "write_property called with property_path_str: {property_path_str} and data: {data}"
        );
        let property_path = Path::new(property_path_str);
        if !property_path.starts_with(Path::new(FPGA_MANAGERS_DIR)) {
            return Err(fdo::Error::from(FpgadError::Argument(format!(
                "Cannot access property {property_path_str}: does not begin with {FPGA_MANAGERS_DIR}"
            ))));
        }
        Ok(fs_write(property_path, false, data)?)
    }
}
