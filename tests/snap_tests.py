#!/usr/bin/env python3
"""
FPGA snap test framework using a well-defined named test type
with human-readable output.
"""

import subprocess
import sys
from pathlib import Path
from dataclasses import dataclass
from typing import Callable, List, Optional

import shutil


class Colors:
    GREEN = "\033[92m"
    RED = "\033[91m"
    YELLOW = "\033[93m"
    CYAN = "\033[96m"
    RESET = "\033[0m"


class TestData:
    def __init__(self, source: Path, target: Path):
        self.source = source
        self.target = target


def copy_test_data_files(test_file: TestData) -> int:
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


# Returns 0 on success, -1 on failure  (true is good)
def cleanup_test_data_files(test_file: TestData) -> int:
    target = test_file.target
    path = Path(target)

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


# def setup_xilinx_files():
#     print(f"{Colors.CYAN}[INFO]{Colors.RESET} Setting up Xilinx test files...")
#
#     # TODO: actually we don't want to have the properly names files in /lib/firmware, because then dtbo load is not
#     #  being tested from our set firmware lookup path
#     test_files = [
#         TestData(
#             source="./daemon/tests/test_data/k26-starter-kits/k26_starter_kits.bit.bin",
#             targets=[
#                 "/lib/firmware/k26-starter-kits.bit.bin",
#                 "/lib/firmware/xilinx/k26-starter-kits/k26_starter_kits.bit.bin",
#             ],
#         ),
#         TestData(
#             source="./daemon/tests/test_data/k24-starter-kits/k24_starter_kits.bit.bin",
#             targets=[
#                 "/lib/firmware/k24-starter-kits.bit.bin",
#                 "/lib/firmware/xilinx/k24-starter-kits/k24_starter_kits.bit.bin",
#             ],
#         ),
#         TestData(
#             source="./daemon/tests/test_data/k26-starter-kits/k26_starter_kits.dtbo",
#             targets=[
#                 "/lib/firmware/k26-starter-kits.dtbo",
#                 "/lib/firmware/xilinx/k26-starter-kits/k26_starter_kits.dtbo",
#             ],
#         ),
#         TestData(
#             source="./daemon/tests/test_data/k24-starter-kits/k24_starter_kits.dtbo",
#             targets=[
#                 "/lib/firmware/k24-starter-kits.dtbo",
#                 "/lib/firmware/xilinx/k24-starter-kits/k24_starter_kits.dtbo",
#             ],
#         ),
#     ]
#
#     for test_file in test_files:
#         copy_test_data_files(test_file)
#
#     print(f"{Colors.GREEN}[DONE]{Colors.RESET} Xilinx test files installed.")


@dataclass
class TestResult:
    name: str
    passed: bool
    message: Optional[str] = None
    returncode: Optional[int] = None
    stdout: Optional[str] = None
    stderr: Optional[str] = None


def run_fpgad(args: List[str]) -> subprocess.CompletedProcess[str]:
    cmd = ["fpgad"] + args
    print(f"{Colors.CYAN}[INFO]{Colors.RESET} Running: {' '.join(cmd)}")

    proc = subprocess.run(
        cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True
    )

    return proc


# # ------------------------------------------------------------
# # Expected result checking
# # ------------------------------------------------------------
# def check_result(result: Dict[str, Any], expected: Dict[str, Any]):
#     if "returncode" in expected:
#         assert result["returncode"] == expected["returncode"], (
#             f"Expected rc={expected['returncode']}, got {result['returncode']}"
#         )
#
#     if "stdout_contains" in expected:
#         assert expected["stdout_contains"] in result["stdout"], (
#             f"Expected stdout to contain '{expected['stdout_contains']}'"
#         )
#
#     if "stderr_contains" in expected:
#         assert expected["stderr_contains"] in result["stderr"], (
#             f"Expected stderr to contain '{expected['stderr_contains']}'"
#         )
#
#     if "stdout_not_contains" in expected:
#         assert expected["stdout_not_contains"] not in result["stdout"], (
#             f"stdout should NOT contain '{expected['stdout_not_contains']}'"
#         )
#
#     if "stderr_not_contains" in expected:
#         assert expected["stderr_not_contains"] not in result["stderr"], (
#             f"stderr should NOT contain '{expected['stderr_not_contains']}'"
#         )


# ------------------------------------------------------------
# Define tests
# ------------------------------------------------------------

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


def test_load_bitstream_local() -> TestResult:
    result = run_fpgad(
        ["load", "bitstream", "./fpgad/k26-starter-kits/k26_starter_kits.bit.bin"]
    )
    passed = (result.returncode == 0) and (result.stdout.__contains__("loaded via"))

    return TestResult(
        name="test_load_bitstream_local",
        passed=passed,
        message="Bitstream Loaded from local directory using `./`",
        returncode=result.returncode,
        stdout=result.stdout,
        stderr=result.stderr,
    )


def test_load_bitstream_home_fullpath() -> TestResult:
    result = run_fpgad(
        ["load", "bitstream", "$(pwd)/fpgad/k26-starter-kits/k26_starter_kits.bit.bin"]
    )
    passed = (result.returncode == 0) and (result.stdout.__contains__("loaded via"))

    return TestResult(
        name="test_load_bitstream_local",
        passed=passed,
        message="Bitstream Loaded from local directory using `$(pwd)`",
        returncode=result.returncode,
        stdout=result.stdout,
        stderr=result.stderr,
    )


