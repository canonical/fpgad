#!/usr/bin/env python3
"""
FPGA snap test framework using a well-defined named test type
with human-readable output.
"""

import os
import subprocess
import unittest
from pathlib import Path
from subprocess import CompletedProcess
from typing import List

import shutil

BAD_FLAGS = 223


class Colors:
    GREEN = "\033[92m"
    RED = "\033[91m"
    YELLOW = "\033[93m"
    CYAN = "\033[96m"
    RESET = "\033[0m"


class TestFPGAdCLI(unittest.TestCase):
    def setUp(self):
        """
        Runs before each tests in this class.
        """
        self.cleanup_applied_overlays()
        self.reset_flags()

    @classmethod
    def tearDownClass(cls):
        """
        Runs once after all tests in this class are finished
        """
        cls.cleanup_applied_overlays()
        cls.set_flags(0)

    # ============================================================
    # ======================= USEFUL DATA ========================
    # ============================================================

    class TestData:
        def __init__(self, source: Path, target: Path):
            """
            Useful container for copying files during tests. Defines the source and target locations for use with
            copy_test_data_files and cleanup_test_data_files
            :param source: the path to the file which should be copied
            :param target: the path to which the file should be copied/was copied to
            """
            self.source = source
            self.target = target

    # ============================================================
    # ===================== HELPER FUNCTIONS =====================
    # ============================================================
    def assert_proc_succeeds(self, proc, msg=None):
        """Assert that a process completed successfully, including stdout/stderr on failure."""
        if msg is None:
            msg = f"Return code is {proc.returncode} when expecting 0"
        full_msg = (
            f"{msg}\n"
            f"Status code:\t{proc.returncode}\n"
            f"stdout:\t{proc.stdout}\n"
            f"stderr:\t{proc.stderr}"
        )
        self.assertEqual(proc.returncode, 0, full_msg)

    def assert_proc_fails(self, proc, msg=None):
        """Assert that a process completed successfully, including stdout/stderr on failure."""
        if msg is None:
            msg = f"Return code is {proc.returncode} when expecting nonzero"
        full_msg = (
            f"{msg}\n"
            f"Status code:\t{proc.returncode}\n"
            f"stdout:\t{proc.stdout}\n"
            f"stderr:\t{proc.stderr}"
        )
        self.assertNotEqual(proc.returncode, 0, full_msg)

    def assert_in_proc_out(
        self, substring: str, proc: CompletedProcess, msg: str = None
    ):
        """Assert that a substring exists in output, including stdout/stderr on failure."""
        if msg is None:
            msg = f"'{substring}' not found in output."
        full_msg = f"{msg}\nstdout:\t{proc.stdout}\nstderr:\t{proc.stderr}"
        self.assertIn(substring, proc.stdout, full_msg)

    def assert_not_in_proc_out(
        self, substring: str, proc: CompletedProcess, msg: str = None
    ):
        """Assert that a substring exists in output, including stdout/stderr on failure."""
        if msg is None:
            msg = f"Undesired '{substring}' found in output."
        full_msg = f"{msg}\nstdout:\t{proc.stdout}\nstderr:\t{proc.stderr}"
        self.assertNotIn(substring, proc.stdout, full_msg)

    def assert_in_proc_err(
        self, substring: str, proc: CompletedProcess, msg: str = None
    ):
        """Assert that a substring exists in output, including stdout/stderr on failure."""
        if msg is None:
            msg = f"'{substring}' not found in stderr."
        full_msg = f"{msg}\nstdout:\t{proc.stdout}\nstderr:\t{proc.stderr}"
        self.assertIn(substring, proc.stderr, full_msg)

    @staticmethod
    def get_fpga0_attribute(attr: str):
        path = Path(f"/sys/class/fpga_manager/fpga0/{attr}")

        with open(path, "r") as f:
            real_attr = f.read().strip()
        return real_attr

    def check_fpga0_attribute(self, attr: str, expected: str):
        path = Path(f"/sys/class/fpga_manager/fpga0/{attr}")

        with open(path, "r") as f:
            real_attr = f.read().strip()
        self.assertIn(expected, real_attr)

    @staticmethod
    def copy_test_data_files(test_file: TestData) -> int:
        """
        Copied a file from test_file.source to test_file.target, use to, e.g., copy a bitstream.
        Use in conjunction with cleanup_test_data_files to test loading from a custom location.
        :rtype: int
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

    @staticmethod
    def cleanup_test_data_files(test_file: TestData) -> int:
        """
        Cleans up the file located at test_file.target, use after copy_test_data_files
        :return:
        :rtype: int
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

    def load_bitstream(self, path: Path) -> CompletedProcess[str]:
        """
        One line wrapper for calling fpgad to load a bitstream
        (may have more functionality added in future)
        :rtype: CompletedProcess[str]
        :param path: path to the bitstream to load
        :return:
        """
        return self.run_fpgad(["load", "bitstream", str(path)])

    def load_overlay(self, path: Path) -> CompletedProcess[str]:
        """
        One line wrapper for calling fpgad to load an overlay
        (may have more functionality added in future)
        :rtype: CompletedProcess[str]
        :param path: path to the overlay to load
        :return: the completed process object, containing return code and captured output
        """
        return self.run_fpgad(["load", "overlay", str(path)])

    @staticmethod
    def cleanup_applied_overlays():
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

    @staticmethod
    def set_flags(flags: int = 0) -> None:
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

    def reset_flags(self):
        """
        Reset flags (to zero) using system calls, instead of fpgad.
        :return: the completed process object, containing return code and captured output
        """
        print(f"{Colors.CYAN}[INFO]{Colors.RESET} Resetting fpga0's flags to 0")
        self.set_flags(0)

    def run_fpgad(self, args: List[str]) -> subprocess.CompletedProcess[str]:
        """
        Run the fpgad cli with provided args as a subprocess
        :rtype: subprocess.CompletedProcess[str]
        :param args: list of arguments to provide to the fpgad cli call
        :return: the completed process object, containing return code and captured output
        """
        cmd = ["fpgad"] + args
        print(f"{Colors.CYAN}[INFO]{Colors.RESET} Running: {' '.join(cmd)}")

        proc = subprocess.run(
            cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True
        )

        return proc

    # ============================================================
    # ===================== TEST DEFINITIONS =====================
    # ============================================================

    # --------------------------------------------------------
    # load bitstream tests
    # --------------------------------------------------------

    def test_load_bitstream_local(self):
        path_str = "./fpgad/k26-starter-kits/k26_starter_kits.bit.bin"
        proc = self.load_bitstream(Path(path_str))
        self.assert_proc_succeeds(proc)
        self.assertIn("loaded to fpga0 using firmware lookup path", proc.stdout)

    def test_load_bitstream_home_fullpath(self):
        prefix = Path(os.getcwd())
        path = prefix.joinpath("fpgad/k26-starter-kits/k26_starter_kits.bit.bin")

        proc = self.load_bitstream(path)
        self.assert_proc_succeeds(proc)
        self.assert_in_proc_out("loaded to fpga0 using firmware lookup path", proc)

    def test_load_bitstream_lib_firmware(self):
        test_file_paths = self.TestData(
            source=Path("./fpgad/k26-starter-kits/k26_starter_kits.bit.bin"),
            target=Path("/lib/firmware/k26-starter-kits.bit.bin"),
        )
        try:
            self.copy_test_data_files(test_file_paths) != 0
        except Exception as e:
            print(
                f"Failed to copy {test_file_paths.source} to {test_file_paths.target}"
            )
            raise e

        proc = self.load_bitstream(test_file_paths.target)

        try:
            self.cleanup_test_data_files(test_file_paths)
        except Exception as e:
            print(f"Failed to clean up {test_file_paths.target}")
            raise e

        self.assert_proc_succeeds(proc)
        self.assert_in_proc_out("loaded to fpga0 using firmware lookup path", proc)

    def test_load_bitstream_lib_firmware_xilinx(self):
        test_file_paths = self.TestData(
            source=Path("./fpgad/k26-starter-kits/k26_starter_kits.bit.bin"),
            target=Path(
                "/lib/firmware/xilinx/k26_starter_kits/k26-starter-kits.bit.bin"
            ),
        )
        try:
            self.copy_test_data_files(test_file_paths) != 0
        except Exception as e:
            print(
                f"Failed to copy {test_file_paths.source} to {test_file_paths.target}"
            )
            raise e

        proc = self.load_bitstream(test_file_paths.target)

        try:
            self.cleanup_test_data_files(test_file_paths)
        except Exception as e:
            print(f"Failed to clean up {test_file_paths.target}")
            raise e

        self.assert_proc_succeeds(proc)
        self.assert_in_proc_out("loaded to fpga0 using firmware lookup path", proc)

    def test_load_bitstream_path_not_exist(self):
        proc = self.load_bitstream(Path("/this/path/is/fake.bit.bin"))
        self.assert_proc_fails(proc)
        self.assert_in_proc_err("FpgadError::IOWrite:", proc)

    def test_load_bitstream_containing_dir(self):
        prefix = Path(os.getcwd())
        path = prefix.joinpath("fpgad/k26-starter-kits/")

        proc = self.load_bitstream(path)
        self.assert_proc_fails(proc)
        self.assert_in_proc_err("FpgadError::IOWrite:", proc)

    def test_load_bitstream_bad_flags(self):
        prefix = Path(os.getcwd())
        path = prefix.joinpath("fpgad/k26-starter-kits/k26_starter_kits.bit.bin")

        self.set_flags(BAD_FLAGS)

        proc = self.load_bitstream(path)
        self.assert_proc_fails(proc)
        self.assert_in_proc_err("FpgadError::IOWrite:", proc)

    # --------------------------------------------------------
    # load overlay tests
    # --------------------------------------------------------

    ### overlay cases:
    #  load from relative path
    #  load from /lib/firmware
    #  load from full path not in /lib/firmware
    #  fail to load from bad path
    #  fail to load due to bad flags

    def test_load_overlay_local(self):
        # Necessary due to bad dtbo content from upstream
        test_file_paths = self.TestData(
            source=Path("./fpgad/k26-starter-kits/k26_starter_kits.bit.bin"),
            target=Path("./fpgad/k26-starter-kits/k26-starter-kits.bit.bin"),
        )

        try:
            self.copy_test_data_files(test_file_paths) != 0
        except Exception as e:
            print(
                f"Failed to copy {test_file_paths.source} to {test_file_paths.target}"
            )
            raise e
        overlay_path = Path("./fpgad/k26-starter-kits/k26_starter_kits.dtbo")
        proc = self.load_overlay(overlay_path)
        try:
            self.cleanup_test_data_files(test_file_paths)
        except Exception as e:
            print(f"Failed to clean up {test_file_paths.target}")
            raise e

        self.assert_proc_succeeds(proc)
        self.assert_in_proc_out("loaded via", proc)

    def test_load_overlay_lib_firmware(self):
        # Necessary due to bad dtbo content from upstream
        test_file_paths = [
            self.TestData(
                source=Path("./fpgad/k26-starter-kits/k26_starter_kits.bit.bin"),
                target=Path("/lib/firmware/k26-starter-kits.bit.bin"),
            ),
            self.TestData(
                source=Path("./fpgad/k26-starter-kits/k26_starter_kits.dtbo"),
                target=Path("/lib/firmware/k26-starter-kits.dtbo"),
            ),
        ]
        for file in test_file_paths:
            try:
                self.copy_test_data_files(file) != 0
            except Exception as e:
                print(f"Failed to copy {file.source} to {file.target}")
                raise e

        overlay_path = Path("/lib/firmware/k26-starter-kits.dtbo")
        proc = self.load_overlay(overlay_path)
        for file in test_file_paths:
            try:
                self.cleanup_test_data_files(file)
            except Exception as e:
                print(f"Failed to clean up {file.target}")
                raise e

        self.assert_proc_succeeds(proc)
        self.assert_in_proc_out("loaded via", proc)

    def test_load_overlay_full_path(self):
        # Necessary due to bad dtbo content from upstream
        test_file_paths = self.TestData(
            source=Path("./fpgad/k26-starter-kits/k26_starter_kits.bit.bin"),
            target=Path("./fpgad/k26-starter-kits/k26-starter-kits.bit.bin"),
        )

        try:
            self.copy_test_data_files(test_file_paths) != 0
        except Exception as e:
            print(
                f"Failed to copy {test_file_paths.source} to {test_file_paths.target}"
            )
            raise e
        prefix = Path(os.getcwd())
        overlay_path = prefix.joinpath("fpgad/k26-starter-kits/k26_starter_kits.dtbo")
        proc = self.load_overlay(overlay_path)
        try:
            self.cleanup_test_data_files(test_file_paths)
        except Exception as e:
            print(f"Failed to clean up {test_file_paths.target}")
            raise e

        self.assert_proc_succeeds(proc)
        self.assert_in_proc_out("loaded via", proc)

    def test_load_overlay_bad_path(self):
        overlay_path = Path("/path/does/not/exist")
        proc = self.load_overlay(overlay_path)
        self.assert_proc_fails(proc)
        self.assert_not_in_proc_out("loaded via", proc)
        self.assert_in_proc_err("FpgadError::OverlayStatus:", proc)

    def test_load_overlay_missing_bitstream(self):
        # TODO: if the dtbo gets fixed, then this test needs to be re-written.
        overlay_path = Path("./fpgad/k26-starter-kits/k26_starter_kits.dtbo")
        proc = self.load_overlay(overlay_path)
        self.assert_proc_fails(proc)
        self.assert_not_in_proc_out("loaded via", proc)
        self.assert_in_proc_err("FpgadError::OverlayStatus:", proc)

    def test_load_overlay_bad_flags(self):
        self.set_flags(BAD_FLAGS)
        # Necessary due to bad dtbo content from upstream
        test_file_paths = self.TestData(
            source=Path("./fpgad/k26-starter-kits/k26_starter_kits.bit.bin"),
            target=Path("./fpgad/k26-starter-kits/k26-starter-kits.bit.bin"),
        )

        try:
            self.copy_test_data_files(test_file_paths) != 0
        except Exception as e:
            print(
                f"Failed to copy {test_file_paths.source} to {test_file_paths.target}"
            )
            raise e
        overlay_path = Path("./fpgad/k26-starter-kits/k26_starter_kits.dtbo")
        proc = self.load_overlay(overlay_path)
        try:
            self.cleanup_test_data_files(test_file_paths)
        except Exception as e:
            print(f"Failed to clean up {test_file_paths.target}")
            raise e

        self.assert_proc_fails(proc)
        self.assert_not_in_proc_out("loaded via", proc)
        self.assert_in_proc_err("FpgadError::OverlayStatus:", proc)

    # --------------------------------------------------------
    # status tests
    # --------------------------------------------------------

    def test_status_executes(self):
        proc = self.run_fpgad(["status"])
        self.assert_proc_succeeds(proc)

    def test_status_with_bitstream(self):
        load_proc = self.load_bitstream(
            Path("./fpgad/k26-starter-kits/k26_starter_kits.bit.bin")
        )
        self.assert_proc_succeeds(
            load_proc, "Failed to load a bitstream before checking status."
        )

        status_proc = self.run_fpgad(["status"])
        self.assert_proc_succeeds(status_proc)
        self.assert_in_proc_out("operating", status_proc)

    def test_status_with_overlay(self):
        test_file_paths = self.TestData(
            source=Path("./fpgad/k26-starter-kits/k26_starter_kits.bit.bin"),
            target=Path("./fpgad/k26-starter-kits/k26-starter-kits.bit.bin"),
        )
        try:
            self.copy_test_data_files(test_file_paths) != 0
        except Exception as e:
            print(
                f"Failed to copy {test_file_paths.source} to {test_file_paths.target}"
            )
            raise e
        load_proc = self.load_overlay(
            Path("./fpgad/k26-starter-kits/k26_starter_kits.dtbo")
        )
        self.cleanup_test_data_files(test_file_paths)
        self.assert_proc_succeeds(load_proc)

        status_proc = self.run_fpgad(["status"])
        self.assert_proc_succeeds(status_proc)
        self.assert_in_proc_out("applied", status_proc)
        self.assert_in_proc_out("operating", status_proc)
        self.assert_in_proc_out("k26_starter_kits.dtbo", status_proc)
        self.assert_not_in_proc_out("error", status_proc)

    def test_status_failed_overlay(self):
        load_proc = self.load_overlay(
            Path("./fpgad/k26-starter-kits/k26_starter_kits.dtbo")
        )
        self.assertNotEqual(
            load_proc.returncode,
            0,
            "Overlay load succeeded and therefore test has failed.",
        )

        proc = self.run_fpgad(["status"])
        self.assert_proc_succeeds(proc)
        self.assert_in_proc_out("error", proc)

    # --------------------------------------------------------
    # set tests
    # --------------------------------------------------------

    def test_set_flags_nonzero(self):
        proc = self.run_fpgad(["set", "flags", "20"])
        self.assert_proc_succeeds(proc)
        self.assert_in_proc_out(
            "20 written to /sys/class/fpga_manager/fpga0/flags", proc
        )
        self.check_fpga0_attribute("flags", "20")

    def test_set_flags_string(self):
        proc = self.run_fpgad(["set", "flags", "zero"])
        self.assert_proc_fails(proc)
        self.check_fpga0_attribute("flags", "0")

    def test_set_state(self):
        old = self.get_fpga0_attribute("state")

        proc = self.run_fpgad(["set", "state", "0"])
        self.assert_proc_fails(proc)
        self.check_fpga0_attribute("state", old)

    def test_set_flags_float(self):
        proc = self.run_fpgad(["set", "flags", "0.2"])
        self.assert_proc_fails(proc)
        self.check_fpga0_attribute("flags", "0")

    def test_set_flags_zero(self):
        proc = self.run_fpgad(["set", "flags", "0"])
        self.assert_proc_succeeds(proc)
        self.assert_in_proc_out(
            "0 written to /sys/class/fpga_manager/fpga0/flags", proc
        )
        self.check_fpga0_attribute("flags", "0")

    # --------------------------------------------------------
    # help tests
    # --------------------------------------------------------

    def test_help_main(self):
        proc = self.run_fpgad(["help"])
        self.assert_proc_succeeds(proc)

    def test_help_main_as_flag(self):
        proc = self.run_fpgad(["--help"])
        self.assert_proc_succeeds(proc)

    def test_help_set(self):
        proc = self.run_fpgad(["help", "set"])
        self.assert_proc_succeeds(proc)

    def test_help_remove(self):
        proc = self.run_fpgad(["help", "remove"])
        self.assert_proc_succeeds(proc)

    def test_help_remove_overlay(self):
        proc = self.run_fpgad(["help", "remove", "overlay"])
        self.assert_proc_succeeds(proc)

    def test_help_remove_bitstream(self):
        proc = self.run_fpgad(["help", "remove", "bitstream"])
        self.assert_proc_succeeds(proc)

    def test_help_load(self):
        proc = self.run_fpgad(["help", "load"])
        self.assert_proc_succeeds(proc)

    def test_help_load_bitstream(self):
        proc = self.run_fpgad(["help", "load", "bitstream"])
        self.assert_proc_succeeds(proc)

    def test_help_load_overlay(self):
        proc = self.run_fpgad(["help", "load", "overlay"])
        self.assert_proc_succeeds(proc)


if __name__ == "__main__":
    unittest.main()
