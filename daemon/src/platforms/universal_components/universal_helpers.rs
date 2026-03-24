// This file is part of fpgad, an application to manage FPGA subsystem together with device-tree and kernel modules.
//
// Copyright 2026 Canonical Ltd.
//
// SPDX-License-Identifier: GPL-3.0-only
//
// fpgad is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License version 3, as published by the Free Software Foundation.
//
// fpgad is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranties of MERCHANTABILITY, SATISFACTORY QUALITY, or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License along with this program.  If not, see http://www.gnu.org/licenses/.

use crate::config;
use crate::error::FpgadError;
use crate::system_io::{fs_read, fs_write};
use log::trace;
use std::path::{Component, Path, PathBuf};

/// Write a specified path to the systems firmware search path.
/// See [these kernel docs](https://docs.kernel.org/driver-api/firmware/fw_search_path.html)
/// for more information on the process.
///
/// # Arguments
///
/// * `new_path`: path inside which firmware can be found
///
/// # Returns: `Result<(), FpgadError>`
/// * `()` on success
/// * `FpgadError::IOWrite` (or similar IO error) if writing fails for any reason.
///
/// # Examples
///
/// ```rust,no_run
/// assert!(write_firmware_source_dir("/lib/firmware/my_firmware_dir").is_ok());
/// ```
pub fn write_firmware_source_dir(new_path: &str) -> Result<(), FpgadError> {
    trace!(
        "Writing fw prefix {} to {}",
        new_path,
        config::FIRMWARE_LOC_CONTROL_PATH
    );
    let fw_lookup_override = Path::new(config::FIRMWARE_LOC_CONTROL_PATH);
    fs_write(fw_lookup_override, false, new_path)
}

/// Helper function to find the overlap between two paths and to return a tuple of the overlap and
/// the difference. Note: the paths must both share the same root otherwise no overlap will be found
///
/// Typically, this is used to create a fw_lookup_path and a corresponding relative path which points
/// to the source file
///
/// # Arguments
///
/// * `source_path`: the full path to the target file (or containing directory?)
/// * `firmware_path`: the root common path for all files to be loaded by the FW subsystem
///
/// # Returns: `Result<(PathBuf, PathBuf), FpgadError>`
/// * `(PathBuf, PathBuf)` - A tuple of (prefix, suffix) where prefix is
///   typically used as the fw_lookup_path and the suffix is remaining relative path from that prefix
/// * `FpgadError::Argument` in case `firmware_path` is not within `source_path`, or for inputs
///   resulting in an empty suffix value
/// # Examples
///
/// ```rust
/// # use std::path::Path;
/// let (prefix, suffix) = make_firmware_pair(
///      Path::new("/lib/firmware/file.bin"),
///      Path::new("/lib/firmware/"),
/// )?;
/// assert_eq!(prefix.to_string_lossy(), "/lib/firmware");
/// assert_eq!(suffix.to_string_lossy(), "file.bin");
/// ```
pub(crate) fn make_firmware_pair(
    source_path: &Path,
    firmware_path: &Path,
) -> Result<(PathBuf, PathBuf), FpgadError> {
    // No firmware search path provided, so just try to use parent dir
    if firmware_path.as_os_str().is_empty() {
        return extract_path_and_filename(source_path);
    }
    if let Ok(suffix) = source_path.strip_prefix(firmware_path) {
        // Remove leading '/' if present
        let cleaned_suffix_path = suffix
            .components()
            .skip_while(|c| matches!(c, Component::RootDir))
            .collect::<PathBuf>();
        if cleaned_suffix_path.as_os_str().is_empty() {
            return Err(FpgadError::Argument(format!(
                "The resulting filename from stripping {firmware_path:?} from {source_path:?} \
                was empty. Cannot write empty string to fpga."
            )));
        }
        Ok((firmware_path.to_path_buf(), cleaned_suffix_path))
    } else {
        Err(FpgadError::Argument(format!(
            "Could not find {source_path:?} inside {firmware_path:?}"
        )))
    }
}

