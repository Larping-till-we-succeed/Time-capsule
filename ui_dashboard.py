"""Dashboard: the mission-control overview of everything TimeCapsule holds."""

import os
import platform
import time
from datetime import datetime

from PySide6.QtCore import QPointF, QRectF, QSize, Qt, Signal
from PySide6.QtGui import (
    QColor, QFont, QLinearGradient, QPainter, QPainterPath, QPen, QRadialGradient,
)
from PySide6.QtWidgets import (
    QFrame, QHBoxLayout, QLabel, QLineEdit, QPushButton, QScrollArea, QSizePolicy,
    QVBoxLayout, QWidget,
)

import store
from theme import (
    ACCENT, BORDER, BORDER2, DIM, GREEN, MUTED, RED, TEXT, VIOLET,
    chip, hbox, hline, paint_dot, vbox,
)
from widgets import (
    ClickFrame, MiniBar, SnapshotBadge, SnapshotPreview, Sparkline, ToggleSwitch,
    paint_preview, snapshot_preview,
)


def _clock(ts):
    d = datetime.fromtimestamp(ts)
    hour = d.hour % 12 or 12
    return f"{hour}:{d.minute:02d} {'AM' if d.hour < 12 else 'PM'}"


def _day(ts):
    label = store.day_label(ts)
    if label.startswith(("Today", "Yesterday")):
        return label
    return datetime.fromtimestamp(ts).strftime("%b %d")


def _uptime():
    try:
        with open("/proc/uptime", encoding="utf-8") as f:
            secs = float(f.read().split()[0])
        return f"{int(secs // 3600)}h {int(secs % 3600 // 60)}m"
    except (OSError, ValueError):
        return "—"


def _tile(icon, color, size=40, radius=11):
    tile = QLabel(icon)
    tile.setAlignment(Qt.AlignCenter)
    tile.setFixedSize(size, size)
    tile.setStyleSheet(
        f"background:{QColor(color).darker(430).name()};color:{color};"
        f"border:1px solid {QColor(color).darker(320).name()};border-radius:{radius}px;"
        f"font-size:{int(size * 0.44)}px"
    )
    return tile


class ClockLogo(QWidget):
    """A glowing clock mark, like the one in the reference header."""

    def __init__(self, size=56, parent=None):
        super().__init__(parent)
        self.setFixedSize(size, size)

    def paintEvent(self, e):
        p = QPainter(self)
        p.setRenderHint(QPainter.Antialiasing)
        cx, cy = self.width() / 2, self.height() / 2
        r = min(cx, cy) - 7
        glow = QColor(ACCENT)
        for i, alpha in ((9, 16), (6, 26), (3, 44)):
            glow.setAlpha(alpha)
            p.setPen(QPen(glow, 2.2))
            p.setBrush(Qt.NoBrush)
            p.drawEllipse(QPointF(cx, cy), r + i, r + i)
        c = QColor(ACCENT)
        c.setAlpha(70)
        p.setPen(QPen(c, 1.4))
        p.drawEllipse(QPointF(cx, cy), r, r)
        p.setPen(QPen(QColor("#a9e6ff"), 2.4))
        p.drawEllipse(QPointF(cx, cy), r - 1.5, r - 1.5)
        p.setPen(QPen(QColor("#e6f7ff"), 2.2))
        p.drawLine(QPointF(cx, cy), QPointF(cx, cy - r * 0.52))
        p.drawLine(QPointF(cx, cy), QPointF(cx + r * 0.36, cy + r * 0.22))
        p.setPen(Qt.NoPen)
        p.setBrush(QColor("#ffffff"))
        p.drawEllipse(QPointF(cx, cy), 2.1, 2.1)


