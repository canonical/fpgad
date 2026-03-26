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

use crate::proxies::status_proxy;
use std::collections::HashMap;
use zbus::Connection;

fn parse_overlay_lines(list_str: &str) -> Vec<String> {
    list_str.lines().map(|line| line.to_string()).collect()
}

fn parse_platform_types_lines(ret_str: &str) -> HashMap<String, String> {
    ret_str
        .lines()
        .filter_map(|line| {
            let mut parts = line.splitn(2, ':');
            match (parts.next(), parts.next()) {
                (Some(key), Some(value)) => Some((key.to_string(), value.to_string())),
                _ => None,
            }
        })
        .collect()
}

// Isolate empty-overlay error mapping so unit tests can validate exact error
// variant/message without depending on a live D-Bus service.
fn first_overlay_or_error(overlays: &[String]) -> Result<String, zbus::Error> {
    let first = overlays.first().ok_or(zbus::Error::Failure(
        "Could not find an overlay to remove".to_string(),
    ))?;
    Ok(first.clone())
}

// Isolate empty-platform error mapping so unit tests can validate exact error
// variant/message without depending on a live D-Bus service.
fn first_device_handle_or_error(
    platforms: &HashMap<String, String>,
) -> Result<String, zbus::Error> {
    match platforms.keys().next() {
        Some(p) => Ok(p.clone()),
        None => Err(zbus::Error::Failure("Got no platforms.".to_string())),
    }
}

/// Sends the dbus command to get a list of overlays and parses it
pub async fn call_get_overlays() -> Result<Vec<String>, zbus::Error> {
    let connection = Connection::system().await?;
    let proxy = status_proxy::StatusProxy::new(&connection).await?;
    let list_str = proxy.get_overlays().await?;
    Ok(parse_overlay_lines(&list_str))
}

/// Sends the dbus command to get the state from an fpga device
pub async fn call_get_fpga_state(device_handle: &str) -> Result<String, zbus::Error> {
    let connection = Connection::system().await?;
    let proxy = status_proxy::StatusProxy::new(&connection).await?;
    proxy.get_fpga_state("", device_handle).await
}

/// Sends the dbus command to get the platform_string for a given device
pub async fn call_get_platform_type(device_handle: &str) -> Result<String, zbus::Error> {
    let connection = Connection::system().await?;
    let proxy = status_proxy::StatusProxy::new(&connection).await?;
    proxy.get_platform_type(device_handle).await
}

/// Sends the dbus command to get the status string for a given overlay
async fn call_get_overlay_status(
    platform: &str,
    overlay_handle: &str,
) -> Result<String, zbus::Error> {
    let connection = Connection::system().await?;
    let proxy = status_proxy::StatusProxy::new(&connection).await?;
    proxy.get_overlay_status(platform, overlay_handle).await
}

/// parses the string from `get_platform_types` interface into a HashMap of
/// device:platform_string
pub async fn call_get_platform_types() -> Result<HashMap<String, String>, zbus::Error> {
    let connection = Connection::system().await?;
    let proxy = status_proxy::StatusProxy::new(&connection).await?;
    let ret_str = proxy.get_platform_types().await?;
    Ok(parse_platform_types_lines(&ret_str))
}

/// gets the first platform in the container from `call_get_platform_types`
pub async fn get_first_platform() -> Result<String, zbus::Error> {
    let platforms = call_get_platform_types().await?;
    Ok(platforms
        .values()
        .next()
        .unwrap_or(&"universal".to_string())
        .clone())
}

/// gets the first overlay in the Vec from `call_get_overlays`
pub async fn get_first_overlay() -> Result<String, zbus::Error> {
    let overlays = call_get_overlays().await?;
    first_overlay_or_error(&overlays)
}

/// gets the first platform in the container from `call_get_platform_types`
pub async fn get_first_device_handle() -> Result<String, zbus::Error> {
    let platforms = call_get_platform_types().await?;
    first_device_handle_or_error(&platforms)
}

