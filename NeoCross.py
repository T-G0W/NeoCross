"""
NeoCross by T-GOW - Minimalist Crosshair Overlay for Windows
Controls:
    F2  = Toggle menu (always active)
    Z   = Change color (only when menu is visible)
    C   = Change size (only when menu is visible)
    X   = Change shape (only when menu is visible)
    F3  = Exit application (only when menu is visible)
"""

import sys
import os
import json
import ctypes
import ctypes.wintypes
import threading
import time
from PyQt5.QtWidgets import (
    QApplication, QWidget, QLabel, QPushButton,
    QVBoxLayout, QHBoxLayout, QFrame
)
from PyQt5.QtCore import Qt, QTimer, pyqtSignal, QObject
from PyQt5.QtGui import QPainter, QColor, QPen, QPainterPath


GWL_EXSTYLE       = -20
WS_EX_LAYERED     = 0x00080000
WS_EX_TRANSPARENT = 0x00000020

def make_click_through(hwnd):
    try:
        style = ctypes.windll.user32.GetWindowLongW(hwnd, GWL_EXSTYLE)
        ctypes.windll.user32.SetWindowLongW(
            hwnd, GWL_EXSTYLE, style | WS_EX_LAYERED | WS_EX_TRANSPARENT
        )
    except Exception as e:
        print(f"[NeoCross] Click-through error: {e}")


VK_F2 = 0x71
VK_F3 = 0x72
VK_Z  = 0x5A
VK_C  = 0x43
VK_X  = 0x58


COLORS = [
    ("#00FF88", "Neon Green"),
    ("#FF4444", "Red"),
    ("#44AAFF", "Blue"),
    ("#FFD700", "Gold"),
    ("#FF44FF", "Magenta"),
    ("#FFFFFF", "White"),
    ("#FF8800", "Orange"),
    ("#00FFFF", "Cyan"),
]

SIZES       = [5, 8, 12, 16, 22, 30]
SIZE_LABELS = ["XS", "S", "M", "L", "XL", "XXL"]

SHAPES      = ["plus", "dot", "circle", "circle_dot", "cross_gap", "x_shape", "triangle"]
SHAPE_SYMS  = ["+",    "•",   "○",      "⊙",          "⊕",         "✕",       "△"]


SETTINGS_DIR = os.path.join(os.getenv("APPDATA"), "NeoCross")
SETTINGS_FILE = os.path.join(SETTINGS_DIR, "settings.json")

DEFAULT_SHAPE = 5
DEFAULT_COLOR = 3
DEFAULT_SIZE  = 1


def load_settings():
    if os.path.exists(SETTINGS_FILE):
        try:
            with open(SETTINGS_FILE, "r") as f:
                return json.load(f)
        except Exception:
            pass
    return {"shape": DEFAULT_SHAPE, "color": DEFAULT_COLOR, "size": DEFAULT_SIZE}


def save_settings(shape_idx, color_idx, size_idx):
    try:
        os.makedirs(SETTINGS_DIR, exist_ok=True)
        data = {"shape": shape_idx, "color": color_idx, "size": size_idx}
        with open(SETTINGS_FILE, "w") as f:
            json.dump(data, f, indent=2)
    except Exception:
        pass


class HotkeySignals(QObject):
    toggle_menu = pyqtSignal()
    next_color  = pyqtSignal()
    next_size   = pyqtSignal()
    next_shape  = pyqtSignal()
    quit_app    = pyqtSignal()


class HotkeyThread(threading.Thread):
    """Polling-based hotkey thread. Does NOT block keys."""

    def __init__(self, signals: HotkeySignals):
        super().__init__(daemon=True, name="HotkeyThread")
        self.signals = signals
        self._menu_visible = True
        self._lock = threading.Lock()
        self._running = True
        self._key_states = {}

    def set_menu_visible(self, visible: bool):
        with self._lock:
            self._menu_visible = visible

    def _is_pressed(self, vk):
        return bool(ctypes.windll.user32.GetAsyncKeyState(vk) & 0x8000)

    def _check_press(self, name, vk):
        pressed = self._is_pressed(vk)
        was_pressed = self._key_states.get(name, False)
        self._key_states[name] = pressed
        return pressed and not was_pressed

    def run(self):
        while self._running:
            try:
                with self._lock:
                    visible = self._menu_visible

                if self._check_press('F2', VK_F2):
                    self.signals.toggle_menu.emit()

                if visible:
                    if self._check_press('F3', VK_F3):
                        self.signals.quit_app.emit()
                    if self._check_press('Z', VK_Z):
                        self.signals.next_color.emit()
                    if self._check_press('C', VK_C):
                        self.signals.next_size.emit()
                    if self._check_press('X', VK_X):
                        self.signals.next_shape.emit()

            except Exception:
                pass

            time.sleep(0.04)

    def stop(self):
        self._running = False