class HeroHeader(QFrame):
    """The banner at the top: planet limb, glowing brand, search and actions."""

    def __init__(self, parent=None):
        super().__init__(parent)
        self.setObjectName("hero")
        self.setFixedHeight(134)
        lay = QHBoxLayout(self)
        lay.setContentsMargins(26, 0, 20, 0)
        lay.setSpacing(16)

        lay.addWidget(ClockLogo(56))
        col = QVBoxLayout()
        col.setSpacing(1)
        title = QLabel("TimeCapsule")
        title.setObjectName("heroTitle")
        sub = QLabel("Your digital time machine.")
        sub.setObjectName("heroSub")
        col.addWidget(title)
        col.addWidget(sub)
        lay.addLayout(col)
        lay.addStretch(1)

        self.search = QLineEdit()
        self.search.setPlaceholderText("Search snapshots…")
        self.search.setClearButtonEnabled(True)
        box = QFrame()
        box.setObjectName("searchbox")
        box.setFixedSize(330, 42)
        bl = QHBoxLayout(box)
        bl.setContentsMargins(12, 0, 8, 0)
        bl.setSpacing(8)
        glyph = QLabel("⌕")
        glyph.setObjectName("searchglyph")
        hint = QLabel("Ctrl + K")
        hint.setObjectName("searcheyebrow")
        bl.addWidget(glyph)
        bl.addWidget(self.search, 1)
        bl.addWidget(hint)
        lay.addWidget(box)

        self.newproj = QPushButton("＋  New project")
        self.newproj.setObjectName("ghost")
        self.newproj.setCursor(Qt.PointingHandCursor)
        self.newproj.setMinimumHeight(40)
        self.newsnap = QPushButton("＋  New snapshot")
        self.newsnap.setObjectName("primary")
        self.newsnap.setCursor(Qt.PointingHandCursor)
        self.newsnap.setMinimumHeight(40)
        lay.addWidget(self.newproj)
        lay.addWidget(self.newsnap)

    def paintEvent(self, e):
        p = QPainter(self)
        p.setRenderHint(QPainter.Antialiasing)
        w, h = self.width(), self.height()
        rect = QRectF(self.rect()).adjusted(0.5, 0.5, -0.5, -0.5)
        path = QPainterPath()
        path.addRoundedRect(rect, 18, 18)
        p.setClipPath(path)
        base = QLinearGradient(0, 0, 0, h)
        base.setColorAt(0.0, QColor("#071624"))
        base.setColorAt(0.55, QColor("#061321"))
        base.setColorAt(1.0, QColor("#050e19"))
        p.fillRect(rect, base)

        # stars
        p.setPen(Qt.NoPen)
        seed = 1337
        for i in range(90):
            seed = (seed * 1103515245 + 12345) & 0x7FFFFFFF
            x = (seed >> 7) % max(1, w)
            y = (seed >> 3) % max(1, int(h * 0.8))
            alpha = 26 + (seed >> 11) % 90
            p.setBrush(QColor(255, 255, 255, alpha))
            size = 0.7 if i % 5 else 1.2
            p.drawEllipse(QPointF(x, y), size, size)

        # planet limb: a huge globe just above the banner, only its lit edge showing
        radius = w * 0.78
        cx = w * 1.24
        cy = h * 1.08 - radius
        haze = QRadialGradient(QPointF(cx - radius * 0.55, cy + radius * 1.05), radius * 0.9)
        haze.setColorAt(0.0, QColor(56, 189, 248, 54))
        haze.setColorAt(1.0, QColor(56, 189, 248, 0))
        p.setBrush(haze)
        p.setPen(Qt.NoPen)
        p.drawEllipse(QPointF(cx, cy), radius * 1.16, radius * 1.16)

        globe = QRadialGradient(QPointF(cx - radius * 0.8, cy + radius * 1.25), radius * 1.5)
        globe.setColorAt(0.0, QColor("#2a6f9e"))
        globe.setColorAt(0.3, QColor("#123a58"))
        globe.setColorAt(0.62, QColor("#0a1d2e"))
        globe.setColorAt(1.0, QColor("#050d16"))
        p.setBrush(globe)
        p.drawEllipse(QPointF(cx, cy), radius, radius)

        p.setBrush(Qt.NoBrush)
        p.setPen(QPen(QColor(120, 214, 255, 120), 2.0))
        p.drawEllipse(QPointF(cx, cy), radius - 1.5, radius - 1.5)
        p.setPen(QPen(QColor(56, 189, 248, 46), 9))
        p.drawEllipse(QPointF(cx, cy), radius + 5, radius + 5)
        p.setPen(QPen(QColor(56, 189, 248, 22), 22))
        p.drawEllipse(QPointF(cx, cy), radius + 16, radius + 16)
        p.setClipping(False)
        p.setPen(QPen(QColor("#123049"), 1))
        p.setBrush(Qt.NoBrush)
        p.drawRoundedRect(rect, 18, 18)


