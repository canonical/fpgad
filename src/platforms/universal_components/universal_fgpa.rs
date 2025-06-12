use crate::error::FpgadError;
use crate::platforms::platform::Fpga;
use crate::system_io::{fs_read, fs_write};
use log::trace;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct UniversalFPGA {
    pub(crate) name: String,
}

impl UniversalFPGA {
    /// Constructor simply stores an owned version of the provided name.
    pub(crate) fn new(name: &str) -> Self {
        UniversalFPGA {
            name: name.to_owned(),
        }
    }
}

impl Fpga for UniversalFPGA {
    /// Get the name of this FPGA device e.g. fpga0.
    fn name(&self) -> &str {
        &self.name
    }

    /// Reads and returns contents of `/sys/class/fpga_manager/self.name/state` or FpgadError::IO.
    ///
    /// returns: Result<String, FpgadError>
    fn state(&self) -> Result<String, FpgadError> {
        trace!("reading /sys/class/fpga_manager/{}/state", self.name);
        let state = fs_read(Path::new(&format!(
            "/sys/class/fpga_manager/{}/state",
            self.name
        )));
        match state {
            Ok(val) => match val.as_str() {
                "operating\n" => Ok(val),
                "unknown\n" => {
                    trace!(
                        "fpga state shows an 'unknown'> Fine if this is first boot, sketchy otherwise."
                    );
                    Ok(val)
                }
                _ => Err(FpgadError::StateError(val)),
            },
            Err(e) => Err(e),
        }
    }

    /// Gets the flags from the hex string stored in the sysfs flags file
    /// e.g. sys/class/fpga_manager/fpga0/flags
    fn get_flags(&self) -> Result<isize, FpgadError> {
        let path = format!("/sys/class/fpga_manager/{}/flags", self.name);
        let contents = fs_read(&PathBuf::from(&path))?;
        let trimmed = contents.trim().trim_start_matches("0x");
        isize::from_str_radix(trimmed, 16)
            .map_err(|_| FpgadError::FlagError("Parsing flags failed".into()))
    }

    /// Sets the flags in the sysfs flags file (e.g. sys/class/fpga_manager/fpga0/flags)
    /// and verifies the the write stuck by reading it back.
    fn set_flags(&self, flags: isize) -> Result<(), FpgadError> {
        trace!(
            "Writing {} to '/sys/class/fpga_manager/{}/flags'",
            flags, self.name
        );
        match fs_write(
            &PathBuf::from(&format!("/sys/class/fpga_manager/{}/flags", self.name)),
            false,
            flags.to_string(),
        ) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Failed to read state.");
                return Err(e);
            }
        };

        self.state()?;

        match self.get_flags() {
            Ok(returned_flags) if returned_flags == flags => Ok(()),
            Ok(returned_flags) => Err(FpgadError::FlagError(format!(
                "Setting flags to {} failed. Returned flag was {}",
                flags, returned_flags
            ))),
            Err(e) => Err(FpgadError::FlagError(format!(
                "Failed to read flags after setting to {}: {}",
                flags, e
            ))),
        }
    }

    /// This can be used to manually load a firmware if the overlay does not trigger the load.
    /// Note: always load firmware before overlay.
    fn load_firmware(&self, bitstream_path: &Path) -> Result<(), FpgadError> {
        fs_write(bitstream_path, false, "/sys/class/fpga_manager/{}/path")?;
        match self.state() {
            Ok(state) => match state.to_string().as_str() {
                "operating\n" => Ok(()),
                "unknown\n" => Err(FpgadError::StateError(format!(
                    "After loading bitstream, fpgastate should be 'operating' but it is '{}'",
                    state
                ))),
                _ => Err(FpgadError::StateError(format!(
                    "After loading bitstream, fpgastate should be 'operating' but it is '{}'",
                    state
                ))),
            },
            Err(e) => Err(e),
        }
    }
}
