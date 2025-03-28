#!/bin/bash
read -p "Current version: " version
rm -rf ~/.tmp/wayland-clipboard-manager-release/
mkdir -p ~/.tmp/wayland-clipboard-manager-release/
cargo build --release
cp target/release/wayland-clipboard-manager ~/.tmp/wayland-clipboard-manager-release/
cp LICENSE ~/.tmp/wayland-clipboard-manager-release/LICENSE

cd ~/.tmp/wayland-clipboard-manager-release/
tar -czf wayland-clipboard-manager-v${version}-bin-linux-x86_64.tar.gz wayland-clipboard-manager LICENSE


