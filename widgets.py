"""Reusable interface pieces: cards, pills, the history chart and empty states."""

import time

from PySide6.QtCore import QRectF, Qt, Signal
from PySide6.QtGui import QColor, QFont, QLinearGradient, QPainter, QPen
from PySide6.QtWidgets import QFrame, QLabel, QPushButton, QSizePolicy, QVBoxLayout, QWidget

from theme import (
    ACCENT, BG, BORDER, DIM, GREEN, MUTED, SURF, SURF2, TEXT, STATUS, accent_color,
    chip, hbox, label, paint_dot, status_label, vbox,
)


class ClickFrame(QFrame):
    """A card that behaves like a button."""

    clicked = Signal()

    def __init__(self, parent=None):
        super().__init__(parent)
        self.setCursor(Qt.PointingHandCursor)

    def mouseReleaseEvent(self, e):
        if e.button() == Qt.LeftButton and self.rect().contains(e.position().toPoint()):
            self.clicked.emit()
        super().mouseReleaseEvent(e)


class StatCard(QFrame):
    def __init__(self, caption, value, icon="", color=ACCENT, hint="", parent=None):
        super().__init__(parent)
        self.setObjectName("card")
        self._color = color
        lay = QVBoxLayout(self)
        lay.setContentsMargins(18, 16, 18, 16)
        lay.setSpacing(6)
        top = hbox(spacing=10)
        self.ico = QLabel(icon)
        self.ico.setStyleSheet(f"font-size:18px;color:{color}")
        self.val = QLabel(str(value))
        self.val.setObjectName("statval")
        self.val.setStyleSheet(f"color:{color}")
        top.addWidget(self.ico)
        top.addWidget(self.val)
        top.addStretch(1)
        lay.addLayout(top)
        self.cap = QLabel(caption.upper())
        self.cap.setObjectName("statcap")
        lay.addWidget(self.cap)
        self.hint = QLabel(hint)
        self.hint.setObjectName("sub")
        self.hint.setVisible(bool(hint))
        lay.addWidget(self.hint)
        self.setSizePolicy(QSizePolicy.Expanding, QSizePolicy.Fixed)

    def set(self, value, hint=None):
        self.val.setText(str(value))
        if hint is not None:
            self.hint.setText(hint)
            self.hint.setVisible(bool(hint))


class StatusPill(QLabel):
    def __init__(self, kind, parent=None):
        super().__init__(status_label(kind), parent)
        color = accent_color(kind)
        self.setStyleSheet(
            f"background:{QColor(color).darker(320).name()};color:{color};"
            "border-radius:8px;padding:2px 8px;font-size:11px;font-weight:800;letter-spacing:0.5px"
        )
        self.setAlignment(Qt.AlignCenter)


class BarChart(QWidget):
    """Compact bar chart for snapshots per day."""

    def __init__(self, parent=None):
        super().__init__(parent)
        self.data = []
        self.setMinimumHeight(150)
        self.setSizePolicy(QSizePolicy.Expanding, QSizePolicy.Expanding)

    def set_data(self, data):
        self.data = list(data)
        self.update()

    def paintEvent(self, e):
        p = QPainter(self)
        p.setRenderHint(QPainter.Antialiasing)
        w, h = self.width(), self.height()
        if not self.data:
            p.setPen(QColor(MUTED))
            p.drawText(self.rect(), Qt.AlignCenter, "No history captured yet")
            return
        pad_b, pad_t = 26, 12
        top = pad_t
        bottom = h - pad_b
        values = [v for _l, v in self.data]
        peak = max(values) or 1
        step = w / max(1, len(self.data))
        bar_w = min(30.0, step * 0.52)
        for i in range(4):
            y = top + (bottom - top) * i / 3
            p.setPen(QPen(QColor("#191c22"), 1))
            p.drawLine(int(0), int(y), int(w), int(y))
        f = QFont()
        f.setPixelSize(10)
        p.setFont(f)
        for i, (name, value) in enumerate(self.data):
            cx = step * i + step / 2
            bh = (bottom - top) * (value / peak) if peak else 0
            rect = QRectF(cx - bar_w / 2, bottom - bh, bar_w, bh)
            if value:
                grad = QLinearGradient(0, bottom - bh, 0, bottom)
                grad.setColorAt(0, QColor(ACCENT))
                grad.setColorAt(1, QColor("#8a5b1c"))
                p.setBrush(grad)
            else:
                p.setBrush(QColor("#1b1e25"))
            p.setPen(Qt.NoPen)
            p.drawRoundedRect(rect, 4, 4)
            if value and peak and bh > 16:
                p.setPen(QColor("#1a1206"))
                p.drawText(QRectF(rect.x(), rect.y(), rect.width(), 14), Qt.AlignCenter, str(value))
            p.setPen(QColor(DIM if i % 2 else MUTED))
            p.drawText(QRectF(cx - step / 2, bottom + 6, step, 14), Qt.AlignCenter, name)


