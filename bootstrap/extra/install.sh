#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")"
pushd ..
    cargo build
    echo "built rust binaries"
popd
echo "copying binaries to target directories"
sudo cp ../../target/debug/homesec_bootstrap /usr/bin/homesec_bootstrap
sudo cp ./homesec-bootstrap.service /etc/systemd/system/homesec-bootstrap.service
sudo systemctl start homesec-bootstrap
echo "installation complete"