class DashStat(QFrame):
    """One of the four overview cards."""

    def __init__(self, caption, icon, color=ACCENT, parent=None):
        super().__init__(parent)
        self.setObjectName("card")
        self.setFixedHeight(104)
        self._color = color
        lay = QHBoxLayout(self)
        lay.setContentsMargins(16, 14, 14, 12)
        lay.setSpacing(12)
        lay.addWidget(_tile(icon, color, 40), 0, Qt.AlignTop)
        col = QVBoxLayout()
        col.setSpacing(0)
        self.val = QLabel("0")
        self.val.setStyleSheet("font-size:24px;font-weight:800;letter-spacing:-0.5px;")
        self.cap = QLabel(caption)
        self.cap.setStyleSheet(f"color:{MUTED};font-size:12px")
        col.addWidget(self.val)
        col.addWidget(self.cap)
        lay.addLayout(col)
        lay.addStretch(1)
        right = QVBoxLayout()
        right.setSpacing(2)
        self.delta = QLabel("")
        self.delta.setAlignment(Qt.AlignRight)
        self.delta.setStyleSheet(f"color:{GREEN};font-size:11px;font-weight:800")
        self.spark = Sparkline(color)
        right.addWidget(self.delta)
        right.addWidget(self.spark)
        lay.addLayout(right)
        self.setAttribute(Qt.WA_StyledBackground, True)

    def set(self, value, delta="", trend=0.0, data=None):
        self.val.setText(str(value))
        self.delta.setText(delta)
        self.delta.setStyleSheet(
            f"color:{GREEN if trend >= 0 else RED};font-size:11px;font-weight:800")
        if data:
            self.spark.set_data(data)

    def paintEvent(self, e):
        super().paintEvent(e)
        p = QPainter(self)
        p.setRenderHint(QPainter.Antialiasing)
        rect = QRectF(self.rect()).adjusted(1, 1, -1, -1)
        path = QPainterPath()
        path.addRoundedRect(rect, 14, 14)
        p.setClipPath(path)
        glow = QColor(self._color)
        glow.setAlpha(40)
        grad = QRadialGradient(rect.width() - 12, -18, max(110.0, rect.width()))
        grad.setColorAt(0.0, glow)
        grad.setColorAt(1.0, QColor(0, 0, 0, 0))
        p.fillPath(path, grad)


class StatusCard(QFrame):
    """The green health card from the reference."""

    def __init__(self, parent=None):
        super().__init__(parent)
        self.setObjectName("card")
        self.setFixedHeight(104)
        lay = QHBoxLayout(self)
        lay.setContentsMargins(16, 14, 16, 14)
        lay.setSpacing(12)
        self.tile = _tile("✓", GREEN, 44, 12)
        lay.addWidget(self.tile)
        col = QVBoxLayout()
        col.setSpacing(2)
        self.title = QLabel("All good!")
        self.title.setStyleSheet("font-size:16px;font-weight:800")
        self.sub = QLabel("Everything is backed up.")
        self.sub.setObjectName("sub")
        self.sub.setWordWrap(True)
        col.addWidget(self.title)
        col.addWidget(self.sub)
        lay.addLayout(col, 1)

    def set_state(self, ok, title, sub):
        color = GREEN if ok else "#fbbf24"
        self.title.setText(title)
        self.sub.setText(sub)
        self.tile.setText("✓" if ok else "!")
        self.tile.setStyleSheet(
            f"background:{QColor(color).darker(430).name()};color:{color};"
            f"border:1px solid {QColor(color).darker(320).name()};border-radius:12px;"
            "font-size:19px")


