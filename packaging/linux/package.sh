#!/bin/sh
# Builds the Linux release files into dist/release: .deb, .rpm and a portable .tar.gz.
# Needs: cargo-deb and cargo-generate-rpm (cargo install cargo-deb cargo-generate-rpm).
set -eu
cd "$(dirname "$0")/../.."
version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n1)
out=dist/release
mkdir -p "$out"

cargo build --release --locked
cargo run --release --locked --manifest-path tools/licenses/Cargo.toml -- \
    --target x86_64-unknown-linux-gnu --out target/THIRD_PARTY_LICENSES.txt

cargo deb --no-build --output "$out/"
cargo generate-rpm --output "$out/"

name="neocross-$version-linux-x86_64"
stage="dist/$name"
rm -rf "$stage"
mkdir -p "$stage"
cp target/release/neocross packaging/linux/neocross.desktop packaging/linux/install.sh LICENSE-MIT LICENSE-APACHE \
    target/THIRD_PARTY_LICENSES.txt "$stage/"
cp assets/logo-512.png "$stage/neocross-512.png"
cp assets/logo-128.png "$stage/neocross-128.png"
tar -C dist -czf "$out/NeoCross-$version-linux-x86_64.tar.gz" "$name"
echo "Built Linux packages in $out"