/// Splits a Path object into its parent directory and basename/filename
///
/// # Arguments
///
/// * `path`: path to be split
///
/// # Returns: `Result<(PathBuf, PathBuf), FpgadError>`
/// * `(PathBuf, PathBuf)` - Tuple of parent directory and basename/filename
/// * `FpgadError::Argument` on invalid `path` or `path` is a root directory (no parent)
/// # Examples
///
/// ```rust,no_run
/// let (parent, base) = extract_path_and_filename(Path::new("/lib/firmware/file.bin"));
/// assert_eq!(parent.to_string_lossy(), "/lib/firmware");
/// assert_eq!(base.to_string_lossy(), "file.bin");
/// ```
pub fn extract_path_and_filename(path: &Path) -> Result<(PathBuf, PathBuf), FpgadError> {
    // Extract filename
    let filename = path
        .file_name()
        .and_then(|f| f.to_str())
        .ok_or(FpgadError::Argument(format!(
            "Provided bitstream path {path:?} is not a file or a valid directory."
        )))?;

    // Extract parent directory
    let base_path = path
        .parent()
        .and_then(|p| p.to_str())
        .ok_or(FpgadError::Argument(format!(
            "Provided bitstream path {path:?} is missing a parent dir."
        )))?;

    Ok((base_path.into(), filename.into()))
}

#[allow(dead_code)]
/// Read the currently specified firmware search path.
/// See [these kernel docs](https://docs.kernel.org/driver-api/firmware/fw_search_path.html)
/// for more information on the process.
///
/// # Returns: `Result<String, FpgadError>`
/// * `String` - The contents of the firmware search path variable.
/// * `FpgadError::IOWrite` (or similar IO error) if writing fails for any reason.
///
/// # Examples
///
/// ```rust,no_run
/// let search_path_str = read_firmware_source_dir()?;
/// assert_eq!(search_path_str, "/lib/firmware/my_firmware_dir");
/// ```
pub fn read_firmware_source_dir() -> Result<String, FpgadError> {
    trace!(
        "Reading fw prefix from {}",
        config::FIRMWARE_LOC_CONTROL_PATH
    );
    let fw_lookup_override = Path::new(config::FIRMWARE_LOC_CONTROL_PATH);
    fs_read(fw_lookup_override)
}

#[cfg(test)]
mod test_make_firmware_pair {
    use crate::error::FpgadError;
    use crate::platforms::universal_components::universal_helpers::make_firmware_pair;
    use googletest::prelude::*;
    use rstest::*;
    use std::path::PathBuf;

    #[gtest]
    #[rstest]
    #[case::all_good(
        "/lib/firmware/file.bin",
        "/lib/firmware/",
        "/lib/firmware/",
        "file.bin"
    )]
    #[case::no_fw_path("/lib/firmware/file.bin", "", "/lib/firmware/", "file.bin")]
    #[case::no_fw_path_no_file("/lib/firmware/", "", "/lib/", "firmware")]
    fn should_pass(
        #[case] source: &str,
        #[case] fw_path: &str,
        #[case] exp_prefix: &str,
        #[case] exp_suffix: &str,
    ) {
        let result = make_firmware_pair(&PathBuf::from(source), &PathBuf::from(fw_path));
        assert_that!(
            result,
            ok(eq(&(PathBuf::from(exp_prefix), PathBuf::from(exp_suffix))))
        );
    }

    #[gtest]
    #[rstest]
    #[case::no_file(
        "/lib/firmware/",
        "/lib/firmware/",
        err(displays_as(contains_substring("The resulting filename from stripping")))
    )]
    #[case::not_in_dir(
        "/lib/firmware/file.bin",
        "/snap/x1/data/file.bin",
        err(displays_as(contains_substring("Could not find")))
    )]
    fn should_fail<M: for<'a> Matcher<&'a std::result::Result<(PathBuf, PathBuf), FpgadError>>>(
        #[case] source: &str,
        #[case] fw_path: &str,
        #[case] condition: M,
    ) {
        let result = make_firmware_pair(&PathBuf::from(source), &PathBuf::from(fw_path));
        assert_that!(&result, condition);
    }
}
