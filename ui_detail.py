"""Snapshot detail: the note, the files that changed and the restore actions."""

from PySide6.QtCore import Qt, Signal
from PySide6.QtGui import QColor
from PySide6.QtWidgets import (
    QFrame, QHBoxLayout, QLabel, QLineEdit, QPlainTextEdit, QPushButton, QScrollArea,
    QSizePolicy, QVBoxLayout, QWidget,
)

import store
from theme import ACCENT, BORDER, DIM, GREEN, MUTED, RED, TEXT, chip, hbox, hline, label, vbox
from widgets import MiniBar, StatusPill


def _size_delta(kind, old, new):
    if kind == "added":
        return f"+{store.fmt_bytes(new)}", GREEN
    if kind == "deleted":
        return f"−{store.fmt_bytes(old)}", RED
    diff = new - old
    if diff == 0:
        return "same size", DIM
    sign = "+" if diff > 0 else "−"
    return f"{sign}{store.fmt_bytes(abs(diff))}", GREEN if diff > 0 else RED


class FileRow(QFrame):
    restore = Signal(str)

    def __init__(self, rel, kind, old_size=0, new_size=0, parent=None):
        super().__init__(parent)
        self.rel = rel
        self.setObjectName("card")
        self.setStyleSheet("QFrame#card{border-radius:11px;background:#0d1e2f;border:1px solid #14314a}")
        lay = QHBoxLayout(self)
        lay.setContentsMargins(12, 9, 10, 9)
        lay.setSpacing(10)
        path = QLabel(rel)
        path.setStyleSheet("font-family:'JetBrains Mono',Consolas,monospace;font-size:12px")
        path.setToolTip(rel)
        path.setSizePolicy(QSizePolicy.Expanding, QSizePolicy.Preferred)
        lay.addWidget(StatusPill(kind), 0)
        lay.addWidget(path, 1)
        text, color = _size_delta(kind, old_size, new_size)
        size = QLabel(text)
        size.setStyleSheet(f"color:{color};font-size:11px;font-weight:700")
        lay.addWidget(size, 0)
        btn = QPushButton("↺")
        btn.setToolTip(f"Restore {rel} from this snapshot")
        btn.setCursor(Qt.PointingHandCursor)
        btn.setFixedSize(26, 26)
        btn.setStyleSheet(
            "QPushButton{background:#102a42;border:1px solid #1c405e;border-radius:7px;padding:0;font-size:13px;color:#c8ccd4}"
            "QPushButton:hover{border-color:#38bdf8;color:#38bdf8}"
        )
        btn.clicked.connect(lambda: self.restore.emit(self.rel))
        lay.addWidget(btn, 0)


