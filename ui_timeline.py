"""Timeline page: a day-grouped rail of snapshots next to the detail panel."""

import time

from PySide6.QtCore import QRectF, Qt, Signal
from PySide6.QtGui import QColor, QFont, QPainter, QPen
from PySide6.QtWidgets import (
    QFrame, QHBoxLayout, QLabel, QLineEdit, QPushButton, QScrollArea, QVBoxLayout, QWidget,
)

import store
from theme import ACCENT, BORDER, DIM, GREEN, MUTED, RED, TEXT, chip, hbox, label, vbox
from ui_detail import SnapshotDetail
from widgets import ClickFrame, EmptyState, ProjectTimeline


class SnapshotRow(QFrame):
    clicked = Signal(str)

    def __init__(self, snap, prev, selected=False, parent=None):
        super().__init__(parent)
        self.sid = snap["id"]
        self.snap = snap
        self.selected = selected
        self.setCursor(Qt.PointingHandCursor)
        self.setFixedHeight(86)
        self.setObjectName("snaprow")
        self._apply_style()

        lay = QHBoxLayout(self)
        lay.setContentsMargins(46, 12, 14, 12)
        lay.setSpacing(6)
        col = QVBoxLayout()
        col.setSpacing(3)
        top = hbox(spacing=8)
        t = QLabel(store.fmt_time(snap["ts"]))
        t.setStyleSheet("font-size:15px;font-weight:800")
        top.addWidget(t)
        title = QLabel(snap.get("title") or "Snapshot")
        title.setStyleSheet(f"color:{MUTED};font-size:12px;font-weight:700")
        top.addWidget(title)
        top.addStretch(1)
        col.addLayout(top)
        note = snap.get("note", "").strip() or "No note"
        n = QLabel(note)
        n.setObjectName("sub")
        n.setStyleSheet(f"color:{'#c8ccd4' if snap.get('note') else DIM};font-size:12px")
        fm = n.fontMetrics()
        n.setText(fm.elidedText(note, Qt.ElideRight, 262))
        col.addWidget(n)
        meta = hbox(spacing=6)
        if prev is not None:
            d = store.compare(prev, snap)["counts"]
            for kind, mark, color in (("added", "+", GREEN), ("modified", "~", ACCENT), ("deleted", "−", RED)):
                if d[kind]:
                    c = chip(f"{mark}{d[kind]}", color, QColor(color).darker(340).name())
                    c.setToolTip(f"{d[kind]} {kind} file{'s' if d[kind] != 1 else ''} in this snapshot")
                    meta.addWidget(c)
        else:
            meta.addWidget(chip("first snapshot", MUTED))
        meta.addStretch(1)
        size = QLabel(store.fmt_bytes(snap["bytes"]))
        size.setStyleSheet(f"color:{DIM};font-size:11px")
        meta.addWidget(size)
        col.addLayout(meta)
        lay.addLayout(col, 1)

    def _apply_style(self):
        if self.selected:
            self.setStyleSheet(
                f"QFrame#snaprow{{background:#171a20;border:1px solid #4a3a1c;border-left:3px solid {ACCENT};"
                "border-radius:0 12px 12px 0}}"
            )
        else:
            self.setStyleSheet(
                "QFrame#snaprow{background:transparent;border:1px solid transparent;"
                "border-left:3px solid transparent;border-radius:0 12px 12px 0}"
                "QFrame#snaprow:hover{background:#14161b;border:1px solid #202430;border-left:3px solid #2c2f38}"
            )

    def set_selected(self, on):
        if self.selected != on:
            self.selected = on
            self._apply_style()

    def mouseReleaseEvent(self, e):
        if e.button() == Qt.LeftButton and self.rect().contains(e.position().toPoint()):
            self.clicked.emit(self.sid)
        super().mouseReleaseEvent(e)

    def paintEvent(self, e):
        super().paintEvent(e)
        p = QPainter(self)
        p.setRenderHint(QPainter.Antialiasing)
        h = self.height()
        x = 24
        pen = QPen(QColor("#2a2e38"), 2)
        p.setPen(pen)
        p.drawLine(x, 0, x, h)
        color = ACCENT if self.selected else "#3a3f4a"
        if self.selected:
            glow = QColor(ACCENT)
            glow.setAlpha(40)
            p.setBrush(glow)
            p.setPen(Qt.NoPen)
            p.drawEllipse(QRectF(x - 13, h / 2 - 13, 26, 26))
        p.setBrush(QColor(color))
        p.setPen(Qt.PenStyle.NoPen)
        p.drawEllipse(QRectF(x - 5, h / 2 - 5, 10, 10))
        if self.selected:
            p.setBrush(QColor("#171a20"))
            p.drawEllipse(QRectF(x - 2, h / 2 - 2, 4, 4))


