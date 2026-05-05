#!/usr/bin/env python3
"""
Helper utilities and data structures for FPGA snap tests.
"""

import os
import shutil
from pathlib import Path


BAD_FLAGS = 223


def is_snap_environment() -> bool:
    """
    Check if running in a snap environment.

    :return: True if SNAP environment variable is set, False otherwise
    """
    return os.getenv("SNAP") is not None


def is_dfx_mgr_available() -> bool:
    """
    Check if the dfx-mgr component is installed and available.

    :return: True if dfx-mgr-client is available, False otherwise
    """
    # Check if running in snap environment
    snap_components = os.getenv("SNAP_COMPONENTS")
    if snap_components:
        dfx_mgr_client_path = f"{snap_components}/dfx-mgr/usr/bin/dfx-mgr-client"
        return os.path.exists(dfx_mgr_client_path)
    else:
        # Not in snap environment, check system path
        return shutil.which("dfx-mgr-client") is not None


def get_test_data_path() -> Path:
    """
    Get the path to test data files.

    In snap environment with test component installed, returns the component data path.
    Otherwise returns the local fpgad directory path.

    :return: Path to test data directory containing k26-starter-kits, etc.
    """
    snap_components = os.getenv("SNAP_COMPONENTS")
    if snap_components:
        # In snap with test component installed
        test_data_path = Path(snap_components) / "test" / "data"
        if test_data_path.exists():
            print(
                f"{Colors.CYAN}[INFO]{Colors.RESET} Using test component data at: {test_data_path}"
            )
            return test_data_path

    # Fallback to local directory
    local_path = Path("./fpgad")
    print(f"{Colors.CYAN}[INFO]{Colors.RESET} Using local test data at: {local_path}")
    return local_path


class Colors:
    """ANSI color codes for terminal output."""

    GREEN = "\033[92m"
    RED = "\033[91m"
    YELLOW = "\033[93m"
    CYAN = "\033[96m"
    RESET = "\033[0m"


class TestData:
    """
    Container for copying files during tests.

    Defines the source and target locations for use with
    copy_test_data_files and cleanup_test_data_files.
    """

    def __init__(self, source: Path, target: Path):
        """
        Initialize test data paths.

        :param source: the path to the file which should be copied
        :param target: the path to which the file should be copied/was copied to
        """
        self.source = source
        self.target = target


def copy_test_data_files(test_file: TestData) -> int:
    """
    Copy a file from test_file.source to test_file.target.

    Use in conjunction with cleanup_test_data_files to test loading from a custom location.

    :param test_file: A TestData object which contains the relevant paths
    :return: 0 on success, -1 on failure
    """
    src = Path(test_file.source)

    if not src.exists():
        print(f"{Colors.YELLOW}[WARN]{Colors.RESET} Source file missing: {src}")
        return -1

    target = test_file.target
    target_path = Path(target)

    # Ensure directory exists
    target_path.parent.mkdir(parents=True, exist_ok=True)

    print(f"{Colors.CYAN}[INFO]{Colors.RESET} Copying {src} → {target_path}")
    shutil.copy2(src, target_path)
    return 0


def cleanup_test_data_files(test_file: TestData) -> int:
    """
    Clean up the file located at test_file.target.

    Use after copy_test_data_files.

    :param test_file: A TestData object which contains the location to which the file was originally copied
    :return: 0 on success, -1 on failure
    """
    target = test_file.target
    path = Path(target)
    print(f"{Colors.CYAN}[INFO]{Colors.RESET} deleting {test_file.target}")
    if not path.exists():
        print(
            f"{Colors.YELLOW}[WARN]{Colors.RESET} Missing file during cleanup: {path}"
        )
        return -1
    try:
        path.unlink()
    except Exception as e:
        print(f"{Colors.RED}[ERROR]{Colors.RESET} Failed to remove {path}: {e}")
        return -1

    return 0


def cleanup_applied_overlays():
    """Remove all applied device tree overlays."""
    directory = "/sys/kernel/config/device-tree/overlays/"
    print(f"{Colors.CYAN}[INFO]{Colors.RESET} Cleaning up applied overlays")

    for item in os.listdir(directory):
        item_path = os.path.join(directory, item)
        if os.path.isdir(item_path):
            try:
                os.rmdir(item_path)  # Remove the directory itself
                print(
                    f"{Colors.CYAN}[INFO]{Colors.RESET} Removed overlay directory at {item_path}"
                )
            except PermissionError:
                print(
                    f"{Colors.RED}[ERROR]{Colors.RESET} Permission denied removing {item_path}. Run as root."
                )
            except OSError as e:
                # This happens if the directory is not empty
                print(
                    f"{Colors.RED}[ERROR]{Colors.RESET} Failed to remove {item_path}: {e}"
                )


def set_flags(flags: int = 0) -> None:
    """
    Set FPGA manager flags directly via sysfs.

    :param flags: Flag value to write (default: 0)
    """
    flags_path = r"/sys/class/fpga_manager/fpga0/flags"
    try:
        with open(flags_path, "w") as f:
            f.write(f"{flags:X}")
        print(
            f"{Colors.CYAN}[INFO]{Colors.RESET} Successfully wrote {flags} to {flags_path}"
        )
    except PermissionError:
        print(
            f"{Colors.RED}[ERROR]{Colors.RESET} Permission denied: you probably need to run as root"
        )
    except FileNotFoundError:
        print(f"{Colors.RED}[ERROR]{Colors.RESET} {flags_path} does not exist")
    except Exception as e:
        print(f"Error writing to {flags_path}: {e}")
