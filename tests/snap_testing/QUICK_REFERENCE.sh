#!/bin/bash
#
# This file is part of fpgad, an application to manage FPGA subsystem together with device-tree and kernel modules.
#
# Copyright 2026 Canonical Ltd.
#
# SPDX-License-Identifier: GPL-3.0-only
#
# fpgad is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License version 3, as published by the Free Software Foundation.
#
# fpgad is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranties of MERCHANTABILITY, SATISFACTORY QUALITY, or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License along with this program.  If not, see http://www.gnu.org/licenses/.
#

# Quick reference for running FPGA tests

cat << 'EOF'
FPGA Test Runner - Quick Reference
====================================

Run All Tests:
  cd tests && ./run_all.sh
  python3 -m unittest discover -s tests -p "test_*.py" -v

Run Platform-Specific Tests:
  ./run_all.sh universal      # Universal platform tests only
  ./run_all.sh xlnx          # Xilinx platform tests only
  ./run_all.sh default       # Help & CLI option tests

Run Individual Test Files:
  python3 -m unittest tests.test_universal.test_bitstream -v
  python3 -m unittest tests.test_xlnx.test_overlay -v
  python3 -m unittest tests.test_default.test_help -v

Run Single Test:
  python3 -m unittest tests.test_universal.test_bitstream.TestBitstreamUniversal.test_load_bitstream_local -v

Directory Structure:
  tests/
  ├── common/              # Shared base classes & utilities
  ├── test_universal/      # Tests with --platform universal
  ├── test_xlnx/          # Tests with --platform xlnx
  └── test_default/       # Tests without platform flag

Legacy (Deprecated):
  python3 snap_tests.py   # Old monolithic test file

Requirements:
  - Must run as root (tests interact with sysfs)
  - fpgad snap must be installed
  - FPGA hardware required (Xilinx Kria or compatible)

EOF

