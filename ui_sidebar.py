"""Left sidebar: brand, project switcher, navigation and the time-machine mark."""

from PySide6.QtCore import QPointF, QRectF, Qt, Signal
from PySide6.QtGui import QColor, QFontMetrics, QLinearGradient, QPainter, QPainterPath, QPen
from PySide6.QtWidgets import (
    QHBoxLayout, QLabel, QPushButton, QScrollArea, QVBoxLayout, QWidget,
)

import store
from theme import ACCENT, DIM, MUTED, TEXT, hbox, hline
from widgets import ClickFrame

VIEWS = [
    ("dashboard", "⌂", "Dashboard"),
    ("timeline", "◷", "Timeline"),
    ("compare", "⚖", "Compare"),
    ("stats", "▤", "Statistics"),
]


class HourglassArt(QWidget):
    """A small glowing hourglass, echoing the reference sidebar illustration."""

    def __init__(self, parent=None):
        super().__init__(parent)
        self.setFixedSize(96, 96)

    def paintEvent(self, e):
        p = QPainter(self)
        p.setRenderHint(QPainter.Antialiasing)
        w, h = self.width(), self.height()
        cx = w / 2

        glow = QColor(ACCENT)
        glow.setAlpha(26)
        p.setPen(Qt.NoPen)
        p.setBrush(glow)
        p.drawEllipse(QPointF(cx, h * 0.5), w * 0.46, h * 0.42)

        top, bottom = 14.0, h - 22.0
        glass = QPainterPath()
        glass.moveTo(cx - 25, top)
        glass.cubicTo(cx - 17, top + 26, cx - 4, top + 30, cx - 3, h * 0.5)
        glass.cubicTo(cx - 4, bottom - 30, cx - 17, bottom - 26, cx - 25, bottom)
        glass.lineTo(cx + 25, bottom)
        glass.cubicTo(cx + 17, bottom - 26, cx + 4, bottom - 30, cx + 3, h * 0.5)
        glass.cubicTo(cx + 4, top + 30, cx + 17, top + 26, cx + 25, top)
        glass.closeSubpath()
        fill = QLinearGradient(0, top, 0, bottom)
        fill.setColorAt(0.0, QColor(56, 189, 248, 34))
        fill.setColorAt(0.5, QColor(56, 189, 248, 12))
        fill.setColorAt(1.0, QColor(56, 189, 248, 30))
        p.setBrush(fill)
        p.setPen(QPen(QColor(140, 224, 255, 175), 1.3))
        p.drawPath(glass)

        sand = QColor("#5ec8f5")
        p.setPen(Qt.NoPen)
        p.setBrush(sand)
        p.drawPolygon([QPointF(cx - 15, top + 6), QPointF(cx + 15, top + 6),
                       QPointF(cx + 7, top + 19), QPointF(cx - 7, top + 19)])
        mound = QPainterPath()
        mound.moveTo(cx - 16, bottom - 5)
        mound.cubicTo(cx - 8, bottom - 18, cx + 8, bottom - 18, cx + 16, bottom - 5)
        mound.closeSubpath()
        p.setBrush(QColor(94, 200, 245, 190))
        p.drawPath(mound)
        p.setPen(QPen(QColor(200, 240, 255, 190), 1.0))
        p.drawLine(QPointF(cx, top + 22), QPointF(cx, bottom - 16))

        for x, y, r in ((cx - 34, h * 0.32, 1.2), (cx + 33, h * 0.42, 1.0),
                        (cx - 29, h * 0.68, 0.9), (cx + 30, h * 0.72, 1.3)):
            p.setPen(Qt.NoPen)
            p.setBrush(QColor(150, 225, 255, 150))
            p.drawEllipse(QPointF(x, y), r, r)

        for y, wide in ((top, 33.0), (bottom, 33.0)):
            p.setPen(Qt.NoPen)
            p.setBrush(QColor(74, 160, 205, 200))
            p.drawRoundedRect(QRectF(cx - wide / 2, y - 3, wide, 3.4), 1.7, 1.7)


class ProjectRow(ClickFrame):
    def __init__(self, proj, count, active=False, parent=None):
        super().__init__(parent)
        self.pid = proj["id"]
        self.setObjectName("projrow")
        self.setProperty("active", "true" if active else "false")
        lay = QVBoxLayout(self)
        lay.setContentsMargins(12, 10, 12, 10)
        lay.setSpacing(3)
        name = QLabel(proj["name"])
        name.setStyleSheet("font-weight:800;font-size:13.5px;letter-spacing:-0.1px")
        path = QLabel(proj["path"])
        path.setObjectName("sub")
        path.setStyleSheet("font-size:10.5px")
        fm = QFontMetrics(path.font())
        path.setText(fm.elidedText(proj["path"], Qt.ElideMiddle, 168))
        path.setToolTip(proj["path"])
        meta = QLabel(f"{count} snapshot{'s' if count != 1 else ''}")
        meta.setStyleSheet(
            f"color:{'#7dd3fc' if active else ACCENT};font-size:10.5px;font-weight:800;"
            "letter-spacing:0.4px")
        lay.addWidget(name)
        lay.addWidget(path)
        lay.addWidget(meta)


