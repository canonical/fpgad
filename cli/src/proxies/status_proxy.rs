use zbus::{Result, proxy};
#[proxy(
    default_service = "com.canonical.fpgad",
    interface = "com.canonical.fpgad.status",
    default_path = "/com/canonical/fpgad/status"
)]
pub trait Status {
    async fn get_fpga_state(&self, platform_string: &str, device_handle: &str) -> Result<String>;
    async fn get_fpga_flags(&self, platform_string: &str, device_handle: &str) -> Result<String>;
    async fn get_overlay_status(
        &self,
        platform_compat_str: &str,
        overlay_handle: &str,
    ) -> Result<String>;
    async fn get_overlays(&self) -> Result<String>;
    async fn get_platform_type(&self, device_handle: &str) -> Result<String>;
    async fn get_platform_types(&self) -> Result<String>;
    async fn get_platform_name(&self, _device_handle: &str) -> Result<String>;
}
