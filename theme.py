"""Visual language for TimeCapsule: palette, stylesheet and small painting helpers."""

from PySide6.QtCore import QRectF, Qt
from PySide6.QtGui import QColor, QFont, QLinearGradient, QPainter, QPen
from PySide6.QtWidgets import QFrame, QHBoxLayout, QLabel, QSizePolicy, QVBoxLayout, QWidget

BG = "#0b0c0e"
SIDEBAR = "#0e0f12"
SURF = "#121419"
SURF2 = "#181b21"
BORDER = "#1f222a"
BORDER2 = "#2b2f39"
TEXT = "#f2f3f5"
MUTED = "#9096a1"
DIM = "#5c6270"
ACCENT = "#ffb454"
ACCENT_DARK = "#c47c1d"
GREEN = "#4ade80"
RED = "#f87171"
BLUE = "#7dd3fc"
VIOLET = "#c4b5fd"

UI_FONT = "'Inter','SF Pro Text','Segoe UI','Helvetica Neue',sans-serif"
MONO = "'JetBrains Mono','Cascadia Code',Consolas,Menlo,monospace"

STATUS = {
    "added": ("Added", GREEN),
    "modified": ("Modified", ACCENT),
    "deleted": ("Deleted", RED),
    "unchanged": ("Unchanged", DIM),
}

QSS = f"""
QWidget{{background:transparent;color:{TEXT};font-family:{UI_FONT};font-size:15px}}
QMainWindow,#root{{background:{BG}}}
QLabel{{background:transparent}}
QToolTip{{background:#14161b;color:{TEXT};border:1px solid {BORDER2};border-radius:8px;padding:6px 9px}}
QScrollArea{{border:none;background:transparent}}
QScrollBar:vertical{{background:transparent;width:10px;margin:2px}}
QScrollBar::handle:vertical{{background:#2a2d35;border-radius:5px;min-height:40px}}
QScrollBar::handle:vertical:hover{{background:#3a3f4a}}
QScrollBar::add-line:vertical,QScrollBar::sub-line:vertical{{height:0}}
QScrollBar:horizontal{{background:transparent;height:10px;margin:2px}}
QScrollBar::handle:horizontal{{background:#2a2d35;border-radius:5px;min-width:40px}}
QScrollBar::add-line:horizontal,QScrollBar::sub-line:horizontal{{width:0}}
#sidebar{{background:{SIDEBAR};border-right:1px solid #17191e}}
#sidebar QLabel{{background:transparent}}
#logo{{font-size:19px;font-weight:800;letter-spacing:1px;color:{TEXT}}}
#logomark{{font-size:22px}}
#wordmark{{font-size:12px;font-weight:700;letter-spacing:3px;color:{ACCENT}}}
#projrow{{background:#13151a;border:1px solid {BORDER};border-radius:12px}}
#projrow:hover{{background:#171a20;border-color:{BORDER2}}}
#projrow[active="true"]{{background:#1b1d23;border:1px solid #7a5a24}}
#nav{{background:transparent;border:none;border-radius:10px;padding:10px 14px;text-align:left;font-weight:700;color:{MUTED}}}
#nav:hover{{background:#171a20;color:{TEXT}}}
#nav[active="true"]{{background:#1a1c22;color:{ACCENT}}}
#primary{{background:qlineargradient(x1:0,y1:0,x2:0,y2:1,stop:0 #ffc670,stop:1 #f7a53f);border:none;border-bottom:2px solid {ACCENT_DARK};border-radius:11px;padding:10px 20px;font-weight:800;font-size:15px;color:#1a1206}}
#primary:hover{{background:qlineargradient(x1:0,y1:0,x2:0,y2:1,stop:0 #ffd189,stop:1 #f9ae4c)}}
#primary:pressed{{border-bottom:1px solid {ACCENT_DARK}}}
#primary:disabled{{background:#22252c;border-bottom:2px solid #1a1c21;color:{DIM}}}
#ghost{{background:#14161b;border:1px solid #262a33;border-radius:11px;padding:9px 16px;font-weight:700;color:#e6e9ef}}
#ghost:hover{{background:#1a1d24;border-color:#343945}}
#ghost:disabled{{color:{DIM};border-color:#1c1f26;background:#101216}}
#danger{{background:#241216;border:1px solid #5f242c;border-radius:11px;padding:9px 16px;font-weight:700;color:#ff9a9a}}
#danger:hover{{background:#2e161b;border-color:#7d3039}}
#title{{font-size:23px;font-weight:800}}
#bigtime{{font-size:20px;font-weight:800}}
#sub{{color:{MUTED};font-size:13px}}
#tiny{{color:{DIM};font-size:11px;font-weight:700;letter-spacing:1px}}
#card{{background:{SURF};border:1px solid {BORDER};border-radius:16px}}
#chip{{background:#1a1d23;border-radius:9px;padding:3px 9px;font-size:12px;font-weight:700;color:#c8ccd4}}
#hr{{background:{BORDER};max-height:1px;min-height:1px;border:none}}
#empty{{color:{MUTED};font-size:14px}}
#diff{{background:#0a0c0f;border:1px solid #1e222a;border-radius:12px;font-family:{MONO};font-size:13px;padding:12px;color:#c9d1d9}}
#note{{background:#0c0e12;border:1px solid #22262e;border-radius:12px;padding:10px}}
#statval{{font-size:26px;font-weight:800}}
#statcap{{color:{MUTED};font-size:12px;font-weight:700;letter-spacing:1px}}
QLineEdit,QPlainTextEdit,QComboBox{{background:#0c0e12;border:1px solid #22262e;border-radius:10px;padding:9px 12px;color:{TEXT};selection-background-color:#3a2a12;selection-color:{TEXT}}}
QLineEdit:focus,QPlainTextEdit:focus,QComboBox:focus{{border-color:{ACCENT}}}
QLineEdit::placeholder{{color:{DIM}}}
QComboBox::drop-down{{border:none;width:24px}}
QComboBox QAbstractItemView{{background:#14161b;border:1px solid {BORDER2};border-radius:10px;selection-background-color:#23262e;outline:none;padding:4px}}
QPushButton{{background:#14161b;border:1px solid #262a33;border-radius:10px;padding:8px 14px;font-weight:600;color:#e6e9ef}}
QPushButton:hover{{background:#1a1d24;border-color:#343945}}
QPushButton:disabled{{background:#101216;border-color:#1c1f26;color:{DIM}}}
QCheckBox{{color:#c8ccd4;font-weight:600}}
QSplitter::handle{{background:#17191e;width:1px}}
QListWidget{{background:transparent;border:none;outline:none}}
QDialog{{background:{BG};border:1px solid {BORDER}}}
QDialog QLabel{{background:transparent}}
QMessageBox{{background:{BG}}}
QMessageBox QLabel{{color:{TEXT};font-size:14px}}
QCheckBox{{color:#c8ccd4;font-weight:600;spacing:8px}}
QCheckBox::indicator{{width:16px;height:16px;border:1px solid #2b2f39;border-radius:5px;background:#0c0e12}}
QCheckBox::indicator:checked{{background:{ACCENT};border-color:{ACCENT}}}
"""


