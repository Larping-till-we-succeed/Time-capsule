"""Compare page: pick two moments, read the summary and inspect file diffs."""

import html

from PySide6.QtCore import Qt, Signal
from PySide6.QtGui import QColor
from PySide6.QtWidgets import (
    QComboBox, QFrame, QHBoxLayout, QLabel, QLineEdit, QPushButton, QScrollArea,
    QSizePolicy, QTextBrowser, QVBoxLayout, QWidget,
)

import diffing
import store
from theme import ACCENT, BORDER, DIM, GREEN, MUTED, RED, TEXT, hbox, label, vbox
from widgets import EmptyState, MiniBar, StatCard, StatusPill


class CompareRow(QFrame):
    clicked = Signal(str)

    def __init__(self, rel, kind, parent=None):
        super().__init__(parent)
        self.rel = rel
        self.kind = kind
        self.active = False
        self.setObjectName("cmprow")
        self.setCursor(Qt.PointingHandCursor)
        self.setFixedHeight(40)
        self._style()
        lay = QHBoxLayout(self)
        lay.setContentsMargins(10, 6, 10, 6)
        lay.setSpacing(10)
        lay.addWidget(StatusPill(kind))
        name = QLabel(rel)
        name.setStyleSheet("font-family:'JetBrains Mono',Consolas,monospace;font-size:12px")
        name.setSizePolicy(QSizePolicy.Expanding, QSizePolicy.Preferred)
        fm = name.fontMetrics()
        name.setToolTip(rel)
        name.setText(fm.elidedText(rel, Qt.ElideMiddle, 320))
        lay.addWidget(name, 1)
        arrow = QLabel("›")
        arrow.setStyleSheet(f"color:{DIM}")
        lay.addWidget(arrow)

    def _style(self):
        if self.active:
            self.setStyleSheet(
                "QFrame#cmprow{background:#10293f;border:1px solid #1d5c8c;"
                f"border-left:2px solid {ACCENT};border-radius:9px}}"
            )
        else:
            self.setStyleSheet(
                "QFrame#cmprow{background:transparent;border:1px solid transparent;"
                "border-left:2px solid transparent;border-radius:9px}"
                "QFrame#cmprow:hover{background:#0c2033;border:1px solid #16344d;"
                "border-left:2px solid #1b3f5c}"
            )

    def set_active(self, on):
        self.active = on
        self._style()

    def mouseReleaseEvent(self, e):
        if e.button() == Qt.LeftButton and self.rect().contains(e.position().toPoint()):
            self.clicked.emit(self.rel)
        super().mouseReleaseEvent(e)