class Sidebar(QWidget):
    project_selected = Signal(str)
    add_project = Signal()
    view_changed = Signal(str)

    def __init__(self, parent=None):
        super().__init__(parent)
        self.setObjectName("sidebar")
        self.setFixedWidth(252)
        self.view = "dashboard"
        root = QVBoxLayout(self)
        root.setContentsMargins(16, 18, 16, 14)
        root.setSpacing(12)

        from ui_dashboard import ClockLogo
        mark = ClockLogo(30)
        word = QLabel("TimeCapsule")
        word.setObjectName("logo")
        root.addLayout(hbox(mark, word, spacing=9))

        root.addSpacing(12)
        head = hbox(spacing=8)
        cap = QLabel("PROJECTS")
        cap.setObjectName("tiny")
        head.addWidget(cap)
        head.addStretch(1)
        add = QPushButton("＋")
        add.setToolTip("Add a project folder")
        add.setCursor(Qt.PointingHandCursor)
        add.setFixedSize(24, 24)
        add.setStyleSheet(
            "QPushButton{background:#0d2438;border:1px solid #1c405e;border-radius:7px;"
            "font-size:13px;padding:0;color:#c8ccd4}"
            "QPushButton:hover{border-color:#38bdf8;color:#38bdf8;background:#0e2a44}"
        )
        add.clicked.connect(self.add_project.emit)
        head.addWidget(add)
        root.addLayout(head)

        self.plist = QWidget()
        self.pcol = QVBoxLayout(self.plist)
        self.pcol.setContentsMargins(0, 0, 0, 0)
        self.pcol.setSpacing(8)
        pscroll = QScrollArea()
        pscroll.setWidgetResizable(True)
        pscroll.setWidget(self.plist)
        pscroll.setMaximumHeight(216)
        root.addWidget(pscroll)

        root.addWidget(hline())
        nav = QVBoxLayout()
        nav.setSpacing(3)
        self.navbtns = {}
        for key, icon, name in VIEWS:
            self._add_nav(nav, key, icon, name)
        root.addLayout(nav)

        root.addSpacing(2)
        root.addWidget(hline())
        srow = QVBoxLayout()
        srow.setSpacing(3)
        self._add_nav(srow, "settings", "⚙", "Settings")
        root.addLayout(srow)
        root.addStretch(1)

        art_row = QHBoxLayout()
        art_row.setContentsMargins(0, 0, 0, 0)
        art_row.addStretch(1)
        art_row.addWidget(HourglassArt())
        art_row.addStretch(1)
        root.addLayout(art_row)
        name = QLabel("TimeCapsule")
        name.setAlignment(Qt.AlignCenter)
        name.setStyleSheet(f"color:{ACCENT};font-size:15px;font-weight:800;letter-spacing:-0.2px")
        root.addWidget(name)
        tag = QLabel("Travel through time.\nNever lose progress.")
        tag.setAlignment(Qt.AlignCenter)
        tag.setObjectName("sub")
        root.addWidget(tag)
        root.addSpacing(8)
        self.storage = QLabel("")
        self.storage.setObjectName("tiny")
        self.storage.setAlignment(Qt.AlignCenter)
        root.addWidget(self.storage)
        ver = QLabel("TimeCapsule 0.0")
        ver.setObjectName("tiny")
        ver.setStyleSheet(f"color:#3d5169;font-size:9px")
        ver.setAlignment(Qt.AlignCenter)
        root.addWidget(ver)
        self.set_view("dashboard")

    def _add_nav(self, lay, key, icon, name):
        b = QPushButton(f"{icon}   {name}")
        b.setObjectName("nav")
        b.setCursor(Qt.PointingHandCursor)
        b.setCheckable(True)
        b.setMinimumHeight(38)
        b.setProperty("active", "false")
        b.clicked.connect(lambda _=False, k=key: self.set_view(k, emit=True))
        self.navbtns[key] = b
        lay.addWidget(b)

    def set_view(self, key, emit=False):
        self.view = key
        for k, b in self.navbtns.items():
            active = k == key
            b.setProperty("active", "true" if active else "false")
            b.setChecked(active)
            b.style().unpolish(b)
            b.style().polish(b)
        if emit:
            self.view_changed.emit(key)

    def reload(self, projects, current_pid, totals):
        while self.pcol.count():
            item = self.pcol.takeAt(0)
            w = item.widget()
            if w:
                w.deleteLater()
        if not projects:
            empty = QLabel("No projects yet.\nAdd a folder to start.")
            empty.setObjectName("sub")
            empty.setWordWrap(True)
            self.pcol.addWidget(empty)
        for p in projects:
            row = ProjectRow(p, totals.get(p["id"], 0), p["id"] == current_pid)
            row.clicked.connect(lambda pid=p["id"]: self.project_selected.emit(pid))
            self.pcol.addWidget(row)
        self.pcol.addStretch(1)
        self.storage.setText(totals.get("__storage__", ""))
