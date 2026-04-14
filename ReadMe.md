# NeoCross by T-GOW

A minimalist, customizable crosshair overlay for Windows. Perfect for games that lack a built-in crosshair or when you want a consistent aiming point across multiple titles.

![NeoCross Preview](https://img.shields.io/badge/Platform-Windows%2010%20%7C%20Windows%2011-blue)
![License](https://img.shields.io/badge/License-Custom-red)
![Version](https://img.shields.io/badge/Version-1.0.0-green)
![RAM](https://img.shields.io/badge/RAM-17MB-brightgreen)

---

## Features

- **7 Crosshair Shapes**: Plus, Dot, Circle, Circle+Dot, Cross with Gap, X-Shape, Triangle
- **8 Color Options**: Neon Green, Red, Blue, Gold, Magenta, White, Orange, Cyan
- **6 Size Presets**: XS to XXL for every preference and resolution
- **Always-On-Top Overlay**: Stays visible above all windows and games
- **Click-Through Design**: Does not interfere with mouse clicks or gameplay
- **Ultra Lightweight**: Uses only **17 MB RAM** — runs silently in the background

---

## System Requirements

- **Operating System**: Windows 10 or Windows 11
- **Architecture**: 64-bit (x64) or 32-bit (x86)
- **Dependencies**: None (standalone executable)

---

## Download

### Latest Release - Version 1.0.0

[![Download NeoCross](https://img.shields.io/badge/Download-NeoCross%20Setup.exe-00FF88?style=for-the-badge&logo=windows&logoColor=white)](https://github.com/T-GOW/NeoCross/releases/download/v1.0.0/NeoCross_Setup.exe)

> **Note**: Works on both 64-bit and 32-bit Windows systems.

---

## Installation

### Using the Installer (Recommended)

1. Download **NeoCross_Setup.exe** from the link above
2. Run the installer
3. Follow the on-screen instructions
4. Launch NeoCross from your Start Menu or Desktop shortcut

### Portable (No Installation)

1. Download the standalone **NeoCross.exe** from the release assets
2. Place it anywhere on your system
3. Double-click to run

---

## Crosshair Shapes

### Available Crosshair Shapes

<table>
  <tr>
    <td align="center"><b>Plus</b><br><img src="https://i.ibb.co/hFVWWzg4/crosshair-1.png" width="120"><br><i>Classic cross</i></td>
    <td align="center"><b>Dot</b><br><img src="https://i.ibb.co/xtdC48H1/crosshair-2.png" width="120"><br><i>Minimal center point</i></td>
    <td align="center"><b>Circle</b><br><img src="https://i.ibb.co/zVrncJng/crosshair-3.png" width="120"><br><i>Clean ring</i></td>
  </tr>
  <tr>
    <td align="center"><b>Circle + Dot</b><br><img src="https://i.ibb.co/VYGyPdPm/crosshair-4.png" width="120"><br><i>Precision hybrid</i></td>
    <td align="center"><b>Cross Gap</b><br><img src="https://i.ibb.co/twZNJMhd/crosshair-5.png" width="120"><br><i>Separated lines</i></td>
    <td align="center"><b>X-Shape</b><br><img src="https://i.ibb.co/PswPFgbp/crosshair-6.png" width="120"><br><i>Diagonal cross</i></td>
  </tr>
  <tr>
    <td align="center"><b>Triangle</b><br><img src="https://i.ibb.co/hFVWWzg4/crosshair-1.png" width="120"><br><i>Angled precision</i></td>
    <td></td>
    <td></td>
  </tr>
</table>

---

## Controls

| Key | Action |
|-----|--------|
| **F2** | Toggle menu visibility (always active) |
| **Z** | Cycle through colors (when menu is visible) |
| **C** | Cycle through sizes (when menu is visible) |
| **X** | Cycle through shapes (when menu is visible) |
| **F3** | Exit application (when menu is visible) |

---

## Building from Source

### Prerequisites

- Python 3.8 or higher
- PyQt5

### Setup

```bash
pip install PyQt5
python NeoCross.py
```

### Creating a Standalone Executable

Using [auto-py-to-exe](https://github.com/brentvollebregt/auto-py-to-exe):

```bash
pip install auto-py-to-exe
auto-py-to-exe
```

Configuration:
- **Script Location**: `NeoCross.py`
- **Onefile**: Onefile
- **Console Window**: Console Based
- **Icon**: (optional) Add your own `.ico` file

---

## License

NeoCross is provided free of charge for personal use. You may modify the source code for personal use, but redistribution and commercial use are prohibited. See the [LICENSE](LICENSE) file for details.

---

## Credits

**Developer**: T-GOW  
**Project**: NeoCross

---

## Support

For issues, feature requests, or questions, please open an issue on the [GitHub Issues page](https://github.com/T-GOW/NeoCross/issues).

---

*NeoCross - Precision at your fingertips.*
