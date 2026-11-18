"""Reusable interface pieces: cards, pills, the history chart and empty states."""

import os
import time

from PySide6.QtCore import QPointF, QRectF, Qt, Signal
from PySide6.QtGui import (
    QColor, QFont, QLinearGradient, QPainter, QPainterPath, QPen, QRadialGradient,
)
from PySide6.QtWidgets import QFrame, QLabel, QPushButton, QSizePolicy, QSpinBox, QVBoxLayout, QWidget

import store
from theme import (
    ACCENT, BG, BORDER, BORDER2, DIM, GREEN, MUTED, SURF, SURF2, TEXT, STATUS,
    accent_color, chip, hbox, label, paint_dot, status_label, vbox,
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
        self.setAttribute(Qt.WA_StyledBackground, True)

    def set(self, value, hint=None):
        self.val.setText(str(value))
        if hint is not None:
            self.hint.setText(hint)
            self.hint.setVisible(bool(hint))

    def paintEvent(self, e):
        super().paintEvent(e)
        p = QPainter(self)
        p.setRenderHint(QPainter.Antialiasing)
        rect = QRectF(self.rect()).adjusted(1, 1, -1, -1)
        path = QPainterPath()
        path.addRoundedRect(rect, 14, 14)
        p.setClipPath(path)
        glow = QColor(self._color)
        glow.setAlpha(44)
        grad = QRadialGradient(rect.width() - 8, -14, max(90.0, rect.width()))
        grad.setColorAt(0.0, glow)
        grad.setColorAt(1.0, QColor(0, 0, 0, 0))
        p.fillPath(path, grad)
        p.setClipping(False)
        bar = QColor(self._color)
        bar.setAlpha(150)
        p.setPen(Qt.NoPen)
        p.setBrush(bar)
        p.drawRoundedRect(QRectF(18, 1, 24, 2), 1, 1)


class SpinBox(QSpinBox):
    """A spin box whose step arrows are drawn by hand, so they stay visible on the dark theme."""

    def paintEvent(self, e):
        super().paintEvent(e)
        p = QPainter(self)
        p.setRenderHint(QPainter.Antialiasing)
        pen = QPen(QColor(MUTED), 1.3)
        pen.setCapStyle(Qt.RoundCap)
        pen.setJoinStyle(Qt.RoundJoin)
        p.setPen(pen)
        cx = self.width() - 15
        cy = self.height() / 2
        p.drawPolyline([QPointF(cx - 4, cy - 3), QPointF(cx, cy - 7), QPointF(cx + 4, cy - 3)])
        p.drawPolyline([QPointF(cx - 4, cy + 3), QPointF(cx, cy + 7), QPointF(cx + 4, cy + 3)])


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
        pad_b, pad_t = 26, 16
        top = pad_t
        bottom = h - pad_b
        values = [v for _l, v in self.data]
        peak = max(values) or 1
        step = w / max(1, len(self.data))
        bar_w = min(26.0, step * 0.46)
        for i in range(4):
            y = top + (bottom - top) * i / 3
            p.setPen(QPen(QColor("#102234"), 1))
            p.drawLine(int(0), int(y), int(w), int(y))
        f = QFont()
        f.setPixelSize(10)
        p.setFont(f)
        for i, (name, value) in enumerate(self.data):
            cx = step * i + step / 2
            bh = (bottom - top) * (value / peak) if peak else 0
            if value:
                rect = QRectF(cx - bar_w / 2, bottom - bh, bar_w, bh)
                grad = QLinearGradient(0, bottom - bh, 0, bottom)
                grad.setColorAt(0, QColor("#7dd3fc"))
                grad.setColorAt(0.55, QColor(ACCENT))
                grad.setColorAt(1, QColor("#1667c9"))
                p.setBrush(grad)
                p.setPen(Qt.NoPen)
                p.drawRoundedRect(rect, 4, 4)
                p.setPen(QColor(MUTED))
                p.drawText(QRectF(rect.x() - 6, rect.y() - 15, rect.width() + 12, 14),
                           Qt.AlignCenter, str(value))
            else:
                p.setBrush(QColor("#102234"))
                p.setPen(Qt.NoPen)
                p.drawRoundedRect(QRectF(cx - bar_w / 2, bottom - 3, bar_w, 3), 1.5, 1.5)
            p.setPen(QColor(DIM if i % 2 else MUTED))
            p.drawText(QRectF(cx - step / 2, bottom + 6, step, 14), Qt.AlignCenter, name)
        p.setPen(QPen(QColor("#17364f"), 1))
        p.drawLine(int(0), int(bottom), int(w), int(bottom))


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
        p.setPen(QPen(QColor("#1c405e"), 1))
        for frac in (0.0, 0.5, 1.0):
            x = left + (right - left) * frac
            p.drawLine(int(x), int(tick_y - 5), int(x), int(tick_y + 5))

        for b in ordered:
            own = sorted((s for s in self.snaps if self._branch_of(s) == b["id"]),
                         key=lambda s: s["ts"])
            y = float(ys[b["id"]])
            is_active = b["id"] == self.active
            line_color = QColor("#6f93b8" if is_active else "#1b3b57")
            base = by_id.get(b.get("base") or "")
            if base is not None:
                bx = self._x(base["ts"], left, right, t0, t1)
                by = float(ys.get(self._branch_of(base), y))
                p.setPen(QPen(QColor("#132c44"), 1))
                p.drawLine(int(left), int(y), int(bx), int(y))
                p.setPen(QPen(line_color, 2))
                p.drawLine(int(bx), int(by), int(bx + 16), int(y))
                p.drawLine(int(bx + 16), int(y), int(right), int(y))
            else:
                p.setPen(QPen(line_color, 2 if is_active else 1))
                p.drawLine(int(left), int(y), int(right), int(y))
            label = b["name"]
            tw = p.fontMetrics().horizontalAdvance(label) + 18
            rect = QRectF(right - tw, y - 8, tw, 16)
            p.setPen(QPen(QColor("#1a3b58" if is_active else "#173650"), 1))
            p.setBrush(QColor("#102a42" if is_active else "#071523"))
            p.drawRoundedRect(rect, 8, 8)
            p.setPen(QColor(ACCENT if is_active else DIM))
            p.drawText(rect, Qt.AlignCenter, label)
            radius = 4.5 if len(own) <= 40 else 3.5
            for s in own:
                x = self._x(s["ts"], left, right, t0, t1)
                chosen = s["id"] == self.selected_id
                if chosen:
                    color = ACCENT
                elif is_active:
                    color = "#6f93b8" if s.get("kind") == "auto" else "#a8c0d8"
                else:
                    color = "#34597a"
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
        p.setBrush(QColor("#0f2233"))
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
        ico.setFixedSize(84, 84)
        ico.setStyleSheet(
            "background:qlineargradient(x1:0,y1:0,x2:0,y2:1,stop:0 #28313f,stop:1 #161d28);"
            f"border:1px solid {BORDER2};border-radius:26px;font-size:36px"
        )
        ico_row = hbox(spacing=0)
        ico_row.addStretch(1)
        ico_row.addWidget(ico)
        ico_row.addStretch(1)
        t = QLabel(title)
        t.setAlignment(Qt.AlignCenter)
        t.setStyleSheet("font-size:19px;font-weight:800;margin-top:6px")
        s = QLabel(text)
        s.setAlignment(Qt.AlignCenter)
        s.setObjectName("empty")
        s.setWordWrap(True)
        s.setMaximumWidth(430)
        txt_row = hbox(spacing=0)
        txt_row.addStretch(1)
        txt_row.addWidget(s)
        txt_row.addStretch(1)
        lay.addLayout(ico_row)
        lay.addWidget(t)
        lay.addLayout(txt_row)
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


# --------------------------------------------------------------- dashboard bits
TEXT_EXT = {
    ".py", ".pyi", ".js", ".jsx", ".ts", ".tsx", ".json", ".md", ".txt", ".yml", ".yaml",
    ".toml", ".ini", ".cfg", ".css", ".scss", ".html", ".htm", ".xml", ".sh", ".bash",
    ".zsh", ".sql", ".rs", ".go", ".java", ".kt", ".c", ".h", ".cpp", ".hpp", ".cs",
    ".rb", ".php", ".lua", ".vue", ".svelte", ".csv", ".log", ".jsx", ".env",
}
TEXT_NAMES = {"dockerfile", "makefile", "license", "readme", "procfile", ".gitignore",
              ".gitattributes", ".editorconfig", ".dockerignore"}


def _is_text(rel):
    name = rel.rsplit("/", 1)[-1].lower()
    return name in TEXT_NAMES or os.path.splitext(name)[1] in TEXT_EXT


def _text_rank(rel):
    """Prefer shallow, readable source files when picking the preview file."""
    name = rel.rsplit("/", 1)[-1].lower()
    ext = os.path.splitext(name)[1]
    if name.startswith("readme"):
        prio = 0
    elif ext in {".py", ".js", ".ts", ".tsx", ".jsx", ".rs", ".go", ".java", ".c", ".cpp",
                 ".cs", ".rb", ".kt"}:
        prio = 1
    elif ext in {".md", ".txt", ".json", ".toml", ".yml", ".yaml"}:
        prio = 2
    else:
        prio = 3
    return (rel.count("/"), prio, rel)


def snapshot_highlight(snap):
    """The real file a snapshot preview is built from, plus its file count."""
    files = (snap or {}).get("files") or {}
    if not files:
        return None, 0
    return min(files, key=_text_rank), len(files)


_PREVIEW_CACHE = {}


def snapshot_preview(pid, snap, max_lines=8, previous=None):
    """Real content of a snapshot: a file it changed, and the first real lines of it."""
    if not pid or not snap:
        return None
    key = (pid, snap.get("id"), max_lines, (previous or {}).get("id"))
    cached = _PREVIEW_CACHE.get(key)
    if cached is not None:
        return cached
    files = snap.get("files") or {}
    count = len(files)
    rel = None
    if previous:
        before = previous.get("files") or {}
        touched = [r for r in files if (before.get(r) or {}).get("h") != files[r].get("h")]
        readable = [r for r in touched if _is_text(r)]
        if readable:
            rel = min(readable, key=_text_rank)
    if rel is None:
        rel, count = snapshot_highlight(snap)
    if not rel:
        return None
    info = files.get(rel) or {}
    raw = store.read_blob(pid, info.get("h", ""))[:2400]
    binary = b"\x00" in raw
    lines = []
    if not binary:
        for line in raw.decode("utf-8", "replace").splitlines():
            line = line.replace("\t", "    ").rstrip()
            if line.strip():
                lines.append(line[:140])
            if len(lines) >= max_lines:
                break
    preview = {"path": rel, "lines": lines, "count": count,
               "bytes": snap.get("bytes", 0), "binary": binary}
    if len(_PREVIEW_CACHE) > 400:
        _PREVIEW_CACHE.clear()
    _PREVIEW_CACHE[key] = preview
    return preview


def _mono(size, bold=False):
    f = QFont("DejaVu Sans Mono")
    f.setStyleHint(QFont.Monospace)
    f.setPixelSize(max(5, int(size)))
    f.setBold(bold)
    return f


def _line_color(text):
    stripped = text.lstrip()
    if stripped.startswith(("#", "//", "/*", "*", "--", "<!--")):
        return QColor("#4f7f9e")
    if stripped.startswith(("def ", "class ", "import ", "from ", "function ", "const ",
                            "let ", "var ", "return ", "if ", "for ", "while ", "export ")):
        return QColor(ACCENT)
    return QColor("#a8c6dd")


def paint_preview(p, rect, preview, radius=8):
    """Paint a snapshot's real file path and the first real lines of its content."""
    path = QPainterPath()
    path.addRoundedRect(rect, radius, radius)
    p.save()
    p.setClipPath(path)
    p.fillRect(rect, QColor("#060e18"))
    header = min(19.0, max(13.0, rect.height() * 0.17))
    p.setPen(Qt.NoPen)
    p.setBrush(QColor("#0b1d2d"))
    p.drawRect(QRectF(rect.x(), rect.y(), rect.width(), header))
    p.setFont(_mono(9 if rect.height() > 52 else 7))
    fm = p.fontMetrics()
    p.setPen(QColor("#7ea9c9"))
    p.drawText(QRectF(rect.x() + 6, rect.y() + 1, rect.width() - 12, header - 2),
               Qt.AlignLeft | Qt.AlignVCenter,
               fm.elidedText(preview.get("path", ""), Qt.ElideMiddle, int(rect.width() - 12)))

    body = QRectF(rect.x() + 6, rect.y() + header + 3, rect.width() - 12,
                  rect.height() - header - 5)
    lines = preview.get("lines") or []
    if not lines:
        p.setFont(_mono(8))
        p.setPen(QColor(DIM))
        p.drawText(body, Qt.AlignLeft | Qt.AlignTop,
                   "binary file" if preview.get("binary") else f"{preview.get('count', 0)} files")
    else:
        lh = max(6.0, min(16.0, body.height() / len(lines)))
        p.setFont(_mono(lh * 0.92))
        fm = p.fontMetrics()
        y = body.y()
        for line in lines:
            if y + lh > body.bottom() + 1:
                break
            p.setPen(_line_color(line))
            p.drawText(QRectF(body.x(), y, body.width(), lh), Qt.AlignLeft | Qt.AlignVCenter,
                       fm.elidedText(line, Qt.ElideRight, int(body.width())))
            y += lh
    p.restore()
    p.setBrush(Qt.NoBrush)
    p.setPen(QPen(QColor(255, 255, 255, 24), 1))
    p.drawRoundedRect(rect, radius, radius)


class SnapshotPreview(QWidget):
    """A live preview of one snapshot's real content."""

    def __init__(self, radius=8, parent=None):
        super().__init__(parent)
        self.radius = radius
        self.preview = None
        self.setAttribute(Qt.WA_TransparentForMouseEvents, True)

    def set_snapshot(self, pid, snap, previous=None):
        self.preview = (snapshot_preview(pid, snap, previous=previous)
                        if (pid and snap) else None)
        self.update()

    def paintEvent(self, e):
        p = QPainter(self)
        p.setRenderHint(QPainter.Antialiasing)
        r = QRectF(self.rect()).adjusted(0.5, 0.5, -0.5, -0.5)
        if not self.preview:
            path = QPainterPath()
            path.addRoundedRect(r, self.radius, self.radius)
            p.fillPath(path, QColor("#08141f"))
            p.setPen(QPen(QColor(BORDER), 1))
            p.drawPath(path)
            p.setFont(QFont())
            p.setPen(QColor(MUTED))
            p.drawText(r, Qt.AlignCenter, "No snapshot yet")
            return
        paint_preview(p, r, self.preview, self.radius)


class SnapshotBadge(QWidget):
    """Compact tile with the real file type and file count of a snapshot."""

    def __init__(self, snap=None, parent=None):
        super().__init__(parent)
        self.setAttribute(Qt.WA_TransparentForMouseEvents, True)
        self.setFixedSize(46, 36)
        self.path = None
        self.count = 0
        self.set_snap(snap)

    def set_snap(self, snap):
        self.path, self.count = snapshot_highlight(snap)
        self.update()

    def paintEvent(self, e):
        p = QPainter(self)
        p.setRenderHint(QPainter.Antialiasing)
        r = QRectF(self.rect()).adjusted(0.5, 0.5, -0.5, -0.5)
        path = QPainterPath()
        path.addRoundedRect(r, 8, 8)
        p.fillPath(path, QColor("#0a1c2b"))
        p.setPen(QPen(QColor(BORDER), 1))
        p.drawPath(path)
        if not self.path:
            p.setFont(_mono(9))
            p.setPen(QColor(DIM))
            p.drawText(r, Qt.AlignCenter, "—")
            return
        ext = os.path.splitext(self.path)[1].lstrip(".").upper() or "FILE"
        p.setFont(_mono(11, True))
        p.setPen(QColor(ACCENT))
        p.drawText(QRectF(r.x(), r.y() + 4, r.width(), 14), Qt.AlignCenter, ext[:4])
        p.setFont(_mono(8))
        p.setPen(QColor(MUTED))
        p.drawText(QRectF(r.x(), r.y() + 19, r.width(), 12), Qt.AlignCenter, f"{self.count} files")


class Sparkline(QWidget):
    """A tiny line chart used inside dashboard stat cards."""

    def __init__(self, color=ACCENT, parent=None):
        super().__init__(parent)
        self.data = []
        self.color = color
        self.setFixedSize(98, 34)

    def set_data(self, data):
        self.data = [max(0.0, float(v)) for v in data]
        self.update()

    def paintEvent(self, e):
        if len(self.data) < 2:
            return
        p = QPainter(self)
        p.setRenderHint(QPainter.Antialiasing)
        w, h = self.width(), self.height()
        lo, hi = min(self.data), max(self.data)
        span = (hi - lo) or 1.0
        pts = [QPointF(2 + i * (w - 4) / (len(self.data) - 1), h - 3 - (v - lo) / span * (h - 8))
               for i, v in enumerate(self.data)]
        path = QPainterPath(pts[0])
        for pt in pts[1:]:
            path.lineTo(pt)
        fill = QPainterPath(path)
        fill.lineTo(w - 2, h)
        fill.lineTo(2, h)
        fill.closeSubpath()
        grad = QLinearGradient(0, 0, 0, h)
        top = QColor(self.color)
        top.setAlpha(80)
        clear = QColor(self.color)
        clear.setAlpha(0)
        grad.setColorAt(0, top)
        grad.setColorAt(1, clear)
        p.fillPath(fill, grad)
        pen = QPen(QColor(self.color), 1.5)
        pen.setJoinStyle(Qt.RoundJoin)
        pen.setCapStyle(Qt.RoundCap)
        p.setPen(pen)
        p.setBrush(Qt.NoBrush)
        p.drawPath(path)
        p.setPen(Qt.NoPen)
        p.setBrush(QColor(self.color))
        p.drawEllipse(pts[-1], 1.9, 1.9)


class ToggleSwitch(QWidget):
    """A small pill switch, used by the dashboard status bar."""

    toggled = Signal(bool)

    def __init__(self, checked=False, parent=None):
        super().__init__(parent)
        self._on = bool(checked)
        self.setFixedSize(38, 20)
        self.setCursor(Qt.PointingHandCursor)

    def isChecked(self):
        return self._on

    def setChecked(self, on):
        self._on = bool(on)
        self.update()

    def mouseReleaseEvent(self, e):
        if e.button() == Qt.LeftButton and self.rect().contains(e.position().toPoint()):
            self._on = not self._on
            self.update()
            self.toggled.emit(self._on)
        super().mouseReleaseEvent(e)

    def paintEvent(self, e):
        p = QPainter(self)
        p.setRenderHint(QPainter.Antialiasing)
        track = QRectF(0, 1, self.width(), self.height() - 2)
        p.setPen(Qt.NoPen)
        p.setBrush(QColor(ACCENT if self._on else "#16344d"))
        p.drawRoundedRect(track, track.height() / 2, track.height() / 2)
        cx = track.right() - track.height() / 2 if self._on else track.left() + track.height() / 2
        p.setBrush(QColor("#f8fcff"))
        p.drawEllipse(QPointF(cx, track.center().y()), track.height() / 2 - 3, track.height() / 2 - 3)
