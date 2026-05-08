// This file is part of fpgad, an application to manage FPGA subsystem together with device-tree and kernel modules.
//
// Copyright 2026 Canonical Ltd.
//
// SPDX-License-Identifier: GPL-3.0-only
//
// fpgad is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License version 3, as published by the Free Software Foundation.
//
// fpgad is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranties of MERCHANTABILITY, SATISFACTORY QUALITY, or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License along with this program.  If not, see http://www.gnu.org/licenses/.

//! Softener daemon lifecycle management.
//!
//! This module provides infrastructure for managing external daemon processes required
//! by vendor-specific FPGA softener implementations (such as dfx-mgrd for Xilinx devices).
//! It handles daemon discovery, startup, monitoring, and graceful shutdown.
//!
//! # Components
//!
//! * [`DaemonConfig`] - Configuration for individual daemon processes
//! * [`DaemonManager`] - Lifecycle manager that starts, monitors, and stops daemons
//!
//! # Daemon Discovery
//!
//! At startup, the manager checks for daemon binaries at configured paths (typically
//! under `$SNAP/usr/bin/`). Only daemons whose binaries are found will be started.
//! Socket files are monitored to confirm successful startup.
//!
//! # Lifecycle
//!
//! The daemon manager runs until the task is cancelled or the program exits. Status of all
//! daemons is checked every 5 seconds. When a daemon exits unexpectedly, the manager attempts
//! to restart it up to 5 times. If restart attempts fail repeatedly, that daemon is abandoned
//! while others continue running. All managed processes are automatically cleaned up when the
//! manager is dropped.
//!
//! # Logging
//!
//! Each daemon's stdout and stderr are redirected to log files under `$SNAP_COMMON/log/`
//! (or `/tmp/log/` if not in a snap).

use crate::error::FpgadError;
use crate::softeners::error::FpgadSoftenerError;
use log::{error, info, warn};
use std::collections::HashMap;
use std::env;
use std::fs::{self, OpenOptions};
use std::os::unix::fs::FileTypeExt; // for "is_socket"
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::Duration;
use tokio::time::sleep;

/// Configuration for a softener daemon.
///
/// Defines the parameters needed to start and monitor a daemon process, including
/// binary location, socket path for health checking, and error handling.
///
/// # Fields
///
/// * `name` - Name of the daemon
/// * `binary_path` - Path to the daemon binary
/// * `socket_path` - Path to the daemon's socket file (used for startup verification)
/// * `start_timeout` - Timeout in seconds to wait for daemon startup
/// * `log_file` - Optional path to log file (will be auto-generated if None)
/// * `error_constructor` - Function to create the appropriate error type
#[derive(Debug, Clone)]
pub struct DaemonConfig {
    /// Name of the daemon
    pub name: String,
    /// Path to the daemon binary
    pub binary_path: PathBuf,
    /// Path to the daemon's socket file
    pub socket_path: PathBuf,
    /// Timeout in seconds to wait for daemon startup
    pub start_timeout: u64,
    /// Optional path to log file (will be auto-generated if None)
    pub log_file: Option<PathBuf>,
    /// Function to create the appropriate error type
    pub error_constructor: fn(String) -> FpgadSoftenerError,
}

impl DaemonConfig {
    /// Create a new daemon configuration with auto-generated log file path.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the daemon
    /// * `binary_path` - Path to the daemon binary
    /// * `socket_path` - Path to the daemon's socket file
    /// * `start_timeout` - Timeout in seconds to wait for daemon startup
    /// * `error_constructor` - Function to create the appropriate error type
    ///
    /// # Returns: `Self`
    /// * New DaemonConfig instance with auto-generated log file path
    pub fn new(
        name: String,
        binary_path: PathBuf,
        socket_path: PathBuf,
        start_timeout: u64,
        error_constructor: fn(String) -> FpgadSoftenerError,
    ) -> Self {
        Self {
            name,
            binary_path,
            socket_path,
            start_timeout,
            log_file: None,
            error_constructor,
        }
    }