class CrosshairWidget(QWidget):

    def __init__(self):
        super().__init__()
        settings = load_settings()
        self.color_idx = settings["color"]
        self.size_idx  = settings["size"]
        self.shape_idx = settings["shape"]
        self.thickness = 2
        self._setup()
        self._center()

    def _setup(self):
        self.setWindowFlags(
            Qt.FramelessWindowHint
            | Qt.WindowStaysOnTopHint
            | Qt.Tool
            | Qt.BypassWindowManagerHint
        )
        self.setAttribute(Qt.WA_TranslucentBackground)
        self.setAttribute(Qt.WA_ShowWithoutActivating)
        self.setAttribute(Qt.WA_TransparentForMouseEvents)
        self.setFocusPolicy(Qt.NoFocus)

    def _center(self):
        s = QApplication.primaryScreen().geometry()
        sz = 200
        self.setGeometry(s.center().x()-sz//2, s.center().y()-sz//2, sz, sz)

    def recenter(self): self._center()

    @property
    def color(self): return QColor(COLORS[self.color_idx][0])
    @property
    def size(self):  return SIZES[self.size_idx]
    @property
    def shape(self): return SHAPES[self.shape_idx]

    def next_color(self): self.color_idx = (self.color_idx+1)%len(COLORS); save_settings(self.shape_idx, self.color_idx, self.size_idx); self.update()
    def next_size(self):  self.size_idx  = (self.size_idx+1)%len(SIZES);   save_settings(self.shape_idx, self.color_idx, self.size_idx); self.update()
    def next_shape(self): self.shape_idx = (self.shape_idx+1)%len(SHAPES); save_settings(self.shape_idx, self.color_idx, self.size_idx); self.update()

    def paintEvent(self, _):
        p   = QPainter(self)
        p.setRenderHint(QPainter.Antialiasing)
        cx  = self.width()  // 2
        cy  = self.height() // 2
        s   = self.size
        t   = self.thickness
        col = self.color

        shp = QPen(QColor(0,0,0,100), t+2, Qt.SolidLine, Qt.RoundCap, Qt.RoundJoin)
        mp  = QPen(col, t, Qt.SolidLine, Qt.RoundCap, Qt.RoundJoin)

        sh = self.shape
        if sh == "plus":
            for pen in (shp, mp):
                p.setPen(pen)
                p.drawLine(cx-s, cy, cx+s, cy)
                p.drawLine(cx, cy-s, cx, cy+s)

        elif sh == "dot":
            r = max(2, s//3)
            p.setPen(Qt.NoPen); p.setBrush(QColor(0,0,0,90))
            p.drawEllipse(cx-r+1, cy-r+1, r*2, r*2)
            p.setBrush(col); p.drawEllipse(cx-r, cy-r, r*2, r*2)

        elif sh == "circle":
            for pen in (shp, mp):
                p.setPen(pen); p.setBrush(Qt.NoBrush)
                p.drawEllipse(cx-s, cy-s, s*2, s*2)

        elif sh == "circle_dot":
            for pen in (shp, mp):
                p.setPen(pen); p.setBrush(Qt.NoBrush)
                p.drawEllipse(cx-s, cy-s, s*2, s*2)
            r = max(2, s//4)
            p.setPen(Qt.NoPen); p.setBrush(col)
            p.drawEllipse(cx-r, cy-r, r*2, r*2)

        elif sh == "cross_gap":
            gap = max(3, s//3)
            lines = [(cx-s,cy,cx-gap,cy),(cx+gap,cy,cx+s,cy),
                     (cx,cy-s,cx,cy-gap),(cx,cy+gap,cx,cy+s)]
            for pen in (shp, mp):
                p.setPen(pen)
                for x1,y1,x2,y2 in lines: p.drawLine(x1,y1,x2,y2)

        elif sh == "x_shape":
            for pen in (shp, mp):
                p.setPen(pen)
                p.drawLine(cx-s,cy-s,cx+s,cy+s)
                p.drawLine(cx+s,cy-s,cx-s,cy+s)

        elif sh == "triangle":
            path = QPainterPath()
            path.moveTo(cx, cy-s)
            path.lineTo(cx-s*0.8, cy+s*0.6)
            path.lineTo(cx+s*0.8, cy+s*0.6)
            path.closeSubpath()
            p.setPen(mp); p.setBrush(Qt.NoBrush); p.drawPath(path)



class MenuWidget(QWidget):

    def __init__(self, crosshair: CrosshairWidget):
        super().__init__()
        self.crosshair = crosshair
        self._visible  = False
        self._hotkey_callback = None
        self._setup()
        self._build()
        self._reposition()

    def _setup(self):
        self.setWindowFlags(Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint | Qt.Tool)
        self.setAttribute(Qt.WA_TranslucentBackground)
        self.setFixedWidth(290)

    def _build(self):
        outer = QVBoxLayout(self)
        outer.setContentsMargins(0, 0, 0, 0)

        card = QFrame(); card.setObjectName("card")
        card.setStyleSheet("""
            QFrame#card {
                background: rgba(8,10,16,220);
                border: 1px solid rgba(255,255,255,0.09);
                border-radius: 14px;
            }
        """)
        outer.addWidget(card)

        lay = QVBoxLayout(card)
        lay.setContentsMargins(18,16,18,18)
        lay.setSpacing(12)

        hdr = QHBoxLayout()
        t = QLabel("NeoCrossV1")
        t.setStyleSheet("color:#fff;font-size:13px;font-weight:700;letter-spacing:1.5px;")
        b = QLabel("BY T-GOW")
        b.setStyleSheet("""color:#00FF88;background:rgba(0,255,136,0.12);
            border:1px solid rgba(0,255,136,0.3);border-radius:4px;
            font-size:9px;font-weight:700;letter-spacing:1px;padding:2px 6px;""")
        hdr.addWidget(t); hdr.addStretch(); hdr.addWidget(b)
        lay.addLayout(hdr)
        lay.addWidget(self._sep())

        lay.addWidget(self._lbl("SHAPE  [X]"))
        self._shape_btns = []
        r = QHBoxLayout(); r.setSpacing(5)
        for i, sym in enumerate(SHAPE_SYMS):
            btn = self._btn(sym, lambda _, i=i: self._set_shape(i))
            self._shape_btns.append(btn); r.addWidget(btn)
        r.addStretch(); lay.addLayout(r)

        lay.addWidget(self._lbl("COLOR  [Z]"))
        self._color_btns = []
        r2 = QHBoxLayout(); r2.setSpacing(5)
        for i, (hx, _) in enumerate(COLORS):
            btn = QPushButton(); btn.setFixedSize(24,24)
            btn.clicked.connect(lambda _, i=i: self._set_color(i))
            self._color_btns.append(btn); r2.addWidget(btn)
        r2.addStretch(); lay.addLayout(r2)

        lay.addWidget(self._lbl("SIZE  [C]"))
        self._size_btns = []
        r3 = QHBoxLayout(); r3.setSpacing(5)
        for i, lbl in enumerate(SIZE_LABELS):
            btn = self._btn(lbl, lambda _, i=i: self._set_size(i), w=38)
            self._size_btns.append(btn); r3.addWidget(btn)
        r3.addStretch(); lay.addLayout(r3)

        lay.addWidget(self._sep())
        hint = QLabel("F2 Menu  ·  Z Color  ·  C Size  ·  X Shape  ·  F3 Exit")
        hint.setStyleSheet("color:rgba(255,255,255,0.22);font-size:9px;letter-spacing:0.5px;")
        hint.setAlignment(Qt.AlignCenter)
        lay.addWidget(hint)

        self.adjustSize()
        self.refresh_all()

    def _sep(self):
        f = QFrame(); f.setFixedHeight(1)
        f.setStyleSheet("background:rgba(255,255,255,0.07);"); return f

    def _lbl(self, text):
        l = QLabel(text)
        l.setStyleSheet("color:rgba(255,255,255,0.32);font-size:9px;font-weight:700;letter-spacing:1.5px;")
        return l

    def _btn(self, text, cb, w=30):
        b = QPushButton(text); b.setFixedSize(w, 28)
        b.clicked.connect(cb); b.setStyleSheet(self._sty(False)); return b

    @staticmethod
    def _sty(active):
        if active:
            return """QPushButton{background:rgba(0,255,136,0.18);color:#00FF88;
                border:1px solid rgba(0,255,136,0.5);border-radius:6px;
                font-size:11px;font-weight:700;}"""
        return """QPushButton{background:rgba(255,255,255,0.05);color:rgba(255,255,255,0.55);
            border:1px solid rgba(255,255,255,0.08);border-radius:6px;font-size:11px;}
            QPushButton:hover{background:rgba(255,255,255,0.10);color:#fff;}"""

    @staticmethod
    def _csty(hx, active):
        bd = "#ffffff" if active else "transparent"
        return f"""QPushButton{{background:{hx};border:2px solid {bd};border-radius:12px;}}
            QPushButton:hover{{border:2px solid #fff;}}"""

    def _set_shape(self, i):
        if not self._visible: return
        self.crosshair.shape_idx = i; save_settings(i, self.crosshair.color_idx, self.crosshair.size_idx); self.crosshair.update(); self._ref_shapes()

    def _set_color(self, i):
        if not self._visible: return
        self.crosshair.color_idx = i; save_settings(self.crosshair.shape_idx, i, self.crosshair.size_idx); self.crosshair.update(); self._ref_colors()

    def _set_size(self, i):
        if not self._visible: return
        self.crosshair.size_idx = i; save_settings(self.crosshair.shape_idx, self.crosshair.color_idx, i); self.crosshair.update(); self._ref_sizes()

    def _ref_shapes(self):
        for i, b in enumerate(self._shape_btns):
            b.setStyleSheet(self._sty(i == self.crosshair.shape_idx))

    def _ref_colors(self):
        for i, b in enumerate(self._color_btns):
            b.setStyleSheet(self._csty(COLORS[i][0], i == self.crosshair.color_idx))

    def _ref_sizes(self):
        for i, b in enumerate(self._size_btns):
            b.setStyleSheet(self._sty(i == self.crosshair.size_idx))

    def refresh_all(self):
        self._ref_shapes(); self._ref_colors(); self._ref_sizes()

    def _reposition(self):
        s = QApplication.primaryScreen().geometry()
        self.adjustSize()
        self.move(s.right()-self.width()-20, 20)

    def toggle(self):
        if self._visible:
            self.hide(); self._visible = False
            if self._hotkey_callback: self._hotkey_callback(False)
        else:
            self._reposition(); self.refresh_all()
            self.show(); self._visible = True
            if self._hotkey_callback: self._hotkey_callback(True)

    def paintEvent(self, _):
        p = QPainter(self)
        p.setCompositionMode(QPainter.CompositionMode_Clear)
        p.fillRect(self.rect(), Qt.transparent)

    def set_hotkey_callback(self, cb):
        self._hotkey_callback = cb



class NeoCrossApp:

    def __init__(self):
        self.app = QApplication(sys.argv)
        self.app.setApplicationName("NeoCrossV1")
        self.app.setQuitOnLastWindowClosed(False)

        self.crosshair = CrosshairWidget()
        self.menu      = MenuWidget(self.crosshair)
        self.crosshair.show()
        make_click_through(int(self.crosshair.winId()))

        self.sig = HotkeySignals()
        self.sig.toggle_menu.connect(self.menu.toggle)
        self.sig.next_color.connect(self._color)
        self.sig.next_size.connect(self._size)
        self.sig.next_shape.connect(self._shape)
        self.sig.quit_app.connect(self._quit)

        self.menu.set_hotkey_callback(self._set_edit_hotkeys_enabled)

        self.hk = HotkeyThread(self.sig)
        self.hk.start()

        self._t = QTimer()
        self._t.timeout.connect(self.crosshair.recenter)
        self._t.start(2000)

        self.app.aboutToQuit.connect(self._cleanup)

    def _set_edit_hotkeys_enabled(self, enabled: bool):
        self.hk.set_menu_visible(enabled)
    
    def _quit(self):
        if self.menu._visible:
            save_settings(self.crosshair.shape_idx, self.crosshair.color_idx, self.crosshair.size_idx)
            self.app.quit()

    def _color(self):
        if not self.menu._visible: return
        self.crosshair.next_color(); self.menu._ref_colors()

    def _size(self):
        if not self.menu._visible: return
        self.crosshair.next_size(); self.menu._ref_sizes()

    def _shape(self):
        if not self.menu._visible: return
        self.crosshair.next_shape(); self.menu._ref_shapes()

    def _cleanup(self):
        self.hk.stop(); self.hk.join(timeout=1)

    def run(self):
        sys.exit(self.app.exec_())


if __name__ == "__main__":
    try:
        ctypes.windll.shcore.SetProcessDpiAwareness(2)
    except Exception:
        try: ctypes.windll.user32.SetProcessDPIAware()
        except Exception: pass

    NeoCrossApp().run()