class TimelineView(QWidget):
    new_snapshot = Signal()
    restore_all = Signal(str)
    restore_file = Signal(str, str)
    compare_requested = Signal(str, str)
    note_saved = Signal()
    delete_requested = Signal(str)

    def __init__(self, parent=None):
        super().__init__(parent)
        self.pid = None
        self.snaps = []
        self.rows = {}
        self.selected = None
        root = QVBoxLayout(self)
        root.setContentsMargins(0, 0, 0, 0)
        root.setSpacing(14)

        rail = QFrame()
        rail.setObjectName("card")
        rl = QVBoxLayout(rail)
        rl.setContentsMargins(18, 12, 18, 8)
        rl.setSpacing(4)
        rhead = hbox(spacing=10)
        rcap = QLabel("PROJECT TIMELINE")
        rcap.setObjectName("tiny")
        self.rail_span = QLabel("")
        self.rail_span.setObjectName("sub")
        rhead.addWidget(rcap)
        rhead.addWidget(self.rail_span)
        rhead.addStretch(1)
        rhead.addWidget(chip("● manual snapshot", "#8b929e"))
        rhead.addWidget(chip("● auto-save", "#454b57"))
        rl.addLayout(rhead)
        self.rail = ProjectTimeline()
        self.rail.selected.connect(self.select)
        rl.addWidget(self.rail)
        root.addWidget(rail)

        cols = QHBoxLayout()
        cols.setSpacing(20)

        left = QVBoxLayout()
        left.setSpacing(10)
        head = hbox(spacing=10)
        cap = QLabel("HISTORY")
        cap.setObjectName("tiny")
        self.count = QLabel("")
        self.count.setObjectName("sub")
        head.addWidget(cap)
        head.addWidget(self.count)
        head.addStretch(1)
        left.addLayout(head)
        self.search = QLineEdit()
        self.search.setPlaceholderText("Search notes and files…")
        self.search.textChanged.connect(lambda _t: self.rebuild())
        left.addWidget(self.search)

        self.lscroll = QScrollArea()
        self.lscroll.setWidgetResizable(True)
        self.lbody = QWidget()
        self.lcol = QVBoxLayout(self.lbody)
        self.lcol.setContentsMargins(0, 0, 6, 0)
        self.lcol.setSpacing(2)
        self.lscroll.setWidget(self.lbody)
        left.addWidget(self.lscroll, 1)
        leftwrap = QWidget()
        leftwrap.setLayout(left)
        leftwrap.setFixedWidth(430)
        cols.addWidget(leftwrap)

        self.detail = SnapshotDetail()
        self.detail.restore_all.connect(self.restore_all.emit)
        self.detail.restore_file.connect(lambda sid, rel: self.restore_file.emit(sid, rel))
        self.detail.compare_previous.connect(self._compare_prev)
        self.detail.note_saved.connect(lambda _sid: self.note_saved.emit())
        self.detail.delete_requested.connect(self.delete_requested.emit)
        cols.addWidget(self.detail, 1)
        root.addLayout(cols, 1)

    def _compare_prev(self, sid):
        prev = store.previous_snapshot(self.pid, sid)
        if prev:
            self.compare_requested.emit(prev["id"], sid)

    def load(self, pid, select=None):
        self.pid = pid
        self.snaps = store.load_snapshots(pid) if pid else []
        self.rebuild(select)

    def _load_rail(self):
        """Fill the project-wide timeline rail above the day-grouped list."""
        if not self.snaps:
            self.rail.set_data([], selected=self.selected)
            self.rail_span.setText("nothing captured yet")
            return
        proj = store.get_project(self.pid) if self.pid else None
        start = min((proj or {}).get("created") or self.snaps[0]["ts"], self.snaps[0]["ts"])
        end = max(time.time(), self.snaps[-1]["ts"])
        if len(self.snaps) > 1:
            span = (self.snaps[-1]["ts"] - self.snaps[0]["ts"]) / 86400
            self.rail_span.setText(f"{len(self.snaps)} moments · {span:.1f} days of history")
        else:
            self.rail_span.setText("1 moment")
        self.rail.set_data(self.snaps, start, end, self.selected,
                          store.fmt_date(start), "today")

    def rebuild(self, select=None):
        self._load_rail()
        while self.lcol.count():
            item = self.lcol.takeAt(0)
            w = item.widget()
            if w:
                w.deleteLater()
        self.rows = {}
        if not self.snaps:
            self.count.setText("")
            state = EmptyState(
                "🕰",
                "No snapshots yet",
                "Capture the current state of your project and it will appear here, "
                "grouped by day, ready to revisit.",
                "Capture the first snapshot",
            )
            state.clicked.connect(self.new_snapshot.emit)
            self.lcol.addWidget(state)
            self.lcol.addStretch(1)
            self.selected = None
            self.detail.setVisible(False)
            return
        self.detail.setVisible(True)
        query = self.search.text().strip().lower()
        snaps = list(reversed(self.snaps))
        if query:
            snaps = [s for s in snaps
                     if query in s.get("title", "").lower()
                     or query in s.get("note", "").lower()
                     or any(query in p.lower() for p in s["files"])]
        self.count.setText(f"{len(snaps)} of {len(self.snaps)}" if query else f"{len(self.snaps)} snapshots")
        order = {s["id"]: s for s in self.snaps}
        last_day = None
        prev_by_id = {}
        for idx, s in enumerate(self.snaps):
            prev_by_id[s["id"]] = self.snaps[idx - 1] if idx else None
        for s in snaps:
            day = store.day_key(s["ts"])
            if day != last_day:
                head = QLabel(store.day_label(s["ts"]).upper())
                head.setObjectName("tiny")
                head.setStyleSheet("padding:14px 0 6px 46px;color:#6f7683")
                self.lcol.addWidget(head)
                last_day = day
            row = SnapshotRow(s, prev_by_id.get(s["id"]), s["id"] == self.selected)
            row.clicked.connect(self.select)
            self.lcol.addWidget(row)
            self.rows[s["id"]] = row
        if not self.rows:
            miss = QLabel("No snapshot matches that search.")
            miss.setObjectName("empty")
            miss.setStyleSheet("padding:30px 8px")
            self.lcol.addWidget(miss)
        self.lcol.addStretch(1)

        current = select or self.selected or (self.snaps[-1]["id"] if self.snaps else None)
        if current in self.rows:
            self.select(current)
        else:
            self.selected = None
            self.select(self.snaps[-1]["id"] if self.snaps else None)

    def select(self, sid):
        if not sid or sid not in self.rows:
            return
        for key, row in self.rows.items():
            row.set_selected(key == sid)
        self.selected = sid
        self.rail.set_selected(sid)
        snap = next((s for s in self.snaps if s["id"] == sid), None)
        prev = store.previous_snapshot(self.pid, sid)
        if snap:
            self.detail.load(self.pid, snap, prev)