    /// Get the log file path, creating default if not set.
    fn log_file(&self) -> PathBuf {
        if let Some(ref log_file) = self.log_file {
            log_file.clone()
        } else {
            let snap_common = std::env::var("SNAP_COMMON").unwrap_or_else(|_| "/tmp".to_string());
            let log_dir = PathBuf::from(snap_common).join("log");
            // Create log directory if it doesn't exist
            let _ = fs::create_dir_all(&log_dir);
            log_dir.join(format!("{}.log", self.name))
        }
    }
}

/// Manages lifecycle of softener daemons.
pub struct DaemonManager {
    daemons: Vec<DaemonConfig>,
    processes: HashMap<String, Child>,
}

impl DaemonManager {
    /// Maximum number of times to attempt restarting a failed daemon before giving up.
    const MAX_RESTART_ATTEMPTS: u32 = 5;
    /// Number of seconds to wait between monitoring loops
    const MONITOR_RATE: Duration = Duration::from_secs(5);
    /// Number of seconds to wait between restart attempts
    const RESTART_DELAY: Duration = Duration::from_secs(1);

    /// Initialize with list of daemons to manage.
    ///
    /// # Arguments
    ///
    /// * `daemons` - Vector of daemon configurations to manage
    ///
    /// # Returns: `Self`
    /// * New DaemonManager instance
    pub fn new(daemons: Vec<DaemonConfig>) -> Self {
        Self {
            daemons,
            processes: HashMap::new(),
        }
    }

    /// Filter configured daemons to only those with available binaries.
    fn filter_available_daemons(&self) -> Vec<DaemonConfig> {
        let mut available = Vec::new();

        for daemon in &self.daemons {
            if daemon.binary_path.is_file() {
                info!(
                    "Detected {} at {}",
                    daemon.name,
                    daemon.binary_path.display()
                );
                available.push(daemon.clone());
            } else {
                info!(
                    "{} not found at {}, skipping",
                    daemon.name,
                    daemon.binary_path.display()
                );
            }
        }

        available
    }

    /// Remove stale socket file if it exists.
    fn cleanup_stale_socket(socket_path: &Path) {
        if socket_path.exists() {
            match fs::metadata(socket_path) {
                Ok(metadata) => {
                    if metadata.file_type().is_socket() {
                        info!("Removing stale socket at {}", socket_path.display());
                        if let Err(e) = fs::remove_file(socket_path) {
                            warn!("Could not remove stale socket: {}", e);
                        }
                    }
                }
                Err(e) => {
                    warn!("Warning: Could not stat socket file: {}", e);
                }
            }
        }
    }

    /// Start a daemon and wait for its socket to appear.
    ///
    /// # Arguments
    ///
    /// * `daemon` - Configuration for the daemon to start
    ///
    /// # Returns: `Result<(), FpgadError>`
    /// * `Ok(())` - Daemon started successfully and socket appeared
    /// * `Err(FpgadError::Softener)` - Failed to start daemon or socket didn't appear
    fn start_daemon(&mut self, daemon: &DaemonConfig) -> Result<(), FpgadError> {
        info!("Starting {}...", daemon.name);

        // Clean up any stale socket
        Self::cleanup_stale_socket(&daemon.socket_path);

        let log_file_path = daemon.log_file();

        // Open log file
        let log_file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&log_file_path)
            .map_err(|e| {
                FpgadError::Softener((daemon.error_constructor)(format!(
                    "Failed to open log file: {}",
                    e
                )))
            })?;

        // Create stdio objects for log file
        let log_stdout = Stdio::from(log_file.try_clone().map_err(|e| {
            FpgadError::Softener((daemon.error_constructor)(format!(
                "Failed to clone log file: {}",
                e
            )))
        })?);
        let log_stderr = Stdio::from(log_file);

        // Start the daemon process
        let child = Command::new(&daemon.binary_path)
            .stdout(log_stdout)
            .stderr(log_stderr)
            .spawn()
            .map_err(|e| {
                FpgadError::Softener((daemon.error_constructor)(format!(
                    "Failed to spawn daemon: {}",
                    e
                )))
            })?;

        let pid = child.id();
        info!("{} started with PID {}", daemon.name, pid);
        info!("Logs will be written to: {}", log_file_path.display());

        // store the process for later checking
        self.processes.insert(daemon.name.clone(), child);

        // Wait for socket to appear
        info!("Waiting for socket at {}...", daemon.socket_path.display());

