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

//! DBus proxy interfaces for the fpgad daemon.
//!
//! This module provides auto-generated DBus proxy traits that allow the CLI to communicate
//! with the fpgad daemon over the system DBus. The proxies are generated using the `zbus`
//! crate's `#[proxy]` macro and provide type-safe, asynchronous access to the daemon's
//! DBus interfaces.
//!
//! # Modules
//!
//! - [`control_proxy`] - Write operations (load bitstreams, apply overlays, set properties)
//! - [`status_proxy`] - Read-only operations (query device state, platform info, overlays)
//!
//! # DBus Service Information
//!
//! - **Service Name**: `com.canonical.fpgad`
//! - **Control Interface**: `com.canonical.fpgad.control` at `/com/canonical/fpgad/control`
//! - **Status Interface**: `com.canonical.fpgad.status` at `/com/canonical/fpgad/status`
//!
//! # Usage
//!
//! These proxies are used internally by the CLI's command handlers ([`load`], [`remove`],
//! [`set`], [`status`]) to communicate with the fpgad daemon. The proxies handle DBus
//! connection management and method call marshalling automatically.
//!
//! For more information on the DBus interfaces, see the
//! [daemon DBus documentation](../../../daemon/comm/dbus/index.html).
//!
//! [`load`]: ../load/index.html
//! [`remove`]: ../remove/index.html
//! [`set`]: ../set/index.html
//! [`status`]: ../status/index.html

pub mod control_proxy;
pub mod status_proxy;
