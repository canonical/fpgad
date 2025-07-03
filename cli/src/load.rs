use crate::LoadSubcommand;
use crate::proxies::control_proxy;
use crate::status::{
    call_get_platform_type, call_get_platform_types, get_first_device_handle, get_first_platform,
};
use zbus::Connection;

/// Sends the dbus command to load a bitstream
async fn call_load_bitstream(
    platform_str: &str,
    device_handle: &str,
    file_path: &str,
) -> Result<String, zbus::Error> {
    let connection = Connection::system().await?;
    let proxy = control_proxy::ControlProxy::new(&connection).await?;
    proxy
        .write_bitstream_direct(platform_str, device_handle, file_path)
        .await
}

/// Sends the dbus command to apply an overlay
async fn call_apply_overlay(
    platform: &str,
    file_path: &str,
    overlay_handle: &str,
) -> Result<String, zbus::Error> {
    let connection = Connection::system().await?;
    let proxy = control_proxy::ControlProxy::new(&connection).await?;
    proxy
        .apply_overlay(platform, overlay_handle, file_path)
        .await
}

/// Populates the platform and overlay handle appropriately before calling `call_apply_overlay`
async fn apply_overlay(
    dev_handle: &Option<String>,
    file_path: &str,
    overlay_handle: &Option<String>,
) -> Result<String, zbus::Error> {
    // Determine platform and overlay handle based on provided parameters
    let (platform, overlay_handle_to_use) = match (dev_handle, overlay_handle) {
        // Both are provided
        (Some(dev), Some(overlay)) => (call_get_platform_type(dev).await?, overlay.clone()),

        // dev_handle provided, overlay_handle not provided so use device name as overlay handle
        (Some(dev), None) => {
            let platform = call_get_platform_type(dev).await?;
            (platform, dev.clone())
        }
        // dev_handle not provided, so use first platform
        (None, Some(overlay)) => {
            let platform = get_first_platform().await?;
            (platform, overlay.clone())
        }
        // neither provided so get first device to and use its platform as platform and its name as
        // overlay_handle
        (None, None) => {
            // this saves making two dbus calls by getting it all from the hashmap
            let platforms = call_get_platform_types().await?;
            let platform = platforms
                .values()
                .next()
                .unwrap_or(&"universal".to_string())
                .clone();
            let overlay = platforms
                .keys()
                .next()
                .unwrap_or(&"overlay0".to_string())
                .clone();
            (platform, overlay)
        }
    };

    call_apply_overlay(&platform, file_path, &overlay_handle_to_use).await
}

/// Populates the device_handle appropriately before calling `call_load_bitstream`
async fn load_bitstream(
    device_handle: &Option<String>,
    file_path: &str,
) -> Result<String, zbus::Error> {
    let dev = match device_handle {
        None => get_first_device_handle().await?,
        Some(dev) => dev.clone(),
    };
    call_load_bitstream("", &dev, file_path).await
}

/// Argument parser for the load command
pub async fn load_handler(
    dev_handle: &Option<String>,
    sub_command: &LoadSubcommand,
) -> Result<String, zbus::Error> {
    match sub_command {
        LoadSubcommand::Overlay { file, handle } => {
            apply_overlay(dev_handle, file.as_ref(), handle).await
        }
        LoadSubcommand::Bitstream { file } => load_bitstream(dev_handle, file.as_ref()).await,
    }
}
