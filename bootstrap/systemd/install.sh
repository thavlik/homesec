#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")"/..
cargo build
