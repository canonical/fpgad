#!/usr/bin/env bash
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

# Check if the tests component is installed
if [ ! -d "$SNAP_COMPONENTS/test" ]; then
    echo "Tests component not installed. Install with \"snap install fpgad+test.comp\""
    exit 1
fi


# Add snap_testing to PYTHONPATH so imports work
export PYTHONPATH="$SNAP_COMPONENTS/test:$PYTHONPATH"

# Debug output
echo "SNAP_COMPONENTS: $SNAP_COMPONENTS"
if [ -d "$SNAP_COMPONENTS/dfx-mgr" ]; then
    echo "dfx-mgr component found at: $SNAP_COMPONENTS/dfx-mgr"
    if [ -f "$SNAP_COMPONENTS/dfx-mgr/usr/bin/dfx-mgr-client" ]; then
        echo "dfx-mgr-client found: $SNAP_COMPONENTS/dfx-mgr/usr/bin/dfx-mgr-client"
    else
        echo "WARNING: dfx-mgr component directory exists but dfx-mgr-client not found"
    fi
else
    echo "dfx-mgr component NOT installed (xlnx tests will be skipped)"
fi

# Run the snap tests
exec "$SNAP_COMPONENTS/test/test_snap.sh" "$@"

