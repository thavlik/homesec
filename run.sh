#!/bin/bash
set -euo pipefail
docker run --network=host rancher/k3s:v1.17.3-k3s1 server --tls-san 0.0.0.0

# --node-label camera=yes