class ProjectTimeline(QWidget):
    """All moments of a project as lines; a branch forks off the moment it started."""

    selected = Signal(str)

    HEIGHT = 92
    TOP = 6
    BOTTOM = 20

    def __init__(self, parent=None):
        super().__init__(parent)
        self.snaps = []
        self.branches = [{"id": "main", "name": "main", "base": None}]
        self.active = "main"
        self.start = None
        self.end = None
        self.selected_id = None
        self.start_label = ""
        self.end_label = ""
        self._dots = []
        self.setFixedHeight(76)
        self.setMouseTracking(True)
        self.setCursor(Qt.PointingHandCursor)
        self.setSizePolicy(QSizePolicy.Expanding, QSizePolicy.Fixed)

    # ------------------------------------------------------------------ data
    def set_graph(self, snaps, branches, active="main", start=None, end=None,
                  selected=None, start_label="", end_label=""):
        self.snaps = list(snaps)
        self.branches = list(branches) or [{"id": "main", "name": "main", "base": None}]
        self.active = active or "main"
        self.start = start
        self.end = end
        self.start_label = start_label
        self.end_label = end_label
        if selected is not None:
            self.selected_id = selected
        self.setFixedHeight(self.HEIGHT)
        self.update()

    def set_selected(self, sid):
        if self.selected_id != sid:
            self.selected_id = sid
            self.update()

    def _lanes(self):
        ordered = sorted(self.branches, key=lambda b: (b["id"] != "main", b.get("created") or 0))
        return ordered, {b["id"]: i for i, b in enumerate(ordered)}

    def _lane_y(self, index, count):
        """Each branch sits in the middle of its own equal slice of the box."""
        span = max(1.0, self.height() - self.TOP - self.BOTTOM)
        return self.TOP + span * (index + 0.5) / max(1, count)

    @staticmethod
    def _branch_of(snap):
        return snap.get("branch") or "main"

    def _axis(self):
        left, right = 16.0, max(17.0, self.width() - 16.0)
        stamps = [float(s["ts"]) for s in self.snaps]
        t0 = float(self.start) if self.start is not None else (min(stamps) if stamps else 0.0)
        t1 = float(self.end) if self.end is not None else (max(stamps) if stamps else t0 + 1.0)
        if t1 <= t0:
            t1 = t0 + 1.0
        return left, right, t0, t1

    def _x(self, ts, left, right, t0, t1):
        frac = (float(ts) - t0) / (t1 - t0)
        return left + (right - left) * max(0.0, min(1.0, frac))

    def _dot_at(self, pos):
        best = None
        for x, y, s in self._dots:
            gap = ((x - pos.x()) ** 2 + (y - pos.y()) ** 2) ** 0.5
            if gap <= 12 and (best is None or gap < best[0]):
                best = (gap, s)
        return best[1] if best else None

    # -------------------------------------------------------------- painting
    def paintEvent(self, e):
        p = QPainter(self)
        p.setRenderHint(QPainter.Antialiasing)
        left, right, t0, t1 = self._axis()
        ordered, lane = self._lanes()
        self._dots = []
        f = QFont()
        f.setPixelSize(10)
        p.setFont(f)
        if not self.snaps:
            p.setPen(QColor(MUTED))
            p.drawText(self.rect(), Qt.AlignCenter, "Every capture you make lands on this line")
            return
        by_id = {s["id"]: s for s in self.snaps}
        count = len(ordered)
        ys = {b["id"]: self._lane_y(lane[b["id"]], count) for b in ordered}
        tick_y = ys.get("main", self._lane_y(0, count))
        p.setPen(QPen(QColor("#2f343e"), 1))
        for frac in (0.0, 0.5, 1.0):
            x = left + (right - left) * frac
            p.drawLine(int(x), int(tick_y - 5), int(x), int(tick_y + 5))

        for b in ordered:
            own = sorted((s for s in self.snaps if self._branch_of(s) == b["id"]),
                         key=lambda s: s["ts"])
            y = float(ys[b["id"]])
            is_active = b["id"] == self.active
            line_color = QColor("#5a616d" if is_active else "#2c313a")
            base = by_id.get(b.get("base") or "")
            if base is not None:
                bx = self._x(base["ts"], left, right, t0, t1)
                by = float(ys.get(self._branch_of(base), y))
                p.setPen(QPen(QColor("#1d2129"), 1))
                p.drawLine(int(left), int(y), int(bx), int(y))
                p.setPen(QPen(line_color, 2))
                p.drawLine(int(bx), int(by), int(bx + 16), int(y))
                p.drawLine(int(bx + 16), int(y), int(right), int(y))
            else:
                p.setPen(QPen(line_color, 2 if is_active else 1))
                p.drawLine(int(left), int(y), int(right), int(y))
            label = b["name"]
            tw = p.fontMetrics().horizontalAdvance(label) + 14
            rect = QRectF(right - tw, y - 7, tw, 14)
            p.fillRect(rect, QColor(BG))
            p.setPen(QColor(ACCENT if is_active else DIM))
            p.drawText(rect, Qt.AlignCenter, label)
            radius = 4.5 if len(own) <= 40 else 3.5
            for s in own:
                x = self._x(s["ts"], left, right, t0, t1)
                chosen = s["id"] == self.selected_id
                if chosen:
                    color = ACCENT
                elif is_active:
                    color = "#5a616d" if s.get("kind") == "auto" else "#8b929e"
                else:
                    color = "#3d434e"
                paint_dot(p, x, y, radius + (1.2 if chosen else 0), color, glow=chosen)
                self._dots.append((x, y, s))

        bottom = self.height() - 16
        p.setPen(QColor(DIM))
        if self.start_label:
            p.drawText(QRectF(left, bottom, 170, 14), Qt.AlignLeft | Qt.AlignVCenter, self.start_label)
        if self.end_label:
            p.drawText(QRectF(right - 170, bottom, 170, 14),
                       Qt.AlignRight | Qt.AlignVCenter, self.end_label)

    # ----------------------------------------------------------- interaction
    def _branch_name(self, snap):
        bid = self._branch_of(snap)
        return next((b["name"] for b in self.branches if b["id"] == bid), bid)

    def mouseMoveEvent(self, e):
        s = self._dot_at(e.position())
        if s is None:
            self.setToolTip("")
        else:
            kind = "Auto-save" if s.get("kind") == "auto" else "Snapshot"
            stamp = time.strftime("%d %b %Y · %H:%M", time.localtime(s["ts"]))
            self.setToolTip(f"{kind} · {stamp} · {self._branch_name(s)}")
        super().mouseMoveEvent(e)

    def mousePressEvent(self, e):
        if e.button() == Qt.LeftButton:
            s = self._dot_at(e.position())
            if s is not None:
                self.selected.emit(s["id"])
        super().mousePressEvent(e)