/// gets one fpga state and returns an ascii table as String
async fn get_fpga_state_message(device_handle: &str) -> Result<String, zbus::Error> {
    let state = call_get_fpga_state(device_handle).await?;
    let platform = call_get_platform_type(device_handle).await?;
    Ok(format!(
        "---- DEVICE  ----\n\
        | dev | platform | state |\n\
        {device_handle} | {platform} | {state}"
    ))
}

/// get all fpga states, gets all overlay statuses, returns a ascii table as String
async fn get_full_status_message() -> Result<String, zbus::Error> {
    let mut ret_string = String::from(
        "---- DEVICES ----\n\
    | dev | platform | state |\n",
    );

    for (dev, platform) in call_get_platform_types().await? {
        let state = call_get_fpga_state(&dev).await?;
        ret_string += format!("| {dev} | {platform} | {state} |\n").as_str();
    }

    // If overlayfs not enabled, or interface not connected this will be an error.
    let overlays = match call_get_overlays().await {
        Ok(overlays) => {
            ret_string += "\n---- OVERLAYS ----\n\
                   | overlay | status |\n";
            overlays
        }
        Err(e) => {
            ret_string += "\n---- OVERLAYS NOT ACCESSIBLE ----\n\n\
            errors encountered:\n";
            ret_string += e.to_string().as_str();
            Vec::new()
        }
    };

    for overlay in overlays {
        // TODO: overlays do not provide enough information to work out what platform to use.
        //  so maybe the status command can take a platform type instead or something.
        //  This is tricky.
        let p = get_first_platform().await?;
        let status = call_get_overlay_status(&p, &overlay).await?;
        ret_string.push_str(format!("| {overlay} | {status} |\n").as_ref());
    }
    Ok(ret_string)
}

/// Argument parser for the status command
pub async fn status_handler(device_handle: &Option<String>) -> Result<String, zbus::Error> {
    let ret_string = match device_handle {
        None => get_full_status_message().await?,
        Some(dev) => get_fpga_state_message(dev.as_str()).await?,
    };
    Ok(ret_string)
}

#[cfg(test)]
mod tests {
    use super::{
        first_device_handle_or_error, first_overlay_or_error, parse_overlay_lines,
        parse_platform_types_lines,
    };
    use std::collections::HashMap;

    #[test]
    fn parse_platform_types_ignores_invalid_lines() {
        let parsed = parse_platform_types_lines("fpga0:xlnx,zynq\ninvalid_line\nfpga1:intel,foo\n");

        assert_eq!(parsed.get("fpga0").map(String::as_str), Some("xlnx,zynq"));
        assert_eq!(parsed.get("fpga1").map(String::as_str), Some("intel,foo"));
        assert!(!parsed.contains_key("invalid_line"));
    }

    #[test]
    fn parse_platform_types_keeps_value_with_extra_colons() {
        let parsed = parse_platform_types_lines("fpga0:xilinx:zynq:mp\n");
        assert_eq!(
            parsed.get("fpga0").map(String::as_str),
            Some("xilinx:zynq:mp")
        );
    }

    #[test]
    fn parse_overlay_lines_keeps_order() {
        let overlays = parse_overlay_lines("overlay0\noverlay1\n");
        assert_eq!(
            overlays,
            vec!["overlay0".to_string(), "overlay1".to_string()]
        );
    }

    #[test]
    fn first_overlay_returns_failure_error_with_expected_message() {
        let overlays = Vec::<String>::new();
        let err = first_overlay_or_error(&overlays).expect_err("empty overlays should fail");

        match &err {
            zbus::Error::Failure(message) => {
                assert_eq!(message, "Could not find an overlay to remove")
            }
            _ => panic!("expected zbus::Error::Failure, got {err}"),
        }

        assert_eq!(err.to_string(), "Could not find an overlay to remove");
    }

    #[test]
    fn first_device_handle_returns_failure_error_with_expected_message() {
        let platforms = HashMap::<String, String>::new();
        let err =
            first_device_handle_or_error(&platforms).expect_err("empty platforms should fail");

        match &err {
            zbus::Error::Failure(message) => assert_eq!(message, "Got no platforms."),
            _ => panic!("expected zbus::Error::Failure, got {err}"),
        }

        assert_eq!(err.to_string(), "Got no platforms.");
    }
}
