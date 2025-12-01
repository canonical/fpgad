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


### bitstream cases:
#  load from relative path
#  load from /lib/firmware (copy it in, the delete after to maintain clean fw_path testing conditions)
#  load from /lib/firmware/xilinx
#  load from full path not in /lib/firmware/
#  fail to load from bad path
#  todo: fail to load due to bad flags

### overlay cases:
#  load from relative path
#  load from /lib/firmware
#  load from full path not in /lib/firmware
#  fail to load from bad path
#  fail to load due to bad flags

### status cases:
#  with only bitstream, no overlay
#  with loaded bitstream and overlay
#  after failing to load a bitstream (wrong bitstream)
#  after failing to load an overlay (bad path/bitstream name)

### set cases:
# set flags to string?
# set flags to float/negative?
# set RO value like state?
# set flags to 0


### help cases:
# check help at root
# check for load
# check for load bitstream/overlay
# check for remove
# check for set
# check for status


class Colors:
    GREEN = "\033[92m"
    RED = "\033[91m"
    YELLOW = "\033[93m"
    CYAN = "\033[96m"
    RESET = "\033[0m"


class TestStringMethods(unittest.TestCase):
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

    @classmethod
    def setUpClass(cls):
        """
        Runs once before all tests in this class. cls can only be used to call class methods or static methods
        """
        cls.cleanup_applied_overlays()
        cls.reset_flags()

    @classmethod
    def tearDownClass(cls):
        """
        Runs once after all tests in this class
        """
        cls.cleanup_applied_overlays()
        cls.reset_flags()

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
            print(f"\n{Colors.YELLOW}[WARN]{Colors.RESET} Source file missing: {src}")
            return -1

        target = test_file.target
        target_path = Path(target)

        # Ensure directory exists
        target_path.parent.mkdir(parents=True, exist_ok=True)

        print(f"\n{Colors.CYAN}[INFO]{Colors.RESET} Copying {src} → {target_path}")
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
        print(f"\n{Colors.CYAN}[INFO]{Colors.RESET} deleting {test_file.target}")
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
        """
        Wrapper to handle discovering and removing all overlays using system calls instead of fpgad.
        Use before attempting to test the loading of an overlay.
        :return: the completed process object, containing return code and captured output
        """
        directory = r"/sys/kernel/config/device-tree/overlays/"
        print(f"\n{Colors.CYAN}[INFO]{Colors.RESET} Cleaning up applied overlays")
        # Loop through all overlays and delete them
        for item in os.listdir(directory):
            item_path = os.path.join(directory, item)
            # Check if the item is a folder
            if os.path.isdir(item_path):
                # Delete the folder and all its contents
                shutil.rmtree(item_path)
                print(
                    f"\n{Colors.CYAN}[INFO]{Colors.RESET} deleting overlay at {item_path}"
                )

    @staticmethod
    def reset_flags():
        """
        Reset flags (to zero) using system calls, instead of fpgad.
        :return: the completed process object, containing return code and captured output
        """
        flags_path = r"/sys/class/fpga_manager/fpga0/flags"
        default_flags = "0"
        print(f"\n{Colors.CYAN}[INFO]{Colors.RESET} Resetting fpga0's flags")
        try:
            with open(flags_path, "w") as f:
                f.write(default_flags)
            print(
                f"{Colors.CYAN}[INFO]{Colors.RESET} Successfully wrote {default_flags} to {flags_path}"
            )
        except PermissionError:
            print(
                f"{Colors.RED}[ERROR]{Colors.RESET} Permission denied: you probably need to run as root"
            )
        except FileNotFoundError:
            print(f"{Colors.RED}[ERROR]{Colors.RESET} {flags_path} does not exist")
        except Exception as e:
            print(f"Error writing to {flags_path}: {e}")

    def run_fpgad(self, args: List[str]) -> subprocess.CompletedProcess[str]:
        """
        Run the fpgad cli with provided args as a subprocess
        :rtype: subprocess.CompletedProcess[str]
        :param args: list of arguments to provide to the fpgad cli call
        :return: the completed process object, containing return code and captured output
        """
        cmd = ["fpgad"] + args
        print(f"\n{Colors.CYAN}[INFO]{Colors.RESET} Running: {' '.join(cmd)}")

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
        self.assertEqual(
            proc.returncode,
            0,
            msg=f"Error code {proc.returncode} expecting 0 - failed to load {path_str}:\nstderr:\t{proc.stderr}",
        )
        self.assertIn("loaded to fpga0 using firmware lookup path", proc.stdout)

    def test_load_bitstream_home_fullpath(self):
        path_str = "$(pwd)/fpgad/k26-starter-kits/k26_starter_kits.bit.bin"
        proc = self.load_bitstream(Path(path_str))
        self.assertEqual(
            proc.returncode,
            0,
            msg=f"Error code {proc.returncode} expecting 0 - failed to load {path_str}:\nstderr:\t{proc.stderr}",
        )
        self.assertIn("loaded to fpga0 using firmware lookup path", proc.stdout)

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

        self.assertEqual(
            proc.returncode,
            0,
            msg=f"Error code {proc.returncode} expecting 0 - failed to load {str(test_file_paths.target)}:\nstderr:\t{proc.stderr}",
        )
        self.assertIn(
            "loaded to fpga0 using firmware lookup path",
            proc.stdout,
            msg=f"`loaded to fpga0 using firmware lookup path` expected in stdout. Instead found:\nstdout:"
            f"\t{proc.stdout}\nstderr:\t{proc.stderr}",
        )

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

        self.assertEqual(
            proc.returncode,
            0,
            msg=f"Error code {proc.returncode} expecting 0 - failed to load {str(test_file_paths.target)}:\nstderr:\t{proc.stderr}",
        )
        self.assertIn(
            "loaded to fpga0 using firmware lookup path",
            proc.stdout,
            msg=f"`loaded to fpga0 using firmware lookup path` expected in stdout. Instead found:\nstdout:"
            f"\t{proc.stdout}\nstderr:\t{proc.stderr}",
        )

    def test_load_bitstream_path_not_exist(self):
        proc = self.load_bitstream(Path("/this/path/is/fake.bit.bin"))
        self.assertNotEqual(proc.returncode, 0)
        self.assertIn("FpgadError::IOWrite:", proc.stderr)
        self.assertIn(
            "failed",
            proc.stdout,
            msg=f"Expected `failed` in stderr, but found\nstdout:\t{proc.stdout}.",
        )

    def test_load_bitstream_containing_dir(self):
        proc = self.load_bitstream(Path("$(pwd)/fpgad/k26-starter-kits/"))
        self.assertNotEqual(proc.returncode, 0)
        self.assertIn("FpgadError::IOWrite:", proc.stderr)
        self.assertIn(
            "failed",
            proc.stdout,
            msg=f"Expected `failed` in stderr, but found\nstdout:\t{proc.stdout}.",
        )

    # --------------------------------------------------------
    # load overlay tests
    # --------------------------------------------------------

    # --------------------------------------------------------
    # status tests
    # --------------------------------------------------------

    ### status cases:
    #  with only bitstream, no overlay
    #  with loaded bitstream and overlay
    #  after failing to load an overlay (bad path/bitstream name)

    def test_status_executes(self):
        proc = self.run_fpgad(["status"])
        self.assertEqual(
            proc.returncode,
            0,
            f"failed to execute status command.\n"
            f"Status code:\t{proc.returncode}\n"
            f"stdout:\t{proc.stdout}\n"
            f"stderr:\t{proc.stderr}",
        )

    def test_status_with_bitstream(self):
        self.cleanup_applied_overlays()

        loaded = self.load_bitstream(
            Path("./fpgad/k26-starter-kits/k26_starter_kits.bit.bin")
        )
        self.assertEqual(
            loaded.returncode,
            0,
            "Failed to load a bitstream before checking status."
            f"Status code:\t{loaded.returncode}\n"
            f"stdout:\t{loaded.stdout}\n"
            f"stderr:\t{loaded.stderr}",
        )

        proc = self.run_fpgad(["status"])
        self.assertEqual(
            proc.returncode,
            0,
            f"failed to execute status command.\n"
            f"Status code:\t{proc.returncode}\n"
            f"stdout:\t{proc.stdout}\n"
            f"stderr:\t{proc.stderr}",
        )
        self.assertIn(
            "operating",
            proc.stdout,
            "operating not found in stdout.\n"
            f"stdout:\t{proc.stdout}\n"
            f"stderr:\t{proc.stderr}",
        )

    def test_status_with_overlay(self):
        self.cleanup_applied_overlays()

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
        self.assertEqual(
            load_proc.returncode,
            0,
            "Failed to load a overlay before checking status."
            f"Status code:\t{load_proc.returncode}\n"
            f"stdout:\t{load_proc.stdout}\n"
            f"stderr:\t{load_proc.stderr}",
        )

        proc = self.run_fpgad(["status"])
        self.assertEqual(
            proc.returncode,
            0,
            f"failed to execute status command.\n"
            f"Status code:\t{proc.returncode}\n"
            f"stdout:\t{proc.stdout}\n"
            f"stderr:\t{proc.stderr}",
        )
        self.assertIn(
            "applied",
            proc.stdout,
            "applied not found in stdout.\n"
            f"stdout:\t{proc.stdout}\n"
            f"stderr:\t{proc.stderr}",
        )
        self.assertIn(
            "operating",
            proc.stdout,
            "operating not found in stdout.\n"
            f"stdout:\t{proc.stdout}\n"
            f"stderr:\t{proc.stderr}",
        )
        self.assertIn(
            "k26_starter_kits.dtbo",
            proc.stdout,
            "k26_starter_kits.dtbo not found in stdout.\n"
            f"stdout:\t{proc.stdout}\n"
            f"stderr:\t{proc.stderr}",
        )
        self.assertNotIn(
            "error",
            proc.stdout,
            f"error found in stdout.\nstdout:\t{proc.stdout}\nstderr:\t{proc.stderr}",
        )

    def test_status_failed_overlay(self):
        self.cleanup_applied_overlays()
        load_proc = self.load_overlay(
            Path("./fpgad/k26-starter-kits/k26_starter_kits.dtbo")
        )
        self.assertNotEqual(
            load_proc.returncode,
            0,
            "Overlay load succeeded and therefore test has failed.",
        )

        proc = self.run_fpgad(["status"])
        self.assertEqual(
            proc.returncode,
            0,
            f"failed to execute status command.\n"
            f"Status code:\t{proc.returncode}\n"
            f"stdout:\t{proc.stdout}\n"
            f"stderr:\t{proc.stderr}",
        )
        self.assertIn(
            "error",
            proc.stdout,
            "expected `error` notfound in stdout.\n"
            f"stdout:\t{proc.stdout}\n"
            f"stderr:\t{proc.stderr}",
        )

    # --------------------------------------------------------
    # set tests
    # --------------------------------------------------------

    # --------------------------------------------------------
    # help tests
    # --------------------------------------------------------


if __name__ == "__main__":
    unittest.main()