class TimelineStrip(QWidget):
    """Thumbnails of the latest moments on a glowing rail."""

    clicked = Signal(str)

    def __init__(self, parent=None):
        super().__init__(parent)
        self.pid = None
        self.snaps = []
        self.previews = {}
        self.selected = None
        self.setMinimumHeight(214)
        self.setMouseTracking(True)
        self.setCursor(Qt.PointingHandCursor)
        self.setSizePolicy(QSizePolicy.Expanding, QSizePolicy.Expanding)

    def set_snaps(self, pid, chain, selected=None):
        self.pid = pid
        self.snaps = list(chain[-4:])
        before = chain[-5] if len(chain) > len(self.snaps) else None
        self.previews = {}
        for snap in self.snaps:
            self.previews[snap["id"]] = snapshot_preview(pid, snap, previous=before)
            before = snap
        self.selected = selected or (self.snaps[-1]["id"] if self.snaps else None)
        self.update()

    def _rects(self):
        n = len(self.snaps)
        if not n:
            return []
        gap = 12.0
        margin = 2.0
        cw = (self.width() - margin * 2 - gap * (n - 1)) / n
        ch = self.height() - 38
        return [(QRectF(margin + i * (cw + gap), 0, cw, ch), s) for i, s in enumerate(self.snaps)]

    def paintEvent(self, e):
        p = QPainter(self)
        p.setRenderHint(QPainter.Antialiasing)
        rects = self._rects()
        if not rects:
            p.setPen(QColor(MUTED))
            f = QFont()
            f.setPixelSize(13)
            p.setFont(f)
            p.drawText(self.rect(), Qt.AlignCenter, "Capture a snapshot to start the timeline.")
            return
        line_y = self.height() - 24
        rail = QPen(QColor("#1c4666"), 2)
        p.setPen(rail)
        first = rects[0][0].center().x()
        last = rects[-1][0].center().x()
        p.drawLine(QPointF(first, line_y), QPointF(last, line_y))

        for card, snap in rects:
            chosen = snap["id"] == self.selected
            path = QPainterPath()
            path.addRoundedRect(card, 12, 12)
            p.save()
            p.setClipPath(path)
            grad = QLinearGradient(card.x(), card.y(), card.x(), card.bottom())
            grad.setColorAt(0.0, QColor("#0d2030" if chosen else "#0a1826"))
            grad.setColorAt(1.0, QColor("#081320"))
            p.fillRect(card, grad)
            p.restore()
            p.setPen(QPen(QColor(ACCENT if chosen else BORDER), 1.4 if chosen else 1))
            p.setBrush(Qt.NoBrush)
            p.drawRoundedRect(card, 12, 12)

            f = QFont()
            f.setPixelSize(12)
            f.setBold(True)
            p.setFont(f)
            p.setPen(QColor(TEXT))
            p.drawText(QRectF(card.x() + 11, card.y() + 8, card.width() - 22, 16),
                       Qt.AlignLeft | Qt.AlignVCenter, _day(snap["ts"]))
            f2 = QFont()
            f2.setPixelSize(11)
            p.setFont(f2)
            p.setPen(QColor(MUTED))
            p.drawText(QRectF(card.x() + 11, card.y() + 24, card.width() - 22, 14),
                       Qt.AlignLeft | Qt.AlignVCenter, _clock(snap["ts"]))

            thumb = QRectF(card.x() + 8, card.y() + 42, card.width() - 16, card.height() - 50)
            if thumb.height() > 12:
                preview = self.previews.get(snap["id"])
                if preview:
                    paint_preview(p, thumb, preview, 8)
                else:
                    p.setPen(Qt.NoPen)
                    p.setBrush(QColor("#08141f"))
                    p.drawRoundedRect(thumb, 8, 8)

            cx = card.center().x()
            paint_dot(p, cx, line_y, 4.6 if chosen else 4.0,
                      ACCENT if chosen else "#6f93b8", glow=chosen)

    def mouseReleaseEvent(self, e):
        if e.button() == Qt.LeftButton:
            for card, snap in self._rects():
                if card.adjusted(0, 0, 0, 34).contains(e.position()):
                    self.clicked.emit(snap["id"])
                    break
        super().mouseReleaseEvent(e)


class RecentRow(ClickFrame):
    def __init__(self, snap, parent=None):
        super().__init__(parent)
        self.sid = snap["id"]
        self.setFixedHeight(56)
        self.setStyleSheet(
            "QFrame{background:transparent;border:1px solid transparent;border-radius:10px}"
            "QFrame:hover{background:#0e2135;border-color:#17344d}")
        lay = QHBoxLayout(self)
        lay.setContentsMargins(4, 6, 8, 6)
        lay.setSpacing(11)
        lay.addWidget(SnapshotBadge(snap))
        col = QVBoxLayout()
        col.setSpacing(1)
        title = QLabel(f"{_day(snap['ts'])}, {_clock(snap['ts'])}")
        title.setStyleSheet("font-size:12.5px;font-weight:700")
        note = QLabel(snap.get("note", "").strip() or snap.get("title", "").strip() or "No note")
        note.setStyleSheet(f"color:{MUTED};font-size:11.5px")
        note.setMaximumWidth(330)
        fm = note.fontMetrics()
        note.setText(fm.elidedText(note.text(), Qt.ElideRight, 300))
        col.addWidget(title)
        col.addWidget(note)
        lay.addLayout(col, 1)
        kind = "auto" if snap.get("kind") == "auto" else "manual"
        lay.addWidget(chip("Auto-save" if kind == "auto" else "manual",
                           "#8ea6c2" if kind == "auto" else ACCENT))
        branch = snap.get("branch") or "main"
        if branch != "main":
            lay.addWidget(chip(f"⑃ {branch}", VIOLET))
        arrow = QLabel("›")
        arrow.setStyleSheet(f"color:{DIM};font-size:16px")
        lay.addWidget(arrow)


