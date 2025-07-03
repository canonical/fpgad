use crate::RemoveSubcommand;
use crate::proxies::control_proxy;
use crate::status::{call_get_platform_type, get_first_overlay, get_first_platform};
use zbus::Connection;

async fn remove_bitstream() -> Result<String, zbus::Error> {
    // TODO: so this is confusing because we don't have a way to remove a bitstream but with
    //  softeners we might end up with this functionality.
    Err(zbus::Error::Failure("Not implemented".to_string()))
}

/// Sends the dbus command to remove an overlay
async fn call_remove_overlay(
    device_handle: &str,
    overlay_handle: &str,
) -> Result<String, zbus::Error> {
    let connection = Connection::system().await?;
    let proxy = control_proxy::ControlProxy::new(&connection).await?;
    proxy.remove_overlay(device_handle, overlay_handle).await
}

/// Populates the platform and overlay handle appropriately before calling `call_apply_overlay`
async fn remove_overlay(
    device_handle: &Option<String>,
    overlay_handle: &Option<String>,
) -> Result<String, zbus::Error> {
    let dev = match device_handle {
        None => get_first_platform().await?,
        Some(dev) => call_get_platform_type(dev).await?,
    };
    let handle = match overlay_handle {
        Some(handle) => handle.clone(),
        None => get_first_overlay().await?,
    };
    call_remove_overlay(&dev, &handle).await
}

/// Argument parser for the remove command
pub async fn remove_handler(
    dev_handle: &Option<String>,
    sub_command: &RemoveSubcommand,
) -> Result<String, zbus::Error> {
    match sub_command {
        RemoveSubcommand::Overlay { handle } => remove_overlay(dev_handle, handle).await,
        RemoveSubcommand::Bitstream => remove_bitstream().await,
    }
}