def label(text, name=None, wrap=False, align=None):
    w = QLabel(text)
    if name:
        w.setObjectName(name)
    w.setWordWrap(wrap)
    if align is not None:
        w.setAlignment(align)
    return w


def hline():
    w = QFrame()
    w.setObjectName("hr")
    w.setFixedHeight(1)
    w.setSizePolicy(QSizePolicy.Expanding, QSizePolicy.Fixed)
    return w


def hbox(*widgets, spacing=8, margins=(0, 0, 0, 0)):
    lay = QHBoxLayout()
    lay.setSpacing(spacing)
    lay.setContentsMargins(*margins)
    for w in widgets:
        if w is None:
            lay.addStretch(1)
        elif isinstance(w, int):
            lay.addSpacing(w)
        else:
            lay.addWidget(w)
    return lay


def vbox(*widgets, spacing=8, margins=(0, 0, 0, 0)):
    lay = QVBoxLayout()
    lay.setSpacing(spacing)
    lay.setContentsMargins(*margins)
    for w in widgets:
        if w is None:
            lay.addStretch(1)
        elif isinstance(w, int):
            lay.addSpacing(w)
        else:
            lay.addWidget(w)
    return lay


def chip(text, color=None, bg=None):
    lab = QLabel(text)
    lab.setObjectName("chip")
    if color or bg:
        bg = bg or "#1a1d23"
        lab.setStyleSheet(f"background:{bg};border-radius:9px;padding:3px 9px;font-size:12px;font-weight:700;color:{color or '#c8ccd4'}")
    return lab


def accent_color(kind):
    return STATUS.get(kind, STATUS["unchanged"])[1]


def status_label(kind):
    return STATUS.get(kind, STATUS["unchanged"])[0]


def paint_dot(p, x, y, r, color, glow=False):
    if glow:
        g = QColor(color)
        g.setAlpha(46)
        p.setBrush(g)
        p.setPen(Qt.NoPen)
        p.drawEllipse(QRectF(x - r * 2.4, y - r * 2.4, r * 4.8, r * 4.8))
    p.setBrush(QColor(color))
    p.setPen(Qt.NoPen)
    p.drawEllipse(QRectF(x - r, y - r, r * 2, r * 2))


class Divider(QFrame):
    def __init__(self, color=BORDER):
        super().__init__()
        self.setFixedHeight(1)
        self._c = color

    def paintEvent(self, e):
        p = QPainter(self)
        p.fillRect(self.rect(), QColor(self._c))
