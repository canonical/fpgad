use zbus::{Result, proxy};
#[proxy(
    default_service = "com.canonical.fpgad",
    interface = "com.canonical.fpgad.control",
    default_path = "/com/canonical/fpgad/control"
)]
pub trait Control {
    async fn load_defaults(&self) -> Result<String>;
    async fn set_fpga_flags(&self, device_handle: &str, flags: u32) -> Result<String>;
    async fn write_bitstream_direct(
        &self,
        device_handle: &str,
        bitstream_path_str: &str,
    ) -> Result<String>;

    async fn apply_overlay(
        &self,
        platform_str: &str,
        overlay_handle: &str,
        overlay_source_path: &str,
    ) -> Result<String>;

    async fn remove_overlay(&self, platform_str: &str, overlay_handle: &str) -> Result<String>;
}