class SnapshotDetail(QWidget):
    restore_all = Signal(str)
    restore_file = Signal(str, str)
    compare_previous = Signal(str)
    note_saved = Signal(str)
    delete_requested = Signal(str)

    def __init__(self, parent=None):
        super().__init__(parent)
        self.pid = None
        self.snap = None
        self.prev = None
        self.rows = []
        root = QVBoxLayout(self)
        root.setContentsMargins(0, 0, 0, 0)
        root.setSpacing(14)

        head = QFrame()
        head.setObjectName("card")
        hl = QVBoxLayout(head)
        hl.setContentsMargins(18, 14, 18, 14)
        hl.setSpacing(5)
        self.time = QLabel("")
        self.time.setObjectName("bigtime")
        self.date = QLabel("")
        self.date.setObjectName("sub")
        self.meta = QLabel("")
        self.meta.setObjectName("sub")
        self.chips = hbox(spacing=8)
        hl.addWidget(self.time)
        hl.addWidget(self.date)
        hl.addWidget(self.meta)
        hl.addSpacing(2)
        hl.addLayout(self.chips)
        root.addWidget(head)

        note_card = QFrame()
        note_card.setObjectName("card")
        nl = QVBoxLayout(note_card)
        nl.setContentsMargins(18, 14, 18, 14)
        nl.setSpacing(8)
        nh = hbox(spacing=8)
        nt = QLabel("Note")
        nt.setStyleSheet("font-size:14px;font-weight:800")
        nh.addWidget(nt)
        self.note_state = QLabel("")
        self.note_state.setObjectName("sub")
        nh.addWidget(self.note_state)
        nh.addStretch(1)
        self.save = QPushButton("Save note")
        self.save.setObjectName("ghost")
        self.save.setCursor(Qt.PointingHandCursor)
        self.save.setEnabled(False)
        self.save.clicked.connect(self._save_note)
        nh.addWidget(self.save)
        self.note = QPlainTextEdit()
        self.note.setPlaceholderText("What happened at this moment? Why did you save it?")
        self.note.setFixedHeight(64)
        self.note.textChanged.connect(self._dirty)
        nl.addLayout(nh)
        nl.addWidget(self.note)
        root.addWidget(note_card)

        changes = QFrame()
        changes.setObjectName("card")
        cl = QVBoxLayout(changes)
        cl.setContentsMargins(18, 14, 18, 14)
        cl.setSpacing(10)
        ch = hbox(spacing=10)
        ct = QLabel("Changes in this snapshot")
        ct.setStyleSheet("font-size:14px;font-weight:800")
        self.change_sum = QLabel("")
        self.change_sum.setObjectName("sub")
        ch.addWidget(ct)
        ch.addWidget(self.change_sum)
        ch.addStretch(1)
        self.filter = QLineEdit()
        self.filter.setPlaceholderText("Filter files…")
        self.filter.setFixedWidth(180)
        self.filter.textChanged.connect(lambda _t: self._rebuild_rows())
        ch.addWidget(self.filter)
        cl.addLayout(ch)
        self.scroll = QScrollArea()
        self.scroll.setWidgetResizable(True)
        self.body = QWidget()
        self.body.col = QVBoxLayout(self.body)
        self.body.col.setContentsMargins(0, 0, 0, 0)
        self.body.col.setSpacing(6)
        self.scroll.setWidget(self.body)
        self.scroll.setMinimumHeight(80)
        cl.addWidget(self.scroll, 1)
        self.same = QLabel("")
        self.same.setObjectName("sub")
        cl.addWidget(self.same)
        root.addWidget(changes, 1)

        foot = hbox(spacing=10)
        self.cmp = QPushButton("⇄  Compare with previous")
        self.cmp.setObjectName("ghost")
        self.cmp.setCursor(Qt.PointingHandCursor)
        self.cmp.clicked.connect(lambda: self.snap and self.compare_previous.emit(self.snap["id"]))
        self.del_btn = QPushButton("Delete snapshot")
        self.del_btn.setObjectName("danger")
        self.del_btn.setCursor(Qt.PointingHandCursor)
        self.del_btn.clicked.connect(lambda: self.snap and self.delete_requested.emit(self.snap["id"]))
        self.restore = QPushButton("↺  Restore this snapshot")
        self.restore.setObjectName("primary")
        self.restore.setCursor(Qt.PointingHandCursor)
        self.restore.clicked.connect(lambda: self.snap and self.restore_all.emit(self.snap["id"]))
        foot.addWidget(self.cmp)
        foot.addStretch(1)
        foot.addWidget(self.del_btn)
        foot.addWidget(self.restore)
        root.addLayout(foot)

    # ------------------------------------------------------------------ load
    def load(self, pid, snap, prev):
        self.pid, self.snap, self.prev = pid, snap, prev
        self.time.setText(store.fmt_time(snap["ts"]))
        self.date.setText(f"{store.fmt_date(snap['ts'])} · {store.ago(snap['ts'])}")
        title = snap.get("title") or "Untitled moment"
        self.meta.setText(f"{title}  ·  {snap['count']} files  ·  {store.fmt_bytes(snap['bytes'])}")
        while self.chips.count():
            item = self.chips.takeAt(0)
            w = item.widget()
            if w:
                w.deleteLater()
        base = prev or {"files": {}}
        diff = store.compare(base, snap)
        c = diff["counts"]
        if snap.get("kind") == "auto":
            self.chips.addWidget(chip("auto-save", MUTED))
        for kind in ("modified", "added", "deleted"):
            if c[kind]:
                color = {"modified": ACCENT, "added": GREEN, "deleted": RED}[kind]
                self.chips.addWidget(chip(f"{c[kind]} {kind}", color, QColor(color).darker(340).name()))
        if not any(c[k] for k in ("modified", "added", "deleted")):
            self.chips.addWidget(chip("identical to previous", MUTED))
        if prev is None:
            self.chips.addWidget(chip("first snapshot", MUTED))
        else:
            self.chips.addWidget(chip("vs previous", MUTED))
        self.chips.addStretch(1)

        self.note.setEnabled(True)
        self.note.setPlaceholderText("What happened at this moment? Why did you save it?")
        self.note.blockSignals(True)
        self.note.setPlainText(snap.get("note", ""))
        self.note.blockSignals(False)
        self.note_state.setText("")
        self.save.setEnabled(False)
        self.cmp.setEnabled(prev is not None)
        self._rebuild_rows()

    def _dirty(self):
        if not self.snap:
            return
        changed = self.note.toPlainText().strip() != (self.snap.get("note") or "").strip()
        self.save.setEnabled(changed)
        self.note_state.setText("unsaved changes" if changed else "")

    def _save_note(self):
        if not self.snap:
            return
        updated = store.update_snapshot(self.pid, self.snap["id"], note=self.note.toPlainText())
        if updated:
            self.snap = updated
        self.save.setEnabled(False)
        self.note_state.setText("saved")
        self.note_saved.emit(self.snap["id"])

    def _rebuild_rows(self):
        for row in self.rows:
            row.setParent(None)
            row.deleteLater()
        self.rows = []
        if not self.snap:
            return
        base = self.prev or {"files": {}}
        diff = store.compare(base, self.snap)
        c = diff["counts"]
        self.change_sum.setText(f"{c['added']} added · {c['modified']} modified · {c['deleted']} deleted")
        self.same.setText(f"{c['same']} unchanged file{'s' if c['same'] != 1 else ''} not shown")
        query = self.filter.text().strip().lower()
        rows = store.ordered_changes(diff)
        if query:
            rows = [(rel, k) for rel, k in rows if query in rel.lower()]
        if not rows:
            msg = QLabel("No file changes here." if not query else "Nothing matches that filter.")
            msg.setObjectName("empty")
            self.body.col.addWidget(msg)
            self.rows.append(msg)
            self.body.col.addStretch(1)
            return
        for rel, kind in rows:
            old = base["files"].get(rel, {}).get("size", 0)
            new = self.snap["files"].get(rel, {}).get("size", 0)
            row = FileRow(rel, kind, old, new)
            row.restore.connect(lambda r=rel: self.restore_file.emit(self.snap["id"], r))
            self.body.col.addWidget(row)
            self.rows.append(row)
        self.body.col.addStretch(1)
