#!/usr/bin/env bash
set -euo pipefail

TARGET=x86_64-unknown-linux-gnu
PLUGIN_DIR=~/.config/opendeck/plugins/com.garrettfaucher.openweather.sdPlugin

cargo build --release
rm -rf "$PLUGIN_DIR"
mkdir -p "$PLUGIN_DIR"
cp -r assets/. "$PLUGIN_DIR/"
cp "target/release/oaopenweather" "$PLUGIN_DIR/oaopenweather-$TARGET"

pkill -x opendeck || true
sleep 0.5
nohup opendeck > /tmp/opendeck.log 2>&1 &
echo "Deployed and restarted OpenDeck."