class ActionRow(ClickFrame):
    def __init__(self, icon, text, hint, parent=None):
        super().__init__(parent)
        self.setObjectName("quickrow")
        self.setFixedHeight(40)
        lay = QHBoxLayout(self)
        lay.setContentsMargins(10, 0, 10, 0)
        lay.setSpacing(11)
        glyph = QLabel(icon)
        glyph.setStyleSheet(f"color:{ACCENT};font-size:15px")
        glyph.setFixedWidth(18)
        glyph.setAlignment(Qt.AlignCenter)
        name = QLabel(text)
        name.setStyleSheet("font-size:12.5px;font-weight:700;color:#dbe9f8")
        key = QLabel(hint)
        key.setStyleSheet(f"color:{DIM};font-size:10.5px")
        lay.addWidget(glyph)
        lay.addWidget(name)
        lay.addStretch(1)
        lay.addWidget(key)


class Dashboard(QWidget):
    new_snapshot = Signal()
    compare = Signal()
    restore = Signal()
    open_timeline = Signal(str)
    open_settings = Signal()

    def __init__(self, parent=None):
        super().__init__(parent)
        self.pid = None
        self.branch = "main"
        self.chain = []
        self.query = ""
        self._dirty = {}
        self._overview = None

        outer = QVBoxLayout(self)
        outer.setContentsMargins(0, 0, 0, 0)
        outer.setSpacing(0)
        scroll = QScrollArea()
        scroll.setWidgetResizable(True)
        scroll.setFrameShape(QFrame.NoFrame)
        scroll.setHorizontalScrollBarPolicy(Qt.ScrollBarAlwaysOff)
        body = QWidget()
        root = QVBoxLayout(body)
        root.setContentsMargins(0, 0, 6, 0)
        root.setSpacing(12)
        scroll.setWidget(body)
        outer.addWidget(scroll)

        stats = hbox(spacing=12)
        self.c_snaps = DashStat("Total Snapshots", "▤", ACCENT)
        self.c_projs = DashStat("Tracked Projects", "▢", "#60a5fa")
        self.c_space = DashStat("Used Space", "◍", "#a78bfa")
        self.c_state = StatusCard()
        for c in (self.c_snaps, self.c_projs, self.c_space, self.c_state):
            stats.addWidget(c)
        root.addLayout(stats)

        mid = hbox(spacing=12)
        tcard = QFrame()
        tcard.setObjectName("card")
        tl = QVBoxLayout(tcard)
        tl.setContentsMargins(16, 12, 16, 8)
        tl.setSpacing(4)
        thead = hbox(spacing=8)
        ticon = QLabel("◷")
        ticon.setStyleSheet(f"color:{ACCENT};font-size:15px")
        ttitle = QLabel("Timeline")
        ttitle.setStyleSheet("font-size:14.5px;font-weight:800")
        view_all = QPushButton("View all →")
        view_all.setObjectName("link")
        view_all.setCursor(Qt.PointingHandCursor)
        view_all.clicked.connect(lambda: self.open_timeline.emit(""))
        thead.addWidget(ticon)
        thead.addWidget(ttitle)
        thead.addStretch(1)
        thead.addWidget(view_all)
        tl.addLayout(thead)
        self.strip = TimelineStrip()
        self.strip.clicked.connect(self.open_timeline.emit)
        tl.addWidget(self.strip, 1)
        mid.addWidget(tcard, 1)

        ccard = QFrame()
        ccard.setObjectName("card")
        ccard.setFixedWidth(352)
        cl = QVBoxLayout(ccard)
        cl.setContentsMargins(16, 12, 16, 14)
        cl.setSpacing(8)
        chead = hbox(spacing=8)
        cicon = QLabel("▣")
        cicon.setStyleSheet(f"color:{ACCENT};font-size:15px")
        ctitle = QLabel("Current State")
        ctitle.setStyleSheet("font-size:14.5px;font-weight:800")
        chead.addWidget(cicon)
        chead.addWidget(ctitle)
        chead.addStretch(1)
        cl.addLayout(chead)
        self.state_preview = SnapshotPreview(9)
        self.state_preview.setMinimumHeight(112)
        cl.addWidget(self.state_preview, 1)
        self.facts = QVBoxLayout()
        self.facts.setSpacing(5)
        cl.addLayout(self.facts)
        mid.addWidget(ccard)
        root.addLayout(mid, 1)

        bottom = hbox(spacing=12)
        rcard = QFrame()
        rcard.setObjectName("card")
        rl = QVBoxLayout(rcard)
        rl.setContentsMargins(16, 12, 16, 12)
        rl.setSpacing(4)
        rhead = hbox(spacing=8)
        ricon = QLabel("★")
        ricon.setStyleSheet("color:#fbbf24;font-size:14px")
        rtitle = QLabel("Recent Snapshots")
        rtitle.setStyleSheet("font-size:14.5px;font-weight:800")
        rview = QPushButton("View all →")
        rview.setObjectName("link")
        rview.setCursor(Qt.PointingHandCursor)
        rview.clicked.connect(lambda: self.open_timeline.emit(""))
        rhead.addWidget(ricon)
        rhead.addWidget(rtitle)
        rhead.addStretch(1)
        rhead.addWidget(rview)
        rl.addLayout(rhead)
        self.recent = QVBoxLayout()
        self.recent.setSpacing(0)
        rl.addLayout(self.recent, 1)
        bottom.addWidget(rcard, 1)

        wcard = QFrame()
        wcard.setObjectName("card")
        wcard.setFixedWidth(276)
        wl = QVBoxLayout(wcard)
        wl.setContentsMargins(16, 12, 16, 12)
        wl.setSpacing(7)
        whead = hbox(spacing=8)
        wicon = QLabel("◫")
        wicon.setStyleSheet(f"color:{ACCENT};font-size:14px")
        wtitle = QLabel("What Changed?")
        wtitle.setStyleSheet("font-size:14.5px;font-weight:800")
        whead.addWidget(wicon)
        whead.addWidget(wtitle)
        whead.addStretch(1)
        wl.addLayout(whead)
        self.changed = QVBoxLayout()
        self.changed.setSpacing(6)
        wl.addLayout(self.changed)
        wl.addStretch(1)
        self.details = QPushButton("View details →")
        self.details.setObjectName("ghost")
        self.details.setCursor(Qt.PointingHandCursor)
        self.details.setMinimumHeight(32)
        self.details.clicked.connect(lambda: self.open_timeline.emit(""))
        wl.addWidget(self.details)
        bottom.addWidget(wcard)

        acard = QFrame()
        acard.setObjectName("card")
        acard.setFixedWidth(312)
        al = QVBoxLayout(acard)
        al.setContentsMargins(16, 12, 16, 12)
        al.setSpacing(2)
        ahead = hbox(spacing=8)
        aicon = QLabel("⚡")
        aicon.setStyleSheet("color:#fbbf24;font-size:14px")
        atitle = QLabel("Quick Actions")
        atitle.setStyleSheet("font-size:14.5px;font-weight:800")
        ahead.addWidget(aicon)
        ahead.addWidget(atitle)
        ahead.addStretch(1)
        al.addLayout(ahead)
        al.addSpacing(4)
        for icon, text, hint, slot in (
            ("◉", "Create Snapshot", "Ctrl + S", self.new_snapshot.emit),
            ("⚖", "Compare Snapshots", "Ctrl + Shift + C", self.compare.emit),
            ("↺", "Restore Snapshot", "Ctrl + R", self.restore.emit),
            ("◷", "Open Timeline", "Ctrl + T", lambda: self.open_timeline.emit("")),
        ):
            row = ActionRow(icon, text, hint)
            row.clicked.connect(slot)
            al.addWidget(row)
        al.addStretch(1)
        bottom.addWidget(acard)
        root.addLayout(bottom)
        root.addStretch(1)

    # ------------------------------------------------------------------ data
    def _clear(self, lay):
        while lay.count():
            item = lay.takeAt(0)
            w = item.widget()
            if w:
                w.deleteLater()
        return lay

    def set_query(self, text):
        self.query = (text or "").strip().lower()
        self._fill_recent()

    def _is_dirty(self, pid, branch):
        """Live-folder check, reused for a moment so page switches stay instant."""
        latest = self.chain[-1]["id"] if self.chain else None
        hit = self._dirty.get(pid)
        now = time.time()
        if hit and hit[0] == latest and now - hit[1] < 15:
            return hit[2]
        try:
            dirty = store.has_changes(pid, branch)
        except store.StoreError:
            dirty = False
        self._dirty[pid] = (latest, now, dirty)
        return dirty

    def load(self, pid, branch="main", overview=None):
        self.pid = pid
        self.branch = branch
        self.chain = store.branch_chain(pid, branch) if pid else []
        projects = store.load_projects()
        stats = overview or store.library_stats()
        self._overview = stats
        counts = stats["counts"]
        total_snaps = sum(counts.values())
        used = stats["used"]
        per_day = stats["per_day"]
        today = time.time()
        series = []
        for back in range(7, -1, -1):
            key = store.day_key(today - back * 86400)
            series.append(per_day.get(key, 0))
        recent_sum = sum(series[-3:])
        earlier_sum = sum(series[:-3]) or 1
        pct = round((recent_sum - earlier_sum) / earlier_sum * 100)
        self.c_snaps.set(total_snaps, f"↑ +{pct}%" if pct >= 0 else f"↓ {pct}%", pct, series)
        new_projects = sum(1 for p in projects if p.get("created", 0) > today - 7 * 86400)
        self.c_projs.set(len(projects),
                         f"↑ +{new_projects}" if new_projects else "stable",
                         1 if new_projects else 0,
                         [max(0, len(projects) - (6 - i)) for i in range(7)])
        sizes = [sum(f["size"] for f in s["files"].values()) for s in self.chain[-8:]]
        if not sizes:
            sizes = [0, 0]
        space_delta = 0
        if len(sizes) > 1 and sizes[0]:
            space_delta = round((sizes[-1] - sizes[0]) / sizes[0] * 100)
        self.c_space.set(store.fmt_bytes(used),
                         f"↑ +{space_delta}%" if space_delta >= 0 else f"↓ {space_delta}%",
                         space_delta, sizes)

        if not pid:
            self.c_state.set_state(False, "No project yet", "Add a folder to start capturing.")
        else:
            dirty = self._is_dirty(pid, branch)
            if dirty:
                self.c_state.set_state(False, "Changes pending",
                                       "The folder changed since the last snapshot.")
            else:
                self.c_state.set_state(True, "All good!", "Everything is backed up.")

        self.strip.set_snaps(self.pid, self.chain,
                             self.chain[-1]["id"] if self.chain else None)
        self._fill_current()
        self._fill_recent()
        self._fill_changed()

    def _fill_current(self):
        self._clear(self.facts)
        latest = self.chain[-1] if self.chain else None
        prev = self.chain[-2] if len(self.chain) > 1 else None
        self.state_preview.set_snapshot(self.pid, latest, prev)
        if not self.pid:
            for name, value in (("OS", platform.system()), ("Kernel", platform.release()),
                                ("Uptime", _uptime())):
                self.facts.addLayout(self._fact(name, value))
            return
        proj = store.get_project(self.pid) or {}
        if latest:
            facts = [
                ("OS", platform.system()),
                ("Kernel", platform.release()),
                ("Uptime", _uptime()),
            ]
        else:
            facts = [
                ("Folder", proj.get("path", "—")),
                ("Captured", "nothing yet"),
                ("Uptime", _uptime()),
            ]
        for name, value in facts:
            self.facts.addLayout(self._fact(name, value))

    def _fact(self, name, value):
        row = QHBoxLayout()
        row.setSpacing(8)
        cap = QLabel(name.upper())
        cap.setStyleSheet(f"color:{DIM};font-size:10px;font-weight:800;letter-spacing:0.8px")
        cap.setFixedWidth(62)
        val = QLabel(value)
        val.setStyleSheet(f"color:{TEXT};font-size:12px")
        fm = val.fontMetrics()
        val.setText(fm.elidedText(value, Qt.ElideMiddle, 226))
        val.setToolTip(value)
        row.addWidget(cap)
        row.addWidget(val, 1)
        return row

    def _fill_recent(self):
        self._clear(self.recent)
        rows = list(reversed(self.chain[-6:]))
        if self.query:
            rows = [s for s in rows
                    if self.query in (s.get("note", "") + " " + s.get("title", "")).lower()]
        if not rows:
            empty = QLabel("No snapshots match that search." if self.query
                           else "No snapshots yet — capture the first moment.")
            empty.setObjectName("empty")
            empty.setStyleSheet("padding:18px 4px")
            self.recent.addWidget(empty)
            return
        for snap in rows[:4]:
            row = RecentRow(snap)
            row.clicked.connect(self.open_timeline.emit)
            self.recent.addWidget(row)

    def _fill_changed(self):
        self._clear(self.changed)
        if not self.chain:
            msg = QLabel("Nothing captured yet.")
            msg.setObjectName("empty")
            self.changed.addWidget(msg)
            return
        last = self.chain[-1]
        prev = self.chain[-2] if len(self.chain) > 1 else None
        if prev:
            counts = store.compare(prev, last)["counts"]
            added, deleted, modified = counts["added"], counts["deleted"], counts["modified"]
        else:
            added, deleted, modified = last.get("count", 0), 0, 0
        for glyph, color, count, name in (
            ("+", GREEN, added, "files added"),
            ("−", RED, deleted, "files removed"),
            ("▤", ACCENT, modified, "files modified"),
        ):
            row = QHBoxLayout()
            row.setSpacing(9)
            mark = QLabel(glyph)
            mark.setFixedWidth(12)
            mark.setAlignment(Qt.AlignCenter)
            mark.setStyleSheet(f"color:{color};font-size:14px;font-weight:800")
            text = QLabel(f"{count} {name}")
            text.setStyleSheet("font-size:12px")
            row.addWidget(mark)
            row.addWidget(text, 1)
            self.changed.addLayout(row)
        self.changed.addSpacing(2)
        self.changed.addWidget(hline())
        cap = QLabel("MOST CHANGED PROJECT")
        cap.setObjectName("tiny")
        self.changed.addWidget(cap)
        proj = store.get_project(self.pid) or {}
        row = QHBoxLayout()
        row.setSpacing(8)
        icon = QLabel("▢")
        icon.setStyleSheet(f"color:{ACCENT};font-size:14px")
        name = QLabel(proj.get("name", "—"))
        name.setStyleSheet("font-size:12px;font-weight:700")
        changes = added + deleted + modified
        tag = QLabel(f"+{changes} changes")
        tag.setStyleSheet(f"color:{GREEN};font-size:11px;font-weight:800")
        row.addWidget(icon)
        row.addWidget(name, 1)
        row.addWidget(tag)
        self.changed.addLayout(row)


