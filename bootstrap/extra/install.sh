#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")"/..
cargo build
sudo cp homesec-bootstrap.service /etc/systemd/system/homesec-bootstrap.service
sudo systemctl start homesec-bootstrap
