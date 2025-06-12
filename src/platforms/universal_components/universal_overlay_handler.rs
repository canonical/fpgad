use std::path::{Path, PathBuf};
use log::trace;
use crate::error::FpgadError;
use crate::error::FpgadError::ArgumentError;
use crate::platforms::platform::OverlayHandler;
use crate::system_io::{fs_create_dir, fs_read, fs_remove_dir, fs_write};

#[derive(Debug)]
struct OverlayRollbackSteps {
    delete_overlay_in_fw: bool,
    reset_firmware_path: bool,
}
#[derive(Debug)]
pub struct UniversalOverlayHandler {
    // we have the source directory, the overlayfs directory, and the name of the relevant files
    bitstream_source_path: PathBuf,
    overlay_source_path: PathBuf,
    overlay_fs_path: PathBuf,
    rollback_steps: OverlayRollbackSteps,
}


impl Drop for UniversalOverlayHandler {
    fn drop(&mut self) {
        if self.rollback_steps.delete_overlay_in_fw {
            self.remove_overlay().expect("Failed to remove overlay");
        }
        if self.rollback_steps.reset_firmware_path {
            println!("NOT IMPLEMENTED: UniversalOverlayHandler::reset_firmware_path()");
        }

        println!("Dropping MyStruct!");
        // TODO: go through steps which are true in self.rollback_steps.
    }
}

impl OverlayHandler for UniversalOverlayHandler {
    /// `/sys/module/firmware_class/parameters/path`
    fn prepare_for_load(&mut self) -> Result<(), FpgadError> {
        if !self.overlay_source_path.exists() | self.overlay_fs_path.is_dir() {
            return Err(ArgumentError(format!(
                "Overlay file '{:?}' has invalid path.",
                self.overlay_source_path
            )));
        }

        if !self.bitstream_source_path.exists() | self.overlay_fs_path.is_dir() {
            return Err(ArgumentError(format!(
                "Bitstream file '{:?}' has invalid path.",
                self.overlay_fs_path
            )));
        }

        if self.overlay_fs_path.exists() {
            fs_remove_dir(&self.overlay_fs_path)?
        }

        trace!("Checking configfs path exists.");
        if !self.overlay_fs_path.parent().unwrap().exists() {
            eprintln!("NOOOOO! no path {:?}", self.overlay_fs_path.parent())
        }

        trace!("Attempting to create '{:?}'", self.overlay_fs_path);
        fs_create_dir(&self.overlay_fs_path)?;
        trace!("Created dir {:?}", self.overlay_fs_path);

        self.rollback_steps.delete_overlay_in_fw = true;

        Ok(())
    }

    fn apply_overlay(&self) -> Result<(), FpgadError> {
        let dtbo_file_name = self
            .overlay_source_path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();
        let overlay_path_file = self.overlay_fs_path.join("path");
        match fs_write(&overlay_path_file, false, dtbo_file_name) {
            Ok(_) => {
                trace!(
                    "{} succesfully written to {:?}",
                    dtbo_file_name, overlay_path_file
                );
            }
            Err(e) => {
                return Err(FpgadError::IO(format!(
                    "Failed to write overlay path {} to {:?} : {}",
                    dtbo_file_name, overlay_path_file, e
                )));
            }
        }
        match self.status() {
            Ok(..) => Ok(()),
            Err(e) => Err(e),
        }
    }
    fn remove_overlay(&self) -> Result<(), FpgadError> {
        Ok(fs_remove_dir(&self.overlay_fs_path)?)
    }

    fn get_required_flags(&self) -> Result<isize, FpgadError> {
        Ok(0)
    }

    fn status(&self) -> Result<String, FpgadError> {
        let status_path = self.overlay_fs_path.join("status");

        trace!("Reading from {:?}", status_path);
        let state = match fs_read(&status_path) {
            Ok(val) => Ok(val),
            Err(e) => Err(e),
        };
        match state {
            Ok(val) => match val.as_str() {
                "applied\n" => Ok(val),
                _ => Err(FpgadError::OverlayStatusError(val)),
            },
            Err(e) => Err(e),
        }
    }
}


impl UniversalOverlayHandler {
    /// Scans the package dir for reqiured files
    pub(crate) fn new(bitstream_path: &Path, overlay_source_path: &Path) -> Self {
        let overlay_fs = Path::new("/sys/kernel/config/device-tree/overlays/fpgad_overlay_0");

        UniversalOverlayHandler {
            bitstream_source_path: bitstream_path.to_owned(),
            overlay_source_path: overlay_source_path.to_owned(),
            overlay_fs_path: overlay_fs.to_owned(),
            rollback_steps: OverlayRollbackSteps {
                delete_overlay_in_fw: false,
                reset_firmware_path: false,
            },
        }
    }
}

