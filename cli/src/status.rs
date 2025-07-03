use crate::proxies::status_proxy;
use std::cmp::max;
use std::collections::HashMap;
use tabled::settings::{Style, Width};
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct DeviceRow {
    #[tabled(rename = "Handle")]
    dev: String,
    #[tabled(rename = "Platform")]
    platform: String,
    #[tabled(rename = "State")]
    state: String,
}

#[derive(Tabled)]
struct OverlayRow {
    #[tabled(rename = "Handle")]
    overlay: String,
    #[tabled(rename = "Status")]
    status: String,
}

use zbus::Connection;

/// Sends the dbus command to get a list of overlays and parses it
pub async fn call_get_overlays() -> Result<Vec<String>, zbus::Error> {
    let connection = Connection::system().await?;
    let proxy = status_proxy::StatusProxy::new(&connection).await?;
    let list_str = proxy.get_overlays().await?;
    let ret_list: Vec<String> = list_str.lines().map(|line| line.to_string()).collect();
    Ok(ret_list)
}

/// Sends the dbus command to get the state from an fpga device
pub async fn call_get_fpga_state(device_handle: &str) -> Result<String, zbus::Error> {
    let connection = Connection::system().await?;
    let proxy = status_proxy::StatusProxy::new(&connection).await?;
    proxy.get_fpga_state("", device_handle).await
}

/// Sends the dbus command to get the platform_compat_string for a given device
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
/// device: platform_compat_string
pub async fn call_get_platform_types() -> Result<HashMap<String, String>, zbus::Error> {
    let connection = Connection::system().await?;
    let proxy = status_proxy::StatusProxy::new(&connection).await?;
    let ret_str = proxy.get_platform_types().await?;
    let ret_map = ret_str
        .lines() // split by '\n'
        .filter_map(|line| {
            let mut parts = line.splitn(2, ':');
            match (parts.next(), parts.next()) {
                (Some(key), Some(value)) => Some((key.to_string(), value.to_string())),
                _ => None, // ignore lines without a colon
            }
        })
        .collect();
    Ok(ret_map)
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

fn section_header(label: &str, width: usize) -> String {
    let padding = width.saturating_sub(label.len() + 2);
    let left = padding / 2;
    let right = padding - left;
    format!(
        "{: <left$} {} {: <right$}",
        "",
        label,
        "",
        left = left,
        right = right
    )
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
    let row = vec![DeviceRow {
        dev: device_handle.to_string(),
        platform,
        state,
    }];

    let mut device_table = Table::new(row);
    device_table.with(Style::modern());
    let dev_header = section_header("DEVICES", device_table.total_width());
    Ok(format!("{dev_header}\n{device_table}"))
}

/// get all fpga states, gets all overlay statuses, returns an ascii table as String
async fn get_full_status_message() -> Result<String, zbus::Error> {
    let mut device_rows = Vec::<DeviceRow>::new();
    for (dev, platform) in call_get_platform_types().await? {
        let state = call_get_fpga_state(&dev).await?;
        device_rows.push(DeviceRow {
            dev,
            platform,
            state,
        });
    }

    let mut overlay_rows = Vec::<OverlayRow>::new();
    for overlay in call_get_overlays().await? {
        // TODO: overlays do not provide enough information to work out what platform to use.
        //  so maybe the status command can take a platform type instead or something.
        //  This is tricky.
        let p = get_first_platform().await?;
        let status = call_get_overlay_status(&p, &overlay).await?;
        overlay_rows.push(OverlayRow { overlay, status });
    }
    let mut device_table = Table::new(device_rows);
    let mut overlay_table = Table::new(overlay_rows);

    let target_width = max(device_table.total_width(), overlay_table.total_width());
    device_table
        .with(Style::modern())
        .with(Width::increase(target_width));
    overlay_table
        .with(Style::modern())
        .with(Width::increase(target_width));
    let dev_header = section_header("DEVICES", target_width);
    let overlay_header = section_header("OVERLAYS", target_width);
    Ok(format!(
        "{dev_header}\n{device_table}\n\n{overlay_header}\n{overlay_table}"
    ))
}

/// Argument parser for the status command
pub async fn status_handler(device_handle: &Option<String>) -> Result<String, zbus::Error> {
    let ret_string = match device_handle {
        None => get_full_status_message().await?,
        Some(dev) => get_fpga_state_message(dev.as_str()).await?,
    };
    Ok(ret_string)
}
