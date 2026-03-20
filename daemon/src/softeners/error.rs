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

//! Error types for FPGA softener implementations.
//!
//! This module defines error types specific to vendor-specific softener implementations
//! (like Xilinx DFX Manager). These errors are converted to the main [`FpgadError`] type
//! for propagation through the daemon's DBus interfaces.
//!
//! # Error Types
//!
//! - [`FpgadSoftenerError`] - Errors from vendor-specific softener operations
//!
//! # Error Handling
//!
//! Softener errors implement conversion traits to both `fdo::Error` (for DBus responses)
//! and `FpgadError` (for internal error handling). All errors are logged when converted.

/// Errors specific to FPGA softener implementations.
///
/// This enum represents errors that occur in vendor-specific softener code,
/// such as failures in communication with external tools or daemons (e.g., dfx-mgr-client).
#[derive(Debug, thiserror::Error)]
pub enum FpgadSoftenerError {
    /// Error from Xilinx DFX Manager operations.
    ///
    /// This variant represents failures when calling dfx-mgr-client or when
    /// dfx-mgrd is not available or returns an error. The string contains
    /// detailed error information from the dfx-mgr tooling.
    #[error("FpgadSoftenerError::DfxMgr: {0}")]
    DfxMgr(std::io::Error),
}
