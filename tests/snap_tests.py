#!/usr/bin/env python3
"""
FPGA snap test framework using a well-defined named test type
with human-readable output.
"""

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


class TestStringMethods(unittest.TestCase):
    # ============================================================
    # ======================= USEFUL DATA ========================
    # ============================================================

    class Colors:
        GREEN = "\033[92m"
        RED = "\033[91m"
        YELLOW = "\033[93m"
        CYAN = "\033[96m"
        RESET = "\033[0m"

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

    def copy_test_data_files(self, test_file: TestData) -> int:
        """
        Copied a file from test_file.source to test_file.target, use to, e.g., copy a bitstream.
        Use in conjunction with cleanup_test_data_files to test loading from a custom location.
        :rtype: int
        :param test_file: A TestData object which contains the relevant paths
        :return: 0 on success, -1 on failure
        """
        src = Path(test_file.source)

        if not src.exists():
            print(
                f"\n{self.Colors.YELLOW}[WARN]{self.Colors.RESET} Source file missing: {src}"
            )
            return -1

        target = test_file.target
        target_path = Path(target)

        # Ensure directory exists
        target_path.parent.mkdir(parents=True, exist_ok=True)

        print(
            f"\n{self.Colors.CYAN}[INFO]{self.Colors.RESET} Copying {src} → {target_path}"
        )
        shutil.copy2(src, target_path)
        return 0

    def cleanup_test_data_files(self, test_file: TestData) -> int:
        """
        Cleans up the file located at test_file.target, use after copy_test_data_files
        :rtype: int
        :param test_file: A TestData object which contains the location to which the file was originally copied
        :return: 0 on success, -1 on failure
        """
        target = test_file.target
        path = Path(target)
        print(
            f"\n{self.Colors.CYAN}[INFO]{self.Colors.RESET} deleting {test_file.target}"
        )
        if not path.exists():
            print(
                f"{self.Colors.YELLOW}[WARN]{self.Colors.RESET} Missing file during cleanup: {path}"
            )
            return -1
        try:
            path.unlink()
        except Exception as e:
            print(
                f"{self.Colors.RED}[ERROR]{self.Colors.RESET} Failed to remove {path}: {e}"
            )
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

    def cleanup_applied_overlays(self):
        """
        Wrapper to handle discovering and removing all overlays.
        Use before attempting to test the loading of an overlay.
        :return: the completed process object, containing return code and captured output
        """
        # todo: implement
        raise NotImplementedError()

    def run_fpgad(self, args: List[str]) -> subprocess.CompletedProcess[str]:
        """
        Run the fpgad cli with provided args as a subprocess
        :rtype: subprocess.CompletedProcess[str]
        :param args: list of arguments to provide to the fpgad cli call
        :return: the completed process object, containing return code and captured output
        """
        cmd = ["fpgad"] + args
        print(f"\n{self.Colors.CYAN}[INFO]{self.Colors.RESET} Running: {' '.join(cmd)}")

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

    def test_status_with_bitstream(self):
        self.cleanup_applied_overlays()
        pass

    def test_status_with_overlay(self):
        # todo: check for existing overlay, and remove if there.
        self.cleanup_applied_overlays()
        pass

    def test_status_failed_overlay(self):
        # todo: check for existing overlay, and remove if there.
        self.cleanup_applied_overlays()
        pass

    # --------------------------------------------------------
    # set tests
    # --------------------------------------------------------

    # --------------------------------------------------------
    # help tests
    # --------------------------------------------------------


if __name__ == "__main__":
    unittest.main()