class CompareView(QWidget):
    def __init__(self, parent=None):
        super().__init__(parent)
        self.pid = None
        self.snaps = []
        self.a = None
        self.b = None
        self.rows = {}
        self.active_file = None
        root = QVBoxLayout(self)
        root.setContentsMargins(0, 0, 0, 0)
        root.setSpacing(16)

        bar = QFrame()
        bar.setObjectName("card")
        bl = QHBoxLayout(bar)
        bl.setContentsMargins(16, 12, 16, 12)
        bl.setSpacing(10)
        self.ca = QComboBox()
        self.cb = QComboBox()
        for c in (self.ca, self.cb):
            c.setMinimumWidth(290)
            c.setCursor(Qt.PointingHandCursor)
            c.currentIndexChanged.connect(lambda _i: self.refresh())
        swap = QPushButton("⇄")
        swap.setToolTip("Swap the two snapshots")
        swap.setCursor(Qt.PointingHandCursor)
        swap.setFixedSize(34, 34)
        swap.clicked.connect(self.swap)
        bl.addWidget(label("FROM", "tiny"))
        bl.addWidget(self.ca)
        bl.addSpacing(4)
        bl.addWidget(label("TO", "tiny"))
        bl.addWidget(self.cb)
        bl.addWidget(swap)
        bl.addStretch(1)
        root.addWidget(bar)

        cards = hbox(spacing=12)
        self.c_add = StatCard("added", "0", "＋", GREEN)
        self.c_mod = StatCard("modified", "0", "~", ACCENT)
        self.c_del = StatCard("deleted", "0", "−", RED)
        self.c_same = StatCard("unchanged", "0", "=", MUTED)
        for c in (self.c_add, self.c_mod, self.c_del, self.c_same):
            cards.addWidget(c)
        root.addLayout(cards)

        body = hbox(spacing=16)
        left = QFrame()
        left.setObjectName("card")
        ll = QVBoxLayout(left)
        ll.setContentsMargins(14, 12, 14, 12)
        ll.setSpacing(8)
        self.search = QLineEdit()
        self.search.setPlaceholderText("Filter changed files…")
        self.search.textChanged.connect(lambda _t: self.fill_rows())
        ll.addWidget(self.search)
        self.scroll = QScrollArea()
        self.scroll.setWidgetResizable(True)
        self.lbody = QWidget()
        self.lcol = QVBoxLayout(self.lbody)
        self.lcol.setContentsMargins(0, 0, 0, 0)
        self.lcol.setSpacing(1)
        self.scroll.setWidget(self.lbody)
        ll.addWidget(self.scroll, 1)
        left.setFixedWidth(480)
        body.addWidget(left)

        right = QFrame()
        right.setObjectName("card")
        rl = QVBoxLayout(right)
        rl.setContentsMargins(14, 12, 14, 12)
        rl.setSpacing(8)
        head = hbox(spacing=8)
        self.fname = QLabel("Select a file")
        self.fname.setStyleSheet("font-family:'JetBrains Mono',Consolas,monospace;font-size:12px")
        self.fname.setSizePolicy(QSizePolicy.Expanding, QSizePolicy.Preferred)
        self.fmeta = QLabel("")
        self.fmeta.setObjectName("sub")
        head.addWidget(self.fname, 1)
        head.addWidget(self.fmeta)
        rl.addLayout(head)
        self.diff = QTextBrowser()
        self.diff.setObjectName("diff")
        self.diff.setOpenExternalLinks(False)
        rl.addWidget(self.diff, 1)
        body.addWidget(right, 1)
        root.addLayout(body, 1)

    # ------------------------------------------------------------------ data
    def load(self, pid, a=None, b=None, branch=None):
        self.pid = pid
        self.snaps = store.branch_chain(pid, branch) if pid else []
        self.ca.blockSignals(True)
        self.cb.blockSignals(True)
        self.ca.clear()
        self.cb.clear()
        for s in self.snaps:
            text = f"{store.fmt_stamp(s['ts'])}  ·  {s.get('title') or s.get('note') or s['id'][:6]}"
            self.ca.addItem(text, s["id"])
            self.cb.addItem(text, s["id"])
        self.ca.blockSignals(False)
        self.cb.blockSignals(False)
        if len(self.snaps) >= 2:
            ia = self.snaps[-2]["id"] if a is None else a
            ib = self.snaps[-1]["id"] if b is None else b
            self.ca.setCurrentIndex(max(0, [s["id"] for s in self.snaps].index(ia)) if ia in [s["id"] for s in self.snaps] else len(self.snaps) - 2)
            self.cb.setCurrentIndex(max(0, [s["id"] for s in self.snaps].index(ib)) if ib in [s["id"] for s in self.snaps] else len(self.snaps) - 1)
        self.refresh()

    def swap(self):
        ia, ib = self.ca.currentIndex(), self.cb.currentIndex()
        self.ca.blockSignals(True)
        self.cb.blockSignals(True)
        self.ca.setCurrentIndex(ib)
        self.cb.setCurrentIndex(ia)
        self.ca.blockSignals(False)
        self.cb.blockSignals(False)
        self.refresh()

    def _snap(self, sid):
        return next((s for s in self.snaps if s["id"] == sid), None)

    def refresh(self):
        self.a = self._snap(self.ca.currentData()) if self.snaps else None
        self.b = self._snap(self.cb.currentData()) if self.snaps else None
        while self.lcol.count():
            item = self.lcol.takeAt(0)
            w = item.widget()
            if w:
                w.deleteLater()
        self.rows = {}
        self.active_file = None
        if not self.a or not self.b or len(self.snaps) < 2:
            self.diff.setHtml(
                f"<div style='color:{MUTED};font-family:sans-serif;padding:18px'>"
                "Add at least two snapshots to compare a project with itself over time.</div>")
            for c in (self.c_add, self.c_mod, self.c_del, self.c_same):
                c.set("0")
            self.fname.setText("Nothing to compare yet")
            self.fmeta.setText("")
            return
        d = store.compare(self.a, self.b)
        c = d["counts"]
        self.c_add.set(c["added"], store.fmt_bytes(sum(self.b["files"][p]["size"] for p in d["added"])))
        self.c_mod.set(c["modified"], store.fmt_bytes(sum(self.b["files"][p]["size"] for p in d["modified"])))
        self.c_del.set(c["deleted"], store.fmt_bytes(sum(self.a["files"][p]["size"] for p in d["deleted"])))
        self.c_same.set(c["same"], f"{len(self.a['files'])} → {len(self.b['files'])} files")
        self.fill_rows()

    def fill_rows(self):
        while self.lcol.count():
            item = self.lcol.takeAt(0)
            w = item.widget()
            if w:
                w.deleteLater()
        self.rows = {}
        if not self.a or not self.b:
            return
        d = store.compare(self.a, self.b)
        rows = store.ordered_changes(d)
        q = self.search.text().strip().lower()
        if q:
            rows = [(rel, k) for rel, k in rows if q in rel.lower()]
        if not rows:
            msg = QLabel("No differences between these two snapshots.")
            msg.setObjectName("empty")
            msg.setWordWrap(True)
            self.lcol.addWidget(msg)
            self.lcol.addStretch(1)
            self.diff.setHtml(f"<div style='color:{MUTED};font-family:sans-serif;padding:18px'>"
                              "These snapshots contain exactly the same files.</div>")
            self.fname.setText("No changes")
            self.fmeta.setText("")
            return
        for rel, kind in rows:
            row = CompareRow(rel, kind)
            row.clicked.connect(self.show_file)
            self.lcol.addWidget(row)
            self.rows[rel] = row
        self.lcol.addStretch(1)
        first = rows[0][0]
        self.show_file(first)

    def show_file(self, rel):
        if not self.a or not self.b:
            return
        for key, row in self.rows.items():
            row.set_active(key == rel)
        self.active_file = rel
        fa = self.a["files"].get(rel)
        fb = self.b["files"].get(rel)
        old = store.read_blob(self.pid, fa["h"]) if fa else b""
        new = store.read_blob(self.pid, fb["h"]) if fb else b""
        self.fname.setText(rel)
        self.fname.setToolTip(rel)
        kind = "modified" if fa and fb else ("added" if fb else "deleted")
        if not diffing.looks_text(rel, old) or not diffing.looks_text(rel, new):
            self.fmeta.setText("binary file")
            self.diff.setHtml(
                f"<div style='color:{MUTED};font-family:sans-serif;padding:18px'>"
                f"<b style='color:{TEXT}'>{html.escape(rel)}</b><br><br>"
                f"This is a binary file, so it cannot be shown as a text diff.<br>"
                f"Size before: {store.fmt_bytes(len(old))} · after: {store.fmt_bytes(len(new))}</div>")
            return
        added, removed = diffing.line_counts(old, new)
        self.fmeta.setText(f"{kind} · +{added} −{removed}")
        lines = diffing.unified(rel, old, new)
        if not lines:
            self.diff.setHtml(f"<div style='color:{MUTED};font-family:sans-serif;padding:18px'>"
                              "The file content is unchanged.</div>")
            return
        out = []
        for kind_l, text in lines:
            color = {"+": GREEN, "-": RED, "@": "#7dd3fc"}.get(kind_l, "#8fa6c0")
            bg = {"+": "#10231a", "-": "#2a1417", "@": "#0a2033"}.get(kind_l, "transparent")
            safe = html.escape(text).replace(" ", "&nbsp;") or "&nbsp;"
            out.append(
                f"<div style='background:{bg};color:{color};white-space:pre;"
                f"font-family:JetBrains Mono,Consolas,monospace;font-size:12px'>{safe}</div>"
            )
        self.diff.setHtml("".join(out))
