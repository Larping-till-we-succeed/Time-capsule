"""Timeline page: a day-grouped rail of snapshots next to the detail panel."""

import time

from PySide6.QtCore import QRectF, Qt, Signal
from PySide6.QtGui import QColor, QFont, QPainter, QPen
from PySide6.QtWidgets import (
    QComboBox, QFrame, QHBoxLayout, QLabel, QLineEdit, QPushButton, QScrollArea,
    QVBoxLayout, QWidget,
)

import store
from theme import ACCENT, BORDER, DIM, GREEN, MUTED, RED, TEXT, chip, hbox, label, vbox
from ui_detail import SnapshotDetail
from widgets import ClickFrame, EmptyState, ProjectTimeline


class SnapshotRow(QFrame):
    clicked = Signal(str)

    def __init__(self, snap, prev, selected=False, branches=None, parent=None):
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
        for name in (branches or []):
            c = chip(f"⑃ {name}", "#7dd3fc")
            c.setToolTip(f"The branch “{name}” starts from this moment")
            meta.addWidget(c)
        meta.addStretch(1)
        size = QLabel(store.fmt_bytes(snap["bytes"]))
        size.setStyleSheet(f"color:{DIM};font-size:11px")
        meta.addWidget(size)
        col.addLayout(meta)
        lay.addLayout(col, 1)

    def _apply_style(self):
        if self.selected:
            self.setStyleSheet(
                f"QFrame#snaprow{{background:#10293f;border:1px solid #1d5c8c;border-left:3px solid {ACCENT};"
                "border-radius:0 12px 12px 0}}"
            )
        else:
            self.setStyleSheet(
                "QFrame#snaprow{background:transparent;border:1px solid transparent;"
                "border-left:3px solid transparent;border-radius:0 12px 12px 0}"
                "QFrame#snaprow:hover{background:#0c2033;border:1px solid #16344d;border-left:3px solid #1b3f5c}"
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
        pen = QPen(QColor("#1a3b58"), 2)
        p.setPen(pen)
        p.drawLine(x, 0, x, h)
        color = ACCENT if self.selected else "#2f5575"
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
            p.setBrush(QColor("#10293f"))
            p.drawEllipse(QRectF(x - 2, h / 2 - 2, 4, 4))


class TimelineView(QWidget):
    new_snapshot = Signal()
    restore_all = Signal(str)
    restore_file = Signal(str, str)
    compare_requested = Signal(str, str)
    note_saved = Signal()
    delete_requested = Signal(str)
    branch_selected = Signal(str)
    new_branch = Signal(str)

    def __init__(self, parent=None):
        super().__init__(parent)
        self.pid = None
        self.snaps = []
        self.rows = {}
        self.selected = None
        self.branch_id = "main"
        root = QVBoxLayout(self)
        root.setContentsMargins(0, 0, 0, 0)
        root.setSpacing(14)

        rail = QFrame()
        rail.setObjectName("card")
        rl = QVBoxLayout(rail)
        rl.setContentsMargins(20, 14, 20, 8)
        rl.setSpacing(2)
        rhead = hbox(spacing=10)
        rcap = QLabel("PROJECT TIMELINE")
        rcap.setObjectName("tiny")
        self.rail_span = QLabel("")
        self.rail_span.setObjectName("sub")
        rhead.addWidget(rcap)
        rhead.addWidget(self.rail_span)
        rhead.addStretch(1)
        rhead.addWidget(chip("● manual snapshot", "#8fa6c0"))
        rhead.addWidget(chip("● auto-save", "#454b57"))
        rhead.addSpacing(10)
        bcap = QLabel("BRANCH")
        bcap.setObjectName("tiny")
        self.branch_box = QComboBox()
        self.branch_box.setMinimumWidth(200)
        self.branch_box.setCursor(Qt.PointingHandCursor)
        self.branch_box.currentIndexChanged.connect(self._branch_pick)
        newb = QPushButton("⑃   New branch")
        newb.setObjectName("ghost")
        newb.setCursor(Qt.PointingHandCursor)
        newb.setToolTip("Start a new branch at the selected moment")
        newb.clicked.connect(lambda: self.new_branch.emit(self.selected or ""))
        rhead.addWidget(bcap)
        rhead.addWidget(self.branch_box)
        rhead.addWidget(newb)
        rl.addLayout(rhead)
        self.rail = ProjectTimeline()
        self.rail.selected.connect(self.select)
        rl.addWidget(self.rail)
        root.addWidget(rail)

        cols = QHBoxLayout()
        cols.setSpacing(16)

        leftcard = QFrame()
        leftcard.setObjectName("card")
        left = QVBoxLayout(leftcard)
        left.setContentsMargins(16, 14, 8, 12)
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
        leftcard.setFixedWidth(424)
        cols.addWidget(leftcard)

        self.detail = SnapshotDetail()
        self.detail.setMinimumWidth(470)
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

    def load(self, pid, select=None, branch=None):
        self.pid = pid
        self.branch_id = branch or (store.active_branch(pid) if pid else "main")
        self.snaps = store.branch_chain(pid, self.branch_id) if pid else []
        self._load_branches()
        self.rebuild(select)

    def _branch_pick(self, index):
        bid = self.branch_box.itemData(index)
        if bid and bid != self.branch_id:
            self.branch_selected.emit(bid)

    def _load_branches(self):
        self.branch_box.blockSignals(True)
        self.branch_box.clear()
        if self.pid:
            branches = store.load_branches(self.pid)
            counts = {}
            for s in store.load_snapshots(self.pid):
                key = s.get("branch") or "main"
                counts[key] = counts.get(key, 0) + 1
            for b in branches:
                n = counts.get(b["id"], 0)
                self.branch_box.addItem(
                    f"{b['name']}  ·  {n} snapshot{'s' if n != 1 else ''}", b["id"])
                tip = ("branched from an earlier moment" if b.get("base")
                       else "the project's first history")
                self.branch_box.setItemData(self.branch_box.count() - 1, tip, Qt.ToolTipRole)
            index = self.branch_box.findData(self.branch_id)
            self.branch_box.setCurrentIndex(index if index >= 0 else 0)
        self.branch_box.blockSignals(False)

    def _branch_points(self):
        points = {}
        if not self.pid:
            return points
        for b in store.load_branches(self.pid):
            base = b.get("base")
            if base:
                points.setdefault(base, []).append(b["name"])
        return points

    def _load_rail(self):
        """Fill the branched timeline rail above the day-grouped list."""
        if not self.pid:
            self.rail.set_graph([], [])
            self.rail_span.setText("")
            return
        proj = store.get_project(self.pid)
        all_snaps = store.load_snapshots(self.pid)
        branches = store.load_branches(self.pid)
        if not all_snaps:
            self.rail.set_graph([], branches, self.branch_id)
            self.rail_span.setText("nothing captured yet")
            return
        start = min((proj or {}).get("created") or all_snaps[0]["ts"], all_snaps[0]["ts"])
        end = max(time.time(), all_snaps[-1]["ts"])
        if len(self.snaps) > 1:
            span = (self.snaps[-1]["ts"] - self.snaps[0]["ts"]) / 86400
            self.rail_span.setText(
                f"{len(self.snaps)} moments · {span:.1f} days · {len(branches)} branches"
                if len(branches) > 1 else f"{len(self.snaps)} moments · {span:.1f} days of history")
        else:
            self.rail_span.setText("1 moment" if len(branches) == 1 else "1 moment on this branch")
        self.rail.set_graph(all_snaps, branches, self.branch_id, start, end, self.selected,
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
        points = self._branch_points()
        last_day = None
        prev_by_id = {}
        for idx, s in enumerate(self.snaps):
            prev_by_id[s["id"]] = self.snaps[idx - 1] if idx else None
        for s in snaps:
            day = store.day_key(s["ts"])
            if day != last_day:
                head = QLabel(store.day_label(s["ts"]).upper())
                head.setObjectName("tiny")
                head.setStyleSheet("padding:14px 0 6px 46px;color:#6f8aa6")
                self.lcol.addWidget(head)
                last_day = day
            row = SnapshotRow(s, prev_by_id.get(s["id"]), s["id"] == self.selected,
                              points.get(s["id"]))
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
