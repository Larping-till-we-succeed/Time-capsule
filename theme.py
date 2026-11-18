"""Visual language for TimeCapsule: palette, stylesheet and small painting helpers."""

from PySide6.QtCore import QRectF, Qt
from PySide6.QtGui import QColor, QFont, QLinearGradient, QPainter, QPen
from PySide6.QtWidgets import QFrame, QHBoxLayout, QLabel, QSizePolicy, QVBoxLayout, QWidget

BG = "#050a12"
SIDEBAR = "#050b14"
SURF = "#0a1826"
SURF2 = "#0f2233"
BORDER = "#122b42"
BORDER2 = "#1d4463"
TEXT = "#e9f1fb"
MUTED = "#8ea6c2"
DIM = "#5a7290"
ACCENT = "#38bdf8"
ACCENT_DARK = "#1667c9"
GREEN = "#34d399"
RED = "#f87171"
BLUE = "#7dd3fc"
VIOLET = "#a78bfa"

UI_FONT = "'Inter','SF Pro Text','Segoe UI','Helvetica Neue',sans-serif"
MONO = "'JetBrains Mono','Cascadia Code',Consolas,Menlo,monospace"

STATUS = {
    "added": ("Added", GREEN),
    "modified": ("Modified", ACCENT),
    "deleted": ("Deleted", RED),
    "unchanged": ("Unchanged", DIM),
}

