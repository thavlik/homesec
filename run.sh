#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")"/test
DEVICE_COUNT=2 \
    ADDRESS_0=$PI_HOST0 \
    ADDRESS_1=$PI_HOST1 \
    cargo run