        for _ in 0..daemon.start_timeout {
            // Check if socket exists
            if daemon.socket_path.exists() {
                match fs::metadata(&daemon.socket_path) {
                    Ok(metadata) if metadata.file_type().is_socket() => {
                        info!("{} socket detected - startup successful", daemon.name);
                        return Ok(());
                    }
                    _ => {}
                }
            }

            // Check if process is still running
            if let Some(process) = self.processes.get_mut(&daemon.name)
                && let Ok(Some(status)) = process.try_wait()
            {
                error!(
                    "{} process terminated unexpectedly with status: {}",
                    daemon.name, status
                );
                error!("Check logs at: {}", log_file_path.display());
                return Err(FpgadError::Softener((daemon.error_constructor)(format!(
                    "{} failed to start",
                    daemon.name
                ))));
            }

            std::thread::sleep(Duration::from_secs(1));
        }

        // Timeout reached - socket didn't appear, daemon startup failed
        error!(
            "Socket didn't appear after {}s. {} startup failed, terminating process",
            daemon.start_timeout, daemon.name
        );
        error!("Check logs at: {}", log_file_path.display());

        // Kill the process since it's not functioning properly
        if let Some(mut process) = self.processes.remove(&daemon.name) {
            if let Err(e) = process.kill() {
                error!("Failed to kill {} process: {}", daemon.name, e);
            } else {
                let _ = process.wait();
                info!("{} process terminated", daemon.name);
            }
        }

