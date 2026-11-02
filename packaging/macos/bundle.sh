#!/bin/sh
# Builds NeoCross.app (Intel + Apple silicon) and NeoCross-<version>-macos.dmg into dist/release.
set -eu
cd "$(dirname "$0")/../.."
version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n1)
out=dist/release
app="$out/NeoCross.app"
mkdir -p "$out"

for t in x86_64-apple-darwin aarch64-apple-darwin; do
    rustup target add "$t" >/dev/null
    cargo build --release --locked --target "$t"
done
cargo run --release --locked --manifest-path tools/licenses/Cargo.toml -- \
    --target aarch64-apple-darwin --out target/THIRD_PARTY_LICENSES.txt

rm -rf "$app"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
lipo -create -output "$app/Contents/MacOS/neocross" \
    target/x86_64-apple-darwin/release/neocross target/aarch64-apple-darwin/release/neocross
sed "s/@VERSION@/$version/g" packaging/macos/Info.plist > "$app/Contents/Info.plist"
cp assets/icon.icns "$app/Contents/Resources/icon.icns"
cp LICENSE-MIT LICENSE-APACHE "$app/Contents/Resources/"
cp target/THIRD_PARTY_LICENSES.txt "$app/Contents/Resources/"
# Ad-hoc signature: required on Apple silicon, no developer account involved.
codesign --force --deep --sign - "$app"

stage=$(mktemp -d)
cp -R "$app" "$stage/"
ln -s /Applications "$stage/Applications"
hdiutil create -volname "NeoCross $version" -srcfolder "$stage" -ov -format UDZO "$out/NeoCross-$version-macos.dmg"
rm -rf "$stage"
echo "Built $out/NeoCross-$version-macos.dmg"
