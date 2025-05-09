use crate::error::Error;

pub fn list_fpga_managers() -> Vec<String> {
    std::fs::read_dir("/sys/class/fpga_manager")
        .map(|iter| {
            iter.filter_map(Result::ok)
                .map(|entry| entry.file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default()
}

/// A sysfs map of an fpga in fpga_manager class.
/// See the example below (not all sysfs files are implemented as methods):
/// ubuntu@kria:~$ tree /sys/class/fpga_manager/fpga0
/// /sys/class/fpga_manager/fpga0
/// ├── device -> ../../../firmware:zynqmp-firmware:pcap
/// ├── firmware
/// ├── flags
/// ├── key
/// ├── name
/// ├── of_node -> ../../../../../../firmware/devicetree/base/firmware/zynqmp-firmware/pcap
/// ├── power
/// │   ├── async
/// │   ├── autosuspend_delay_ms
/// │   ├── control
/// │   ├── runtime_active_kids
/// │   ├── runtime_active_time
/// │   ├── runtime_enabled
/// │   ├── runtime_status
/// │   ├── runtime_suspended_time
/// │   └── runtime_usage
/// ├── state
/// ├── status
/// ├── subsystem -> ../../../../../../class/fpga_manager
/// └── uevent
///
pub trait Fpga {
    fn name(&self) -> &str;
    fn state(&self) -> Result<String, Error>;
    fn load_bitstream(&self) -> bool;
    fn unload_bitstream(&self) -> bool;
}

pub trait OverlayHandler {
    fn apply_devicetree(&self) -> bool;
    fn unapply_devicetree(&self) -> bool;
}

pub trait Platform {
    fn name(&self) -> &str;
    fn fpga(&mut self, name: &str) -> &impl Fpga;
    fn overlay_handler(&self) -> &impl OverlayHandler;
}
