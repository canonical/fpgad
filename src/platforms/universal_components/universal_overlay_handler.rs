use crate::error::FpgadError;
use crate::error::FpgadError::ArgumentError;
use crate::platforms::platform::OverlayHandler;
use crate::system_io::{fs_create_dir, fs_read, fs_remove_dir, fs_write};
use log::trace;
use std::path::{Path, PathBuf};

/// Store the steps that need to be undone on delete/failure.
#[derive(Debug)]
struct OverlayRollbackSteps {
    delete_configfs_dir: bool,
}
/// Stores the three relevant paths: source files for dtbo/bitstream and the overlayfs dir to which
/// the dtbo path was written.
/// Also stores the steps that need to be undone
#[derive(Debug)]
pub struct UniversalOverlayHandler {
    overlay_source_path: PathBuf,
    overlay_fs_path: PathBuf,
    rollback_steps: OverlayRollbackSteps,
}

impl Drop for UniversalOverlayHandler {
    /// Upon deletion of the overlay handler, it should undo any applications .e.g on failure.
    fn drop(&mut self) {
        trace!("Dropping UniversalOverlayHandler!");
        // check for necessary is inside the function.
        if self.remove_overlay().is_err() {
            eprintln!("Failed to remove overlay")
        }
    }
}

impl OverlayHandler for UniversalOverlayHandler {
    /// Checks inputs are real files (doesn't yet check they are valid)
    /// Checks for `overlay_fs_path`.
    /// In future this may change the firmware location through
    /// `/sys/module/firmware_class/parameters/`.
    fn prepare_for_load(&mut self) -> Result<(), FpgadError> {
        if !self.overlay_source_path.exists() | self.overlay_source_path.is_dir() {
            return Err(ArgumentError(format!(
                "Overlay file '{:?}' has invalid path.",
                self.overlay_source_path
            )));
        }
        
        if self.overlay_fs_path.exists() {
            fs_remove_dir(&self.overlay_fs_path)?
        }

        trace!("Checking configfs path exists at {:?}", self.overlay_fs_path);
        if let Some(parent_path) = self.overlay_fs_path.parent() {
            if !parent_path.exists() {
                return Err(FpgadError::IO(format!(
                    "The path {:?} doesn't seem to exist.",
                    parent_path
                )));
            }
        } else {
            return Err(FpgadError::IO(format!(
                "The path {:?} has no parent directory.",
                self.overlay_fs_path
            )));
        }

        trace!("Attempting to create '{:?}'", self.overlay_fs_path);
        fs_create_dir(&self.overlay_fs_path)?;
        trace!("Created dir {:?}", self.overlay_fs_path);

        self.rollback_steps.delete_configfs_dir = true;

        Ok(())
    }

    /// Attempts to apply a device tree overlay which should trigger a firmware load.
    /// There are multiple ways to trigger a firmware load so this is not valid if the
    /// dtbo doesn't contain a firmware to load.
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
                    "'{}' successfully written to {:?}",
                    dtbo_file_name, overlay_path_file
                );
            }
            Err(e) => {
                return Err(FpgadError::IO(format!(
                    "Failed to write overlay path '{}' to '{:?}' : '{}'",
                    dtbo_file_name, overlay_path_file, e
                )));
            }
        }
        match self.status() {
            Ok(..) => Ok(()),
            Err(e) => Err(e),
        }
    }

    /// Attempts to delete overlay_fs_path
    fn remove_overlay(&mut self) -> Result<(), FpgadError> {
        if self.rollback_steps.delete_configfs_dir {
            let removed = fs_remove_dir(&self.overlay_fs_path);
            self.rollback_steps.delete_configfs_dir = false;
            removed?
        }
        Ok(())
    }

    /// WARNING NOT IMPLEMENTED:
    /// This is where the required fpga flags will be determined from the dtbo,
    /// such as compressed or encrypted.
    fn get_required_flags(&self) -> Result<isize, FpgadError> {
        Ok(0)
    }

    /// Read status from <overlay_fs_path>/status file and verify that it is "applied"
    fn status(&self) -> Result<String, FpgadError> {
        let status_path = self.overlay_fs_path.join("status");

        trace!("Reading from {:?}", status_path);
        let status = fs_read(&status_path);
        match status {
            Ok(val) => match val.as_str() {
                "applied\n" => Ok(val),
                _ => Err(FpgadError::OverlayStatusError(val)),
            },
            Err(e) => Err(e),
        }
    }
}

impl UniversalOverlayHandler {
    /// Scans the package dir for required files
    pub(crate) fn new(overlay_source_path: &Path) -> Self {
        let overlay_fs = Path::new("/sys/kernel/config/device-tree/overlays/fpgad_overlay_0");

        UniversalOverlayHandler {
            overlay_source_path: overlay_source_path.to_owned(),
            overlay_fs_path: overlay_fs.to_owned(),
            rollback_steps: OverlayRollbackSteps {
                delete_configfs_dir: false,
            },
        }
    }
}