def test_load_bitstream_lib_firmware() -> TestResult:
    test_file_paths = TestData(
        source=Path("./fpgad/k26-starter-kits/k26_starter_kits.bit.bin"),
        target=Path("/lib/firmware/k26-starter-kits.bit.bin"),
    )
    result = TestResult(
        name=test_load_bitstream_lib_firmware,
        passed=True,  # assume success
        message="Bitstream loaded from `/lib/firmware` directory",
        returncode=None,
        stdout=None,
        stderr=None,
    )
    if copy_test_data_files(test_file_paths) != 0:
        result.passed = False
        result.message = "Copy test data failed"
        return result
    proc = run_fpgad(["load", "bitstream", test_file_paths.target])
    result.returncode = proc.returncode
    result.stdout = proc.stdout
    result.stderr = proc.stderr

    if proc.returncode != 0 or "loaded via" not in proc.stdout:
        result.passed = False
        result.message = "Bitstream load failed"

    if cleanup_test_data_files(test_file_paths) != 0:
        result.passed = False
        result.message = "Cleanup test data failed"

    return result


def test_load_bitstream_lib_firmware_xilinx() -> TestResult:
    test_file_paths = TestData(
        source=Path("./fpgad/k26-starter-kits/k26_starter_kits.bit.bin"),
        target=Path("/lib/firmware/xilinx/k26_starter_kits/k26-starter-kits.bit.bin"),
    )
    result = TestResult(
        name=test_load_bitstream_lib_firmware,
        passed=True,  # assume success
        message="Bitstream loaded from `/lib/firmware/xilinx/k26_starter_kits/` directory",
        returncode=None,
        stdout=None,
        stderr=None,
    )
    if copy_test_data_files(test_file_paths) != 0:
        result.passed = False
        result.message = "Copy test data failed"
        return result
    proc = run_fpgad(["load", "bitstream", test_file_paths.target])
    result.returncode = proc.returncode
    result.stdout = proc.stdout
    result.stderr = proc.stderr

    if proc.returncode != 0 or "loaded via" not in proc.stdout:
        result.passed = False
        result.message = "Bitstream load failed"

    if cleanup_test_data_files(test_file_paths) != 0:
        result.passed = False
        result.message = "Cleanup test data failed"

    return result


def test_load_bitstream_path_not_exist() -> TestResult:
    result = TestResult(
        name="test_load_bitstream_local",
        passed=True,  # assume passed
        message="Bitstream Loaded from local directory using `./`",
        returncode=None,
        stdout=None,
        stderr=None,
    )
    proc = run_fpgad(["load", "bitstream", "/this/path/is/fake.bit.bin"])
    result.returncode = proc.returncode
    result.stdout = proc.stdout
    result.stderr = proc.stderr
    if (proc.returncode == 1) and (proc.stdout.__contains__("FpgadError::IOWrite:")):
        result.passed = True
    else:
        result.passed = False
        result.message = "Bitstream was not supposed to load but seems to have."
    return result


def test_load_bitstream_containing_dir() -> TestResult:
    result = TestResult(
        name="test_load_bitstream_local",
        passed=True,  # assume passed
        message="Bitstream Loaded from local directory using `./`",
        returncode=None,
        stdout=None,
        stderr=None,
    )
    proc = run_fpgad(
        ["load", "bitstream", "$(pwd)/fpgad/k26-starter-kits/k26_starter_kits.bit.bin"]
    )
    result.returncode = proc.returncode
    result.stdout = proc.stdout
    result.stderr = proc.stderr
    if (proc.returncode == 1) and (proc.stdout.__contains__("FpgadError::IOWrite:")):
        result.passed = True
    else:
        result.passed = False
        result.message = "Bitstream was not supposed to load but seems to have."
    return result


TESTS: List[Callable] = [
    test_load_bitstream_local,
    test_load_bitstream_home_fullpath,
    test_load_bitstream_lib_firmware,
    test_load_bitstream_lib_firmware_xilinx,
    test_load_bitstream_path_not_exist,
    test_load_bitstream_containing_dir,
]


def run_tests():
    failures = 0
    skipped = 0
    results = []

    print(f"{Colors.CYAN}Starting FPGA Snap Tests...{Colors.RESET}")

    for idx, test in enumerate(TESTS, start=1):
        # Auto-detect function name as label
        test_label = getattr(test, "__name__", f"Test#{idx}")
        print(f"\n=== Test #{idx}: {test_label} ===")

        # Run the test
        try:
            result: TestResult = test()
        except Exception as e:
            failures += 1
            print(
                f"{Colors.RED}[ERROR]{Colors.RESET} {test_label}: Exception occurred: {e}"
            )
            results.append((test_label, "ERROR"))
            continue

        # Determine pass/fail
        if result.passed:
            print(f"{Colors.GREEN}[PASS]{Colors.RESET} {test_label}")
            results.append((test_label, "PASS"))
        else:
            failures += 1
            print(f"{Colors.RED}[FAIL]{Colors.RESET} {test_label}: {result.message}")
            if result.stdout:
                print(f"  STDOUT: {result.stdout.strip()}")
            if result.stderr:
                print(f"  STDERR: {result.stderr.strip()}")
            results.append((test_label, "FAIL"))

    # Summary
    print("\n" + "=" * 50)
    print(f"{Colors.CYAN}Test Summary:{Colors.RESET}")
    for label, status in results:
        color = (
            Colors.GREEN
            if status == "PASS"
            else Colors.RED
            if status == "FAIL" or status == "ERROR"
            else Colors.YELLOW
        )
        print(f"{color}{status:<6}{Colors.RESET} : {label}")

    print("=" * 50)
    print(
        f"Total: {len(results)}, Passed: {len([r for r in results if r[1] == 'PASS'])}, "
        f"Failed: {failures}, Skipped: {skipped}"
    )

    # Exit code
    sys.exit(1 if failures else 0)


if __name__ == "__main__":
    run_tests()
