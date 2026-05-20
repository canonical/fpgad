#!/usr/bin/env bash

# Set to default values. Will be set in snap_integration_tests.yml
export SNAP_TEST_SOURCE="${SNAP_TEST_SOURCE:-local}"
export SNAP_CHANNEL="${SNAP_CHANNEL:-${SNAP_TEST_SOURCE}}"