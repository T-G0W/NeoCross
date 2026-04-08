# TGOW - CrosshairV1

<div align="center">

![Version](https://img.shields.io/badge/version-1.0.0-blue.svg)
![Platform](https://img.shields.io/badge/Windows-10%20%7C%2011-lightgrey.svg)
![RAM](https://img.shields.io/badge/RAM-17MB-green.svg)
![CPU](https://img.shields.io/badge/CPU-0%25-brightgreen.svg)
[![Discord](https://img.shields.io/badge/Discord-t_gow-5865F2.svg?logo=discord&logoColor=white)](https://discord.com)
[![Buy Me a Coffee](https://img.shields.io/badge/Buy%20Me%20a-Coffee-FFDD00.svg?logo=buy-me-a-coffee&logoColor=black)](https://www.buymeacoffee.com/t-gow)

**A minimalist, performance-optimized crosshair overlay for Windows**

</div>

> 📝 **Note:** The first preview below is recorded in German for demonstration purposes. The actual downloaded software features a complete **English interface**.

---

## 📋 Table of Contents
- [✨ Features](#-features)
- [💻 System Requirements & Resource Usage](#-system-requirements--resource-usage)
- [📥 Download & Installation](#-download--installation)
- [🎮 Usage & Controls](#-usage--controls)
- [🖼️ Preview](#-preview)
- [📞 Support & Contact](#-support--contact)
- [☕ Support Development](#-support-development)
- [📜 License](#-license)

---

## ✨ Features
- 🎯 **7 Unique Crosshair Shapes** – Plus, Dot, Circle, Circle+Dot, Cross Gap, X-Shape, Triangle
- 🎨 **8 Vibrant Colors** – Neon Green, Red, Blue, Gold, Magenta, White, Orange, Cyan
- 📏 **6 Size Presets** – From XS (5px) to XXL (30px)
- ⚡ **Ultra-Lightweight** – Runs silently with near-zero system impact
- 🎮 **Game-Friendly** – Zero input blocking when the menu is hidden
- 🖱️ **Click-Through** – Overlay never interferes with mouse events
- 🔧 **Instant Controls** – Keyboard-driven menu with zero lag
- 📍 **Auto-Recenter** – Stays perfectly centered across resolution changes

---

## 💻 System Requirements & Resource Usage

| Requirement | Specification |
|-------------|---------------|
| **Operating System** | Windows 10 / Windows 11 |
| **Python Version** | Python 3.8+ (for source builds) |
| **Dependencies** | `PyQt5` |

| Resource | Usage |
|----------|-------|
| **RAM** | 17 MB |
| **CPU** | 0% |
| **Disk** | 0% |
| **Network** | 0% |

---

## 📥 Download & Installation

### 📦 Pre-Built (Recommended)
1. Download the latest release from the [Releases](https://github.com/T-G0W/tgow-crosshair-v1/releases) page.
2. Run `TGOW-CrosshairV1.exe`.
3. The crosshair appears instantly in the center of your primary monitor.

### 🐍 From Source
```bash
git clone https://github.com/T-G0W/tgow-crosshair-v1.git
cd tgow-crosshair-v1
pip install PyQt5
python crosshair.py
```

---

## 🎮 Usage & Controls

| Key | Action | Availability |
|:---:|--------|--------------|
| **`F2`** | Toggle Settings Menu (Show/Hide) | ✅ Always Active |
| **`Z`** | Cycle Through Colors | 🔒 Menu Visible Only |
| **`C`** | Cycle Through Sizes | 🔒 Menu Visible Only |
| **`X`** | Cycle Through Shapes | 🔒 Menu Visible Only |
| **`F3`** | Exit Program | 🔒 Menu Visible Only |

### 💡 How It Works
1. Launch the application → Crosshair appears centered.
2. Press **`F2`** to open the settings panel (top-right corner).
3. Customize your crosshair via buttons or hotkeys.
4. Press **`F2`** again to hide the menu and return to your game.
5. While hidden, **all hotkeys (`Z`, `C`, `X`, `F3`) are fully disabled**, ensuring zero interference with gameplay. Only `F2` remains active to reopen the menu.

---

## 🖼️ Preview

### 📹 Live Demos

<table>
<tr>
  <td align="center">
    <b>Menu Interface & Controls</b><br>
      <img src="https://i.ibb.co/zWV8w5Cm/crosshair-menu.gif" width="100%" alt="TGOW Crosshair Menu Demo">
    </td>
    <td align="center">
      <b>In-Game Usage</b><br>
    <img src="https://i.ibb.co/mCQCGk9s/crosshair-game.gif" width="100%" alt="TGOW Crosshair In-Game Demo">
    </td>
  </tr>
</table>

> 💡 *Tip: If the GIFs don't display, replace the Imgur links above with direct `.gif` URLs (remove `/a/` from the path).*

### 🎯 Available Crosshair Shapes

<table>
  <tr>
    <td align="center"><b>Plus</b><br><img src="https://i.ibb.co/hFVWWzg4/crosshair-1.png" width="120"><br><i>Classic cross</i></td>
    <td align="center"><b>Dot</b><br><img src="https://i.ibb.co/xtdC48H1/crosshair-2.png" width="120"><br><i>Minimal center point</i></td>
    <td align="center"><b>Circle</b><br><img src="https://i.ibb.co/zVrncJng/crosshair-3.png" width="120"><br><i>Clean ring</i></td>
    </tr>
    <tr>
    <td align="center"><b>Circle + Dot</b><br><img src="https://i.ibb.co/VYGyPdPm/crosshair-4.png" width="120"><br><i>Precision hybrid</i></td>
    <td align="center"><b>Cross Gap</b><br><img src="https://i.ibb.co/twZNJMhd/crosshair-5.png" width="120"><br><i>Separated lines</i></td>
    <td align="center"><b>X-Shape</b><br><img src="https://i.ibb.co/twZNJMhd/crosshair-6.png" width="120"><br><i>Diagonal cross</i></td>
    </tr>
    <tr>
    <td colspan="3" align="center"><b>Triangle</b><br><img src="https://i.ibb.co/PswPFgbp/crosshair-7.png" width="120"><br><i>Forward-pointing arrow</i></td>
  </tr>
</table>

---

## 📞 Support & Contact

Found a bug, have a suggestion, or need help? Reach out directly:

- 🟣 **Discord:** `t_gow`
- 🐙 **GitHub Issues:** [Open an Issue](https://github.com/T-G0W/tgow-crosshairv1/issues)

I actively monitor both channels and appreciate constructive feedback to improve the tool.

---

## ☕ Support Development

If TGOW - CrosshairV1 improves your gameplay or workflow, consider supporting the project:

<div align="center">
  <a href="https://www.buymeacoffee.com/tgowscripts" target="_blank">
    <img src="https://cdn.buymeacoffee.com/buttons/v2/default-yellow.png" alt="Buy Me A Coffee" width="217">
  </a>
</div>

Every contribution goes directly into maintaining and expanding free tools for the community. ❤️

---

## 📜 License

This software is **proprietary** and protected under a custom license.

✅ **Allowed:** Private & commercial use (unmodified)  
❌ **Restricted:** Modification, redistribution, sublicensing, or integration into other projects **requires prior written permission** from the author.

📄 View the full legal terms in the [`LICENSE`](LICENSE) file.  
📬 Permission requests: Contact `t_gow` on Discord or open a GitHub Issue.

---

<div align="center">
  <sub>Made with ❤️ by <strong>T-GOW</strong></sub><br>
  <a href="#tgow---crosshairv1">⬆ Back to Top</a>
</div>
