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

use fpgad_api::status::{
    call_get_fpga_state, call_get_overlay_status, call_get_overlays, call_get_platform_type,
    call_get_platform_types,
};

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
    let first = overlays.first().ok_or(zbus::Error::Failure(
        "Could not find an overlay to remove".to_string(),
    ))?;
    Ok(first.clone())
}

/// gets the first platform in the container from `call_get_platform_types`
pub async fn get_first_device_handle() -> Result<String, zbus::Error> {
    let platform = match call_get_platform_types().await?.keys().next() {
        Some(p) => p.clone(),
        None => return Err(zbus::Error::Failure("Got no platforms.".to_string())),
    };

    Ok(platform)
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
    ret_string += "\n---- OVERLAYS ----\n\
                   | overlay | status |\n";
    for overlay in call_get_overlays().await? {
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
