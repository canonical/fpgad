use std::sync::{Mutex, OnceLock};

// TODO: this should be provided by the config file
pub(crate) static CONFIG_FS_PREFIX: &str = "/sys/kernel/config/device-tree/overlays/";
pub(crate) static FW_PREFIX: &str = "/lib/firmware/";
pub(crate) static SYSFS_PREFIX: &str = "/sys/class/fpga_manager/";

pub struct SystemConfig {
    firmware_prefix: Mutex<String>,
    sys_fs_prefix: Mutex<String>,
    config_fs_prefix: Mutex<String>,
}

impl SystemConfig {
    fn default() -> SystemConfig {
        SystemConfig {
            config_fs_prefix: Mutex::new(CONFIG_FS_PREFIX.to_string()),
            firmware_prefix: Mutex::new(FW_PREFIX.to_string()),
            sys_fs_prefix: Mutex::new(SYSFS_PREFIX.to_string()),
        }
    }
}

static CONFIG: OnceLock<SystemConfig> = OnceLock::new();

impl SystemConfig {
    #[allow(dead_code)]
    pub fn set_config_fs_prefix(&mut self, prefix: String) {
        *self.config_fs_prefix.lock().unwrap() = prefix;
    }
    #[allow(dead_code)]
    pub fn set_firmware_prefix(&mut self, prefix: String) {
        *self.firmware_prefix.lock().unwrap() = prefix;
    }
    #[allow(dead_code)]
    pub fn set_sysfs_prefix(&mut self, prefix: String) {
        *self.sys_fs_prefix.lock().unwrap() = prefix;
    }

    pub fn config_fs_prefix(&self) -> String {
        self.config_fs_prefix.lock().unwrap().clone()
    }

    pub fn firmware_prefix(&self) -> String {
        self.firmware_prefix.lock().unwrap().clone()
    }

    pub fn sys_fs_prefix(&self) -> String {
        self.sys_fs_prefix.lock().unwrap().clone()
    }
}

fn init_system_config() -> SystemConfig {
    // TODO: read from file will happen here.
    SystemConfig::default()
}

pub fn system_config() -> &'static SystemConfig {
    CONFIG.get_or_init(init_system_config)
}
