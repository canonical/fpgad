# FPGA Snap Tests

This directory contains integration tests for the fpgad snap package.

## Directory Structure

```
tests/
├── common/                     # Shared utilities and base classes
│   ├── base_test.py           # Base test class with setUp/tearDown
│   └── helpers.py             # Colors, TestData, utility functions
├── test_universal/            # Tests with --platform universal
│   ├── test_bitstream.py      # Bitstream loading tests
│   ├── test_overlay.py        # Overlay loading tests
│   ├── test_status.py         # Status command tests
│   └── test_set.py            # Set command tests
├── test_xlnx/                 # Tests with --platform xlnx
│   ├── test_bitstream.py      # Bitstream loading tests
│   ├── test_overlay.py        # Overlay loading tests
│   ├── test_status.py         # Status command tests
│   └── test_set.py            # Set command tests
├── test_default/              # Tests without platform flag
│   ├── test_help.py           # Help command tests
│   └── test_cli_options.py    # CLI option tests (--device, --name, etc.)
├── run_all.sh                 # Convenience script to run tests
└── snap_tests.py              # Legacy monolithic test file (deprecated)
```

## Running Tests

### Running Snap Tests (Production)

When testing the installed snap package, use the `fpgad.test` command:

```bash
# Install the snap with test component
sudo snap install fpgad+test --dangerous ./fpgad_*.snap

# Connect required interfaces (for daemon and tests)
sudo snap connect fpgad:fpga
sudo snap connect fpgad:kernel-firmware-control
sudo snap connect fpgad:hardware-observe
sudo snap connect fpgad:device-tree-overlays
sudo snap connect fpgad:run-dfx-mgrd-socket
sudo snap connect fpgad:cli-dbus fpgad:daemon-dbus

# Run all tests (MUST use sudo)
sudo fpgad.test

# Run specific test suite
sudo fpgad.test universal  # Universal platform tests only
sudo fpgad.test xlnx       # Xilinx dfx-mgr tests only
sudo fpgad.test default    # Default (no platform) tests only
```

**Important Notes:**
- Tests **MUST** be run with `sudo` as they need to write to sysfs
- All snap interfaces must be connected before running tests
- The test component must be installed (`fpgad+test`)
- **Ubuntu Core Limitation**: Tests will NOT work on Ubuntu Core. The `system-files` interface required for sysfs access (`/sys/class/fpga_manager/*` and `/sys/kernel/config/device-tree/overlays`) is not auto-connected on Ubuntu Core and cannot be manually connected on a strictly confined system. Tests must be run on Ubuntu Server or Desktop.

### Running Tests During Development

For development/debugging without building the snap:

```bash
# From tests directory
python3 -m unittest discover -s . -p "test_*.py" -v

# Or use the convenience script
./run_all.sh
```

### Run Platform-Specific Tests

```bash
# Universal platform only
python3 -m unittest discover -s test_universal -p "test_*.py" -v
./run_all.sh universal

# Xilinx platform only
python3 -m unittest discover -s test_xlnx -p "test_*.py" -v
./run_all.sh xlnx

# Default (no platform flag) tests only
python3 -m unittest discover -s test_default -p "test_*.py" -v
./run_all.sh default
```

### Run Specific Test File

```bash
python3 -m unittest tests.test_universal.test_bitstream
python3 -m unittest tests.test_xlnx.test_overlay
python3 -m unittest tests.test_default.test_help
```

### Run Single Test

```bash
python3 -m unittest tests.test_universal.test_bitstream.TestBitstreamUniversal.test_load_bitstream_local
```

## Requirements

### For Snap Tests
- **Root/sudo access** (tests interact with sysfs and FPGA hardware)
- FPGA hardware (Xilinx Kria or compatible)
- fpgad snap installed with test component
- **All snap interfaces connected** (provides necessary permissions)
- Test data files in snap provider content location
- **Ubuntu Server or Desktop** (tests will not work on Ubuntu Core due to system-files interface restrictions)

### For Development Tests (without snap)
- **Root/sudo access** (direct sysfs interaction requires privileges)
- FPGA hardware (Xilinx Kria or compatible)
- fpgad daemon running
- Test data files in `daemon/tests/test_data/`
- Test data files copied to `fpgad/k26-starter-kits/` and `fpgad/k24-starter-kits/` (relative to test working directory)

## Test Structure

All test classes inherit from `FPGATestBase` which provides:
- Automatic cleanup of overlays before/after tests
- Reset of FPGA flags
- Common assertion helpers for process results
- Utility methods for FPGA operations

### Test Data Setup

The tests require bitstream (`.bit.bin`) and device tree overlay (`.dtbo`) files to be present in specific locations:

**Source files** (repository):
- `daemon/tests/test_data/k26-starter-kits/` - Contains test files for Kria K26
- `daemon/tests/test_data/k24-starter-kits/` - Contains test files for Kria KD240

**Expected locations** (at test runtime):
- `fpgad/k26-starter-kits/` - Relative to the test working directory
- `fpgad/k24-starter-kits/` - Relative to the test working directory

The `local_snap_tests.sh` script automatically copies files from the source locations to the expected locations during test setup. If running tests manually, you must ensure these files are copied to the correct locations.

## Migration from Legacy Tests

The old `snap_tests.py` file is deprecated but kept for reference. All tests have been:
1. Split by platform (universal, xlnx, default)
2. Organized by functionality (bitstream, overlay, status, set, help, cli_options)
3. Refactored to use shared base classes and utilities

To run the legacy tests:
```bash
python3 snap_tests.py
```

## CI/CD Integration

The GitHub Actions workflow automatically:
1. Builds the snap
2. Deploys to test hardware via Testflinger
3. Runs all tests using unittest discovery
4. Collects and uploads test artifacts

## Adding New Tests

1. Determine which platform directory (test_universal, test_xlnx, or test_default)
2. Add test methods to existing test classes or create new test file
3. Inherit from `FPGATestBase`
4. Use provided assertion helpers
5. Follow existing naming conventions

Example:
```python
from common.base_test import FPGATestBase

class TestNewFeature(FPGATestBase):
    PLATFORM = "universal"  # or "xlnx", or omit for default
    
    def test_something(self):
        proc = self.run_fpgad(["--platform", self.PLATFORM, "command"])
        self.assert_proc_succeeds(proc)
```

**Note:** Import paths use `from common.*` when tests are in the snap_testing directory.

