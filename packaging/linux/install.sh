#!/bin/sh
# Installs NeoCross for the current user (no root needed): ~/.local/bin, menu entry and icon.
# Remove it again with: ./install.sh --uninstall
set -eu
here=$(cd "$(dirname "$0")" && pwd)
bin="${HOME}/.local/bin"
apps="${XDG_DATA_HOME:-$HOME/.local/share}/applications"
icons="${XDG_DATA_HOME:-$HOME/.local/share}/icons/hicolor"

if [ "${1:-}" = "--uninstall" ]; then
    rm -f "$bin/neocross" "$apps/neocross.desktop" "$icons/512x512/apps/neocross.png" "$icons/128x128/apps/neocross.png"
    rm -f "${XDG_CONFIG_HOME:-$HOME/.config}/autostart/neocross.desktop"
    echo "NeoCross removed. Your settings stay in ${XDG_CONFIG_HOME:-$HOME/.config}/neocross."
    exit 0
fi

mkdir -p "$bin" "$apps" "$icons/512x512/apps" "$icons/128x128/apps"
install -m 755 "$here/neocross" "$bin/neocross"
sed "s|^Exec=neocross|Exec=$bin/neocross|" "$here/neocross.desktop" > "$apps/neocross.desktop"
install -m 644 "$here/neocross-512.png" "$icons/512x512/apps/neocross.png"
install -m 644 "$here/neocross-128.png" "$icons/128x128/apps/neocross.png"
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$apps" || true
fi
echo "NeoCross installed to $bin/neocross. Find it in your app menu."