        Err(FpgadError::Softener((daemon.error_constructor)(format!(
            "{} socket did not appear within {}s timeout",
            daemon.name, daemon.start_timeout
        ))))
    }

    /// Start all detected daemons.
    ///
    /// # Returns: `Result<(), String>`
    /// * `Ok(())` - All daemons started successfully (or no daemons to start)
    /// * `Err(String)` - Failed to start one or more daemons
    pub fn check_and_start_daemons(&mut self) -> Result<(), String> {
        let available = self.filter_available_daemons();

        if available.is_empty() {
            info!("No daemons to start");
            return Ok(());
        }

        for daemon in &available {
            self.start_daemon(daemon).map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    /// Attempt to restart a failed daemon multiple times.
    ///
    /// This method repeatedly attempts to restart a daemon up to `MAX_RESTART_ATTEMPTS` times,
    /// waiting 5 seconds between each attempt. If any restart succeeds, the function returns
    /// early with success. If all attempts fail, returns an error.
    ///
    /// # Arguments
    ///
    /// * `daemon` - Configuration for the daemon to restart
    ///
    /// # Returns: `Result<(), FpgadError>`
    /// * `Ok(())` - Daemon successfully restarted on at least one attempt
    /// * `Err(FpgadError::Softener)` - All restart attempts failed
    async fn try_restart_daemon(&mut self, daemon: &DaemonConfig) -> Result<(), FpgadError> {
        for attempt in 0..Self::MAX_RESTART_ATTEMPTS {
            warn!(
                "Attempting to restart {} (attempt {}/{})",
                daemon.name,
                attempt,
                Self::MAX_RESTART_ATTEMPTS
            );

            // Wait a bit before restarting
            sleep(Self::RESTART_DELAY).await;

            match self.start_daemon(daemon) {
                Ok(_) => {
                    info!("{} successfully restarted", daemon.name);
                    return Ok(());
                }
                Err(e) => {
                    error!("Failed to restart {}: {}", daemon.name, e);
                }
            }
        }
        Err(FpgadError::Softener((daemon.error_constructor)(format!(
            "Failed to restart {} after {} attempts",
            daemon.name,
            Self::MAX_RESTART_ATTEMPTS
        ))))
    }

    /// Monitor daemon processes and keep them alive.
    ///
    /// Continuously monitors all managed daemon processes. If a daemon dies unexpectedly,
    /// attempts to restart it up to `MAX_RESTART_ATTEMPTS` times. If restart attempts are
    /// exhausted, that daemon is abandoned while others continue running. This function
    /// runs indefinitely until the task is cancelled.
    pub async fn monitor_daemons(&mut self) {
        if self.processes.is_empty() {
            info!("No daemons to monitor");
            return;
        }

        info!("Monitoring {} daemon(s)...", self.processes.len());

        loop {
            let mut daemons_to_restart = Vec::new();

            // Check all processes and collect names of dead ones
            for (name, process) in self.processes.iter_mut() {
                match process.try_wait() {
                    Ok(Some(status)) => {
                        error!("{} process died unexpectedly with status: {}", name, status);
                        daemons_to_restart.push(name.clone());
                    }
                    Ok(None) => {}
                    Err(e) => {
                        error!("Error checking process status for {}: {}", name, e);
                    }
                }
            }

            // Attempt to restart failed daemons - abandoning any that cannot be restarted
            for daemon_name in daemons_to_restart {
                // Remove the dead process
                self.processes.remove(&daemon_name);

                if let Some(daemon) = self.daemons.iter().find(|d| d.name == daemon_name).cloned() {
                    self.try_restart_daemon(&daemon).await.unwrap_or_else(|e| {
                        error!("Abandoning daemon: {}", e);
                    })
                } else {
                    error!("Could not find daemon config for daemon with name {daemon_name}")
                }
            }

            sleep(Self::MONITOR_RATE).await;
        }
    }

    /// Clean up all managed processes.
    ///
    /// Terminates all running daemon processes and waits for them to exit.
    /// This method is automatically called when the DaemonManager is dropped.
    pub fn cleanup(&mut self) {
        info!("Cleaning up daemon processes...");

        for (name, mut process) in self.processes.drain() {
            match process.try_wait() {
                Ok(Some(_)) => {
                    // Already exited
                }
                Ok(None) => {
                    // Still running, terminate it
                    info!("Terminating {} (PID {})...", name, process.id());

                    if let Err(e) = process.kill() {
                        error!("Failed to kill {}: {}", name, e);
                    } else {
                        // Wait for process to exit
                        let _ = process.wait();
                    }
                }
                Err(e) => {
                    error!("Error checking status of {}: {}", name, e);
                }
            }
        }
    }
}

impl Drop for DaemonManager {
    fn drop(&mut self) {
        self.cleanup();
    }
}

/// Get managed daemon configurations based on environment.
fn get_managed_daemons() -> Vec<DaemonConfig> {
    let prefix = if let Ok(snap_env) = env::var("SNAP_COMPONENTS") {
        snap_env + "/dfx-mgr"
    } else {
        "".to_string()
    };

    vec![
        DaemonConfig::new(
            "dfx-mgrd".to_string(),
            PathBuf::from(format!("{}/usr/bin/dfx-mgrd", prefix)),
            PathBuf::from("/run/dfx-mgrd.socket"),
            10,
            FpgadSoftenerError::DfxMgr,
        ), // Add more daemon configurations here as needed
           // DaemonConfig::new(
           //     "another-daemon".to_string(),
           //     PathBuf::from(format!("{}/usr/bin/another-daemon", snap_path)),
           //     PathBuf::from("/run/another-daemon.socket"),
           // )
    ]
}

/// Run the softener daemon wrapper in the current task.
///
/// This is the main entry point for the daemon management subsystem. It creates
/// a daemon manager with environment-appropriate configuration, starts all available
/// daemons, and monitors them until shutdown. This function should be called from
/// a spawned tokio task.
///
/// # Environment Variables
///
/// * `SNAP` - If set, daemon binaries are expected at `$SNAP/usr/bin/`
/// * `SNAP_COMMON` - If set, log files are written to `$SNAP_COMMON/log/`
///
/// # Examples
///
/// ```rust,no_run
/// # use daemon::softeners::softeners_thread::run_softener_daemons;
/// #[tokio::main]
/// async fn main() {
///     if std::env::var("SNAP").is_ok() {
///         tokio::spawn(async {
///             run_softener_daemons().await;
///         });
///     }
/// }
/// ```
pub async fn run_softener_daemons() {
    info!("Starting softener daemon wrapper...");

    let daemons = get_managed_daemons();
    let mut manager = DaemonManager::new(daemons);

    // Start all daemons
    match manager.check_and_start_daemons() {
        Ok(_) => {
            // Monitor daemons until termination
            manager.monitor_daemons().await;
        }
        Err(e) => {
            error!("Failed to start one or more daemons: {}", e);
        }
    }

    info!("Daemon wrapper exiting");
}