QSS = f"""
QWidget{{background:transparent;color:{TEXT};font-family:{UI_FONT};font-size:14px}}
QMainWindow,#root{{background:qlineargradient(x1:0,y1:0,x2:0.65,y2:1,stop:0 #0b2437,stop:0.5 #061220,stop:1 {BG})}}
QLabel{{background:transparent}}
QToolTip{{background:#0b1c2c;color:{TEXT};border:1px solid {BORDER2};border-radius:8px;padding:7px 10px}}
QScrollArea{{border:none;background:transparent}}
QScrollArea > QWidget > QWidget{{background:transparent}}
QScrollBar:vertical{{background:transparent;width:11px;margin:2px 2px 2px 0}}
QScrollBar::handle:vertical{{background:#14344c;border-radius:4px;min-height:44px}}
QScrollBar::handle:vertical:hover{{background:#235c85}}
QScrollBar::add-line:vertical,QScrollBar::sub-line:vertical{{height:0}}
QScrollBar::add-page:vertical,QScrollBar::sub-page:vertical{{background:transparent}}
QScrollBar:horizontal{{background:transparent;height:11px;margin:0 2px 2px 2px}}
QScrollBar::handle:horizontal{{background:#14344c;border-radius:4px;min-width:44px}}
QScrollBar::handle:horizontal:hover{{background:#235c85}}
QScrollBar::add-line:horizontal,QScrollBar::sub-line:horizontal{{width:0}}

#sidebar{{background:{SIDEBAR};border-right:1px solid #0f2437}}
#sidebar QLabel{{background:transparent}}
#logo{{font-size:18px;font-weight:800;letter-spacing:0.4px;color:{TEXT}}}
#logomark{{font-size:21px}}
#wordmark{{font-size:10px;font-weight:800;letter-spacing:2.6px;color:{ACCENT}}}
#projrow{{background:#0a1b2b;border:1px solid {BORDER};border-radius:11px}}
#projrow:hover{{background:#0f2437;border-color:{BORDER2}}}
#projrow[active="true"]{{background:#102a44;border:1px solid #1f5f92}}
#nav{{background:transparent;border:1px solid transparent;border-radius:10px;padding:9px 12px;text-align:left;font-weight:700;font-size:14px;color:{MUTED}}}
#nav:hover{{background:#0e2135;color:{TEXT}}}
#nav[active="true"]{{background:qlineargradient(x1:0,y1:0,x2:1,y2:0,stop:0 #17517f,stop:0.65 #123c63,stop:1 #0d2a46);border:1px solid #1f5f92;color:#c4e7ff}}

#primary{{background:qlineargradient(x1:0,y1:0,x2:0,y2:1,stop:0 #4aa8ff,stop:1 #2268d8);border:1px solid #2f7fdc;border-bottom:2px solid #1450a8;border-radius:11px;padding:10px 20px;font-weight:800;font-size:14px;color:#f2f9ff}}
#primary:hover{{background:qlineargradient(x1:0,y1:0,x2:0,y2:1,stop:0 #69b8ff,stop:1 #2b76ea)}}
#primary:pressed{{border-bottom:1px solid #1450a8;padding-top:11px;padding-bottom:9px}}
#primary:disabled{{background:#0d1f30;border:1px solid #16344d;border-bottom:2px solid #10283c;color:{DIM}}}
#ghost{{background:#0e2135;border:1px solid #1b3f5c;border-radius:11px;padding:9px 16px;font-weight:700;color:#dbe9f8}}
#ghost:hover{{background:#142c44;border-color:#2a5d85;color:#fff}}
#ghost:pressed{{background:#0a1a2a}}
#ghost:disabled{{color:{DIM};border-color:#122b42;background:#071422}}
#danger{{background:#2a1319;border:1px solid #6b2a34;border-radius:11px;padding:9px 16px;font-weight:700;color:#ff9b9b}}
#danger:hover{{background:#3a1a22;border-color:#8f3945}}

#title{{font-size:22px;font-weight:800;letter-spacing:-0.2px}}
#bigtime{{font-size:19px;font-weight:800;letter-spacing:-0.2px}}
#sub{{color:{MUTED};font-size:12.5px}}
#tiny{{color:{DIM};font-size:10px;font-weight:800;letter-spacing:1.4px}}
#card{{background:qlineargradient(x1:0,y1:0,x2:0,y2:1,stop:0 #0c1d2d,stop:1 #091622);border:1px solid #143049;border-radius:16px}}
#chip{{background:#0f2437;border:1px solid #1b3f5c;border-radius:8px;padding:3px 9px;font-size:11.5px;font-weight:700;color:#c6dbf0}}
#hr{{background:{BORDER};max-height:1px;min-height:1px;border:none}}
#empty{{color:{MUTED};font-size:13.5px}}
#diff{{background:#040c15;border:1px solid #12293e;border-radius:12px;font-family:{MONO};font-size:12.5px;padding:14px;color:#c9d9e8}}
#note{{background:#071422;border:1px solid #163049;border-radius:12px;padding:10px}}
#statval{{font-size:27px;font-weight:800;letter-spacing:-0.6px}}
#statcap{{color:{MUTED};font-size:10.5px;font-weight:800;letter-spacing:1.3px}}

QLineEdit,QPlainTextEdit,QComboBox,QSpinBox{{background:#071422;border:1px solid #17344d;border-radius:10px;padding:9px 12px;color:{TEXT};selection-background-color:#164e80;selection-color:{TEXT}}}
QLineEdit:hover,QPlainTextEdit:hover,QComboBox:hover,QSpinBox:hover{{border-color:#235070}}
QLineEdit:focus,QPlainTextEdit:focus,QComboBox:focus,QSpinBox:focus{{border-color:{ACCENT};background:#0a1a2a}}
QLineEdit::placeholder{{color:{DIM}}}
QComboBox::drop-down{{border:none;width:26px}}
QComboBox QAbstractItemView{{background:#0b1c2c;border:1px solid {BORDER2};border-radius:10px;selection-background-color:#164e80;outline:none;padding:5px}}
QComboBox QAbstractItemView::item{{min-height:26px;padding:2px 6px}}
QSpinBox{{padding-right:2px}}

QPushButton{{background:#0e2135;border:1px solid #1b3f5c;border-radius:10px;padding:8px 14px;font-weight:600;color:#dbe9f8}}
QPushButton:hover{{background:#16344d;border-color:#2a5d85}}
QPushButton:pressed{{background:#0a1a2a}}
QPushButton:disabled{{background:#071422;border-color:#122b42;color:{DIM}}}
QCheckBox{{color:#cfdff0;font-weight:600;spacing:9px}}
QCheckBox::indicator{{width:16px;height:16px;border:1px solid #2a4a66;border-radius:5px;background:#071422}}
QCheckBox::indicator:hover{{border-color:{ACCENT}}}
QCheckBox::indicator:checked{{background:{ACCENT};border-color:{ACCENT}}}
QSplitter::handle{{background:#0d2234;width:1px}}
QListWidget{{background:transparent;border:none;outline:none}}
QDialog{{background:{BG};border:1px solid {BORDER}}}
QDialog QLabel{{background:transparent}}
QMessageBox{{background:{BG}}}
QMessageBox QLabel{{color:{TEXT};font-size:13.5px}}

#hero{{border:1px solid #123049;border-radius:18px}}
#heroTitle{{font-size:27px;font-weight:800;letter-spacing:-0.6px;color:#f4f9ff}}
#heroSub{{font-size:13px;color:#9db4cd}}
#searchbox{{background:#07131f;border:1px solid #15334e;border-radius:11px}}
#searchbox:hover{{border-color:#235070}}
#searchbox QLineEdit{{background:transparent;border:none;padding:0;font-size:13px}}
#searchglyph{{color:#6d87a2;font-size:17px}}
#searcheyebrow{{color:#6d87a2;font-size:10.5px;font-weight:700;border:1px solid #17344d;border-radius:6px;padding:2px 6px}}
#quickrow{{background:transparent;border:1px solid transparent;border-radius:10px}}
#quickrow:hover{{background:#0e2135;border-color:#1b3f5c}}
#statusbar{{background:#040a12;border-top:1px solid #0f2437}}
#sblabel{{color:#8ea6c2;font-size:12px}}
#sbstrong{{color:#d7e8f8;font-size:12px;font-weight:700}}
#link{{background:transparent;border:none;padding:0;color:{ACCENT};font-size:12px;font-weight:700}}
#link:hover{{color:#8fd6ff}}
#statbig{{font-size:24px;font-weight:800;letter-spacing:-0.4px}}
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
        bg = bg or "#0f2437"
        lab.setStyleSheet(
            f"background:{bg};border:1px solid {QColor(color or '#c8ccd4').darker(360).name()};"
            "border-radius:8px;padding:3px 9px;font-size:11.5px;font-weight:700;"
            f"color:{color or '#c8ccd4'}"
        )
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