class MiniBar(QWidget):
    def __init__(self, frac, color=ACCENT, parent=None):
        super().__init__(parent)
        self.frac = max(0.0, min(1.0, frac))
        self.color = color
        self.setFixedHeight(6)
        self.setSizePolicy(QSizePolicy.Expanding, QSizePolicy.Fixed)

    def paintEvent(self, e):
        p = QPainter(self)
        p.setRenderHint(QPainter.Antialiasing)
        p.setPen(Qt.NoPen)
        p.setBrush(QColor("#181b21"))
        p.drawRoundedRect(QRectF(0, 0, self.width(), self.height()), 3, 3)
        p.setBrush(QColor(self.color))
        p.drawRoundedRect(QRectF(0, 0, max(3.0, self.width() * self.frac), self.height()), 3, 3)


class EmptyState(QWidget):
    clicked = Signal()

    def __init__(self, icon, title, text, button=None, parent=None):
        super().__init__(parent)
        lay = QVBoxLayout(self)
        lay.setContentsMargins(24, 24, 24, 24)
        lay.setSpacing(10)
        lay.addStretch(1)
        ico = QLabel(icon)
        ico.setAlignment(Qt.AlignCenter)
        ico.setStyleSheet("font-size:44px")
        t = QLabel(title)
        t.setAlignment(Qt.AlignCenter)
        t.setStyleSheet("font-size:19px;font-weight:800")
        s = QLabel(text)
        s.setAlignment(Qt.AlignCenter)
        s.setObjectName("empty")
        s.setWordWrap(True)
        lay.addWidget(ico)
        lay.addWidget(t)
        lay.addWidget(s)
        if button:
            b = QPushButton(button)
            b.setObjectName("primary")
            b.setCursor(Qt.PointingHandCursor)
            b.clicked.connect(self.clicked.emit)
            row = hbox(spacing=0)
            row.addStretch(1)
            row.addWidget(b)
            row.addStretch(1)
            lay.addSpacing(6)
            lay.addLayout(row)
            self.button = b
        lay.addStretch(1)


class Section(QWidget):
    """A titled block used across the stats and detail pages."""

    def __init__(self, title, subtitle="", parent=None):
        super().__init__(parent)
        self.lay = QVBoxLayout(self)
        self.lay.setContentsMargins(0, 0, 0, 0)
        self.lay.setSpacing(10)
        head = hbox(spacing=10)
        t = QLabel(title)
        t.setStyleSheet("font-size:16px;font-weight:800")
        head.addWidget(t)
        if subtitle:
            s = QLabel(subtitle)
            s.setObjectName("sub")
            head.addWidget(s)
        head.addStretch(1)
        self.lay.addLayout(head)
        self.body = QVBoxLayout()
        self.body.setSpacing(8)
        self.lay.addLayout(self.body)
