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
- [🖼️ Preview](#️-preview)
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

Key,Action,Availability
`F2`,Toggle Settings Menu (Show/Hide),✅ Always Active
`Z`,Cycle Through Colors,🔒 Menu Visible Only
`C`,Cycle Through Sizes,🔒 Menu Visible Only
`X`,Cycle Through Shapes,🔒 Menu Visible Only
`F3`,Exit Program,🔒 Menu Visible Only
