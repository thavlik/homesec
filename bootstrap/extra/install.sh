#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")"
pushd ..
    cargo build
popd
sudo cp ../../target/debug/homesec-bootstrap /usr/bin/homesec-bootstrap
sudo cp ./homesec-bootstrap.service /etc/systemd/system/homesec-bootstrap.service
sudo systemctl start homesec-bootstrap