class StatusBar(QWidget):
    settings_clicked = Signal()
    autosave_toggled = Signal(bool)

    def __init__(self, parent=None):
        super().__init__(parent)
        self.setObjectName("statusbar")
        self.setFixedHeight(46)
        lay = QHBoxLayout(self)
        lay.setContentsMargins(18, 0, 18, 0)
        lay.setSpacing(10)
        dot = QLabel("●")
        dot.setStyleSheet(f"color:{GREEN};font-size:12px")
        self.ready = QLabel("Ready")
        self.ready.setObjectName("sbstrong")
        lay.addWidget(dot)
        lay.addWidget(self.ready)
        lay.addSpacing(18)
        auto = QLabel("Auto snapshots:")
        auto.setObjectName("sblabel")
        self.toggle = ToggleSwitch()
        self.toggle.toggled.connect(self.autosave_toggled.emit)
        self.autostate = QLabel("OFF")
        self.autostate.setObjectName("sbstrong")
        lay.addWidget(auto)
        lay.addWidget(self.toggle)
        lay.addWidget(self.autostate)
        lay.addSpacing(16)
        clock = QLabel("◷")
        clock.setStyleSheet(f"color:{DIM};font-size:13px")
        self.nextlab = QLabel("Next snapshot: —")
        self.nextlab.setObjectName("sblabel")
        lay.addWidget(clock)
        lay.addWidget(self.nextlab)
        lay.addStretch(1)
        self.storage = QLabel("")
        self.storage.setObjectName("sblabel")
        lay.addWidget(self.storage)
        self.bar = MiniBar(0.0, "#3b82f6")
        self.bar.setFixedWidth(120)
        lay.addWidget(self.bar)
        gear = QPushButton("⚙")
        gear.setObjectName("ghost")
        gear.setCursor(Qt.PointingHandCursor)
        gear.setFixedSize(30, 30)
        gear.setStyleSheet(
            "QPushButton{background:transparent;border:1px solid transparent;border-radius:8px;"
            f"color:{MUTED};font-size:14px;padding:0}}"
            f"QPushButton:hover{{background:#0e2135;border-color:#1b3f5c;color:{TEXT}}}")
        gear.clicked.connect(self.settings_clicked.emit)
        lay.addWidget(gear)

    def update_state(self, pid, cfg, remaining_ms, last_auto=None):
        self.toggle.setChecked(cfg.get("enabled"))
        self.autostate.setText("ON" if cfg.get("enabled") else "OFF")
        if cfg.get("enabled"):
            mins = max(0, remaining_ms) / 60000 if remaining_ms and remaining_ms > 0 else cfg["minutes"]
            if mins >= 60:
                self.nextlab.setText(f"Next snapshot: in {int(mins // 60)}h {int(mins % 60)}m")
            else:
                self.nextlab.setText(f"Next snapshot: in {max(1, int(round(mins)))} min")
        else:
            self.nextlab.setText("Next snapshot: paused")
        if not pid:
            self.storage.setText("No project storage yet")
            self.bar.frac = 0.0
            self.bar.update()
            return
        used = store.storage_bytes(pid)
        chain = store.branch_chain(pid, store.active_branch(pid))
        captured = sum(s.get("bytes", 0) for s in chain) or 0
        self.storage.setText(f"{store.fmt_bytes(used)} / {store.fmt_bytes(captured)}")
        self.bar.frac = 0.0 if not captured else max(0.02, min(1.0, used / captured))
        self.bar.update()
