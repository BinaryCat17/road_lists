#!/bin/bash
# Build road_lists locally and deploy the binary to the server.
set -euo pipefail
cd "$(dirname "$0")/.."

HOST="salskayastep"
REMOTE_DIR="/opt/road_lists"
SERVICE="road-lists"

echo "==> Building release binary locally..."
cargo build --release

echo "==> Uploading binary..."
scp target/release/road_lists "$HOST:$REMOTE_DIR/target/release/road_lists.new"

echo "==> Swapping binary and restarting $SERVICE..."
ssh "$HOST" bash -s <<EOF
set -euo pipefail
cd "$REMOTE_DIR"
mv target/release/road_lists target/release/road_lists.bak
mv target/release/road_lists.new target/release/road_lists
systemctl restart $SERVICE
sleep 1
systemctl is-active --quiet $SERVICE && echo "$SERVICE is active" || (echo "$SERVICE FAILED to start, rolling back"; mv target/release/road_lists.bak target/release/road_lists; systemctl restart $SERVICE; exit 1)
EOF

echo "==> road_lists deployed."
