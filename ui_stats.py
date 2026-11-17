"""Statistics page: how much history exists, where it grew and what changes most."""

from PySide6.QtCore import Qt
from PySide6.QtWidgets import (
    QFrame, QHBoxLayout, QLabel, QScrollArea, QSizePolicy, QVBoxLayout, QWidget,
)

import store
from theme import ACCENT, DIM, GREEN, MUTED, RED, TEXT, hbox, hline, label, vbox
from widgets import BarChart, EmptyState, MiniBar, Section, StatCard


class StatsView(QWidget):
    def __init__(self, parent=None):
        super().__init__(parent)
        self.pid = None
        root = QVBoxLayout(self)
        root.setContentsMargins(0, 0, 0, 0)
        root.setSpacing(16)

        cards = hbox(spacing=12)
        self.c_snaps = StatCard("snapshots", "0", "🕰", ACCENT)
        self.c_files = StatCard("tracked files", "0", "▤", "#7dd3fc")
        self.c_size = StatCard("project size", "0 B", "◈", GREEN)
        self.c_store = StatCard("project storage", "0 B", "❖", "#c4b5fd")
        for c in (self.c_snaps, self.c_files, self.c_size, self.c_store):
            cards.addWidget(c)
        root.addLayout(cards)

        chart_card = QFrame()
        chart_card.setObjectName("card")
        cl = QVBoxLayout(chart_card)
        cl.setContentsMargins(18, 14, 18, 14)
        cl.setSpacing(6)
        head = hbox(spacing=10)
        t = QLabel("Snapshots over the last 14 days")
        t.setStyleSheet("font-size:15px;font-weight:800")
        self.span = QLabel("")
        self.span.setObjectName("sub")
        head.addWidget(t)
        head.addWidget(self.span)
        head.addStretch(1)
        cl.addLayout(head)
        self.chart = BarChart()
        cl.addWidget(self.chart, 1)
        root.addWidget(chart_card, 1)

        bottom = hbox(spacing=16)
        left = QFrame()
        left.setObjectName("card")
        ll = QVBoxLayout(left)
        ll.setContentsMargins(18, 14, 18, 14)
        ll.setSpacing(10)
        lt = QLabel("Most reworked files")
        lt.setStyleSheet("font-size:15px;font-weight:800")
        ll.addWidget(lt)
        self.scroll = QScrollArea()
        self.scroll.setWidgetResizable(True)
        self.fbody = QWidget()
        self.fcol = QVBoxLayout(self.fbody)
        self.fcol.setContentsMargins(0, 0, 0, 0)
        self.fcol.setSpacing(8)
        self.scroll.setWidget(self.fbody)
        ll.addWidget(self.scroll, 1)
        bottom.addWidget(left, 1)

        right = QFrame()
        right.setObjectName("card")
        right.setFixedWidth(330)
        rl = QVBoxLayout(right)
        rl.setContentsMargins(18, 14, 18, 14)
        rl.setSpacing(10)
        rt = QLabel("Project facts")
        rt.setStyleSheet("font-size:15px;font-weight:800")
        rl.addWidget(rt)
        holder = QWidget()
        self.facts = QVBoxLayout(holder)
        self.facts.setContentsMargins(0, 0, 0, 0)
        self.facts.setSpacing(12)
        fscroll = QScrollArea()
        fscroll.setWidgetResizable(True)
        fscroll.setWidget(holder)
        rl.addWidget(fscroll, 1)
        bottom.addWidget(right)
        root.addLayout(bottom, 1)

    def _fact(self, title, value):
        box = QVBoxLayout()
        box.setSpacing(2)
        a = QLabel(title.upper())
        a.setObjectName("tiny")
        b = QLabel(value)
        b.setStyleSheet("font-size:13px;font-weight:700;color:#e6e9ef")
        b.setWordWrap(True)
        b.setTextInteractionFlags(Qt.TextSelectableByMouse)
        box.addWidget(a)
        box.addWidget(b)
        return box

    def load(self, pid, branch=None):
        self.pid = pid
        while self.fcol.count():
            item = self.fcol.takeAt(0)
            w = item.widget()
            if w:
                w.deleteLater()
        while self.facts.count():
            item = self.facts.takeAt(0)
            if item.layout():
                while item.layout().count():
                    c = item.layout().takeAt(0)
                    if c.widget():
                        c.widget().deleteLater()
                item.layout().deleteLater()
        if not pid:
            self.chart.set_data([])
            return
        s = store.project_stats(pid, branch)
        proj = store.get_project(pid)
        self.c_snaps.set(s["snapshots"])
        self.c_files.set(s["files"], "tracked in the live folder")
        self.c_size.set(store.fmt_bytes(s["live_bytes"]), "sum of tracked files")
        saved = max(0, s["live_bytes"] * max(1, s["snapshots"]) - s["storage"])
        self.c_store.set(store.fmt_bytes(s["storage"]), f"{store.fmt_bytes(saved)} saved by dedupe")
        self.chart.set_data(s["per_day"])
        if s["first"]:
            self.span.setText(f"since {store.fmt_date(s['first'])}")
        else:
            self.span.setText("no history yet")

        if not s["top_changed"]:
            empty = QLabel("Files that get edited between snapshots will show up here.")
            empty.setObjectName("empty")
            empty.setWordWrap(True)
            self.fcol.addWidget(empty)
        else:
            top = s["top_changed"][0][1] or 1
            for rel, count in s["top_changed"]:
                row = QWidget()
                rl = QVBoxLayout(row)
                rl.setContentsMargins(0, 0, 0, 0)
                rl.setSpacing(4)
                head = hbox(spacing=8)
                name = QLabel(rel)
                name.setStyleSheet("font-family:'JetBrains Mono',Consolas,monospace;font-size:12px")
                name.setSizePolicy(QSizePolicy.Expanding, QSizePolicy.Preferred)
                fm = name.fontMetrics()
                name.setText(fm.elidedText(rel, Qt.ElideMiddle, 430))
                name.setToolTip(rel)
                cnt = QLabel(f"{count}×")
                cnt.setStyleSheet(f"color:{ACCENT};font-size:11px;font-weight:800")
                head.addWidget(name, 1)
                head.addWidget(cnt)
                rl.addLayout(head)
                rl.addWidget(MiniBar(count / top))
                self.fcol.addWidget(row)
        self.fcol.addStretch(1)

        for title, value in (
            ("Branch", f"{s['branch_name']} · {s['branches']} total"),
            ("Timeline span", f"{s['span_days']:.1f} days" if s["span_days"] else "one moment"),
            ("First capture", store.fmt_stamp(s["first"]) if s["first"] else "—"),
            ("Latest capture", store.fmt_stamp(s["last"]) if s["last"] else "—"),
            ("Folder", proj["path"] if proj else "—"),
        ):
            self.facts.addLayout(self._fact(title, value))
