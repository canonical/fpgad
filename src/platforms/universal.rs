use crate::{error::Error, system_io::fs_read};
use log::trace;

use super::platform::{Fpga, OverlayHandler, Platform};

#[derive(Debug)]
pub struct UniversalPlatform {
    name: &'static str,
    f_fpga: Option<UniversalFPGA>,
}

impl UniversalPlatform {
    /// Creates a new [`UniversalPlatform`].
    pub(crate) fn new() -> Self {
        trace!("creating new UniversalPlatform");
        UniversalPlatform {
            name: "Universal",
            f_fpga: None,
        }
    }
}

impl Platform for UniversalPlatform {
    /// Returns the `name` of the [`UniversalPlatform`]
    fn name(&self) -> &str {
        self.name
    }

    fn fpga(&mut self, name: &str) -> &impl Fpga {
        assert!(
            !name.is_empty() && name.is_ascii(),
            "fpga name must be compliant with sysfs rules."
        );

        // Create FPGA if not same or present
        if self.f_fpga.as_ref().is_none_or(|f| f.name != name) {
            self.f_fpga = Some(UniversalFPGA::new(name.to_string()));
        }
        self.f_fpga.as_ref().unwrap()
    }

    fn overlay_handler(&self) -> &impl OverlayHandler {
        self
    }
}

impl OverlayHandler for UniversalPlatform {
    fn apply_devicetree(&self) -> bool {
        todo!()
    }

    fn unapply_devicetree(&self) -> bool {
        todo!()
    }
}

#[derive(Debug)]
pub struct UniversalFPGA {
    name: String,
}

impl UniversalFPGA {
    pub(crate) fn new(name: String) -> Self {
        UniversalFPGA { name }
    }
}

impl Fpga for UniversalFPGA {
    fn name(&self) -> &str {
        &self.name
    }

    fn state(&self) -> Result<String, Error> {
        trace!("reading /sys/class/fpga_manager/{}/state", self.name);
        Ok(fs_read(&format!(
            "/sys/class/fpga_manager/{}/state",
            self.name
        ))?)
    }

    fn load_bitstream(&self) -> bool {
        todo!()
    }

    fn unload_bitstream(&self) -> bool {
        todo!()
    }
}
