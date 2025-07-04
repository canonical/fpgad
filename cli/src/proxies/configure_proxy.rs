use zbus::{Result, proxy};
#[proxy(
    default_service = "com.canonical.fpgad",
    interface = "com.canonical.fpgad.configure",
    default_path = "/com/canonical/fpgad/configure"
)]
pub trait Configure {
    async fn get_overlay_control_dir(&self) -> Result<String>;
    async fn get_firmware_source_dir(&self) -> Result<String>;
    async fn get_fpga_managers_dir(&self) -> Result<String>;
    async fn set_overlay_control_dir(&self, new_path: &str) -> Result<String>;
    async fn set_firmware_source_dir(&self, new_path: &str) -> Result<String>;
    async fn set_fpga_managers_dir(&self, new_path: &str) -> Result<String>;
}
