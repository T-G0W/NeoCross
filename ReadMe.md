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
      <img src="[https://imgur.com/9NHYn5o.gif](https://imgur.com/tgow-crosshair-preview-5XnKL4t)" width="100%" alt="TGOW Crosshair Menu Demo">
    </td>
    <td align="center">
      <b>In-Game Usage</b><br>
      <img src="[https://imgur.com/KPiHZHn.gif](https://imgur.com/tgow-crosshair-KVmL4pt)" width="100%" alt="TGOW Crosshair In-Game Demo">
    </td>
  </tr>
</table>

> 💡 *Tip: If the GIFs don't display, replace the Imgur links above with direct `.gif` URLs (remove `/a/` from the path).*

### 🎯 Available Crosshair Shapes

<table>
  <tr>
    <td align="center"><b>Plus</b><br><img src="[https://cdn.discordapp.com/attachments/1491214256711008396/1491217838038581299/1.png](https://media.discordapp.net/attachments/1491214256711008396/1491217838038581299/1.png?ex=69d6e456&is=69d592d6&hm=2b59ba77eb8c36d2294761aa07d0cec0cf1fff8d4ec55d66348780b61e8c7e10&=&format=webp&quality=lossless)" width="120"><br><i>Classic cross</i></td>
    <td align="center"><b>Dot</b><br><img src="[https://cdn.discordapp.com/attachments/1491214256711008396/1491217837627412490/2.png](https://media.discordapp.net/attachments/1491214256711008396/1491217837627412490/2.png?ex=69d6e455&is=69d592d5&hm=83195e8dc2de6984b06ccad176f0b3914f890429e1c9469735cf26ed803731d5&=&format=webp&quality=lossless)" width="120"><br><i>Minimal center point</i></td>
    <td align="center"><b>Circle</b><br><img src="[https://cdn.discordapp.com/attachments/1491214256711008396/1491217836994199663/3.png](https://media.discordapp.net/attachments/1491214256711008396/1491217836994199663/3.png?ex=69d6e455&is=69d592d5&hm=acca7d4cfb96df142ebda4be34da4c65d2024428380ce74dc4d67eb753dc358a&=&format=webp&quality=lossless)" width="120"><br><i>Clean ring</i></td>
  </tr>
  <tr>
    <td align="center"><b>Circle + Dot</b><br><img src="[https://cdn.discordapp.com/attachments/1491214256711008396/1491217836314464326/4.png](https://media.discordapp.net/attachments/1491214256711008396/1491217836314464326/4.png?ex=69d6e455&is=69d592d5&hm=8d75fa07bda2debbbb337e2c8df505260d66e2cfb46673da101e5dbe97a9c457&=&format=webp&quality=lossless)" width="120"><br><i>Precision hybrid</i></td>
    <td align="center"><b>Cross Gap</b><br><img src="[https://cdn.discordapp.com/attachments/1491214256711008396/1491217836016795821/5.png](https://media.discordapp.net/attachments/1491214256711008396/1491217836016795821/5.png?ex=69d6e455&is=69d592d5&hm=621c7647073546899fcd837ba3ceefdd3332743a1d7b2dcabf92f1595577452d&=&format=webp&quality=lossless)" width="120"><br><i>Separated lines</i></td>
    <td align="center"><b>X-Shape</b><br><img src="[https://cdn.discordapp.com/attachments/1491214256711008396/1491217835656220795/6.png](https://cdn.discordapp.com/attachments/1491214256711008396/1491217835656220795/6.png?ex=69d6e455&is=69d592d5&hm=522d21a5658464dfdf6ddd7fe1d690353401b99b561c6ae8b0e0eb907dc92a6a&)" width="120"><br><i>Diagonal cross</i></td>
  </tr>
  <tr>
    <td colspan="3" align="center"><b>Triangle</b><br><img src="[https://cdn.discordapp.com/attachments/1491214256711008396/1491217835085533314/7.png](https://cdn.discordapp.com/attachments/1491214256711008396/1491217835085533314/7.png?ex=69d6e455&is=69d592d5&hm=cdca410e72bfccbc67920556c3e7939fd84debb224ea20240d956209d7b22295&)" width="120"><br><i>Forward-pointing arrow</i></td>
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
