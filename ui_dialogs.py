"""Dialogs: add a project, describe a snapshot, confirm a restore."""

import os

from PySide6.QtCore import Qt
from PySide6.QtWidgets import (
    QCheckBox, QDialog, QFileDialog, QFrame, QHBoxLayout, QLabel, QLayout, QLineEdit,
    QPlainTextEdit, QPushButton, QScrollArea, QVBoxLayout, QWidget,
)

import store
from theme import ACCENT, BORDER, DIM, GREEN, MUTED, RED, TEXT, hbox, hline, label, vbox
from widgets import StatusPill


class BaseDialog(QDialog):
    def __init__(self, parent=None, title="", width=520):
        super().__init__(parent)
        self.setWindowTitle(title)
        self.setMinimumWidth(width)
        self.setModal(True)
        self.root = QVBoxLayout(self)
        self.root.setContentsMargins(22, 20, 22, 18)
        self.root.setSpacing(14)
        head = QLabel(title)
        head.setObjectName("title")
        self.root.addWidget(head)

    def finish(self, ok_text, cancel_text="Cancel", primary=True):
        row = hbox(spacing=10)
        row.addStretch(1)
        cancel = QPushButton(cancel_text)
        cancel.setObjectName("ghost")
        cancel.setCursor(Qt.PointingHandCursor)
        cancel.clicked.connect(self.reject)
        ok = QPushButton(ok_text)
        ok.setObjectName("primary" if primary else "ghost")
        ok.setCursor(Qt.PointingHandCursor)
        ok.setDefault(True)
        ok.clicked.connect(self.accept)
        row.addWidget(cancel)
        row.addWidget(ok)
        self.root.addLayout(row)
        self.ok_button = ok
        return ok


def _field(caption, widget):
    box = QVBoxLayout()
    box.setSpacing(6)
    cap = QLabel(caption)
    cap.setObjectName("tiny")
    box.addWidget(cap)
    if isinstance(widget, QLayout):
        box.addLayout(widget)
    else:
        box.addWidget(widget)
    return box


class AddProjectDialog(BaseDialog):
    def __init__(self, parent=None):
        super().__init__(parent, "Add a project", 560)
        self.path = QLineEdit()
        self.path.setPlaceholderText("/home/you/projects/my-app")
        self.name = QLineEdit()
        self.name.setPlaceholderText("Optional display name")
        browse = QPushButton("Browse…")
        browse.setObjectName("ghost")
        browse.setCursor(Qt.PointingHandCursor)
        browse.clicked.connect(self.pick)
        prow = hbox(self.path, browse, spacing=8)
        self.root.addLayout(_field("FOLDER TO WATCH", prow))
        self.root.addLayout(_field("NAME", self.name))
        self.hint = QLabel("TimeCapsule copies file contents into its own store, so snapshots "
                           "survive even if you edit the folder later. Caches, build output and "
                           "files over 2 MB are skipped.")
        self.hint.setObjectName("sub")
        self.hint.setWordWrap(True)
        self.root.addWidget(self.hint)
        self.finish("Add project")

    def pick(self):
        path = QFileDialog.getExistingDirectory(self, "Choose a project folder", os.path.expanduser("~"))
        if path:
            self.path.setText(path)

    def accept(self):
        path = self.path.text().strip()
        if not path or not os.path.isdir(os.path.expanduser(path)):
            self.hint.setText("Pick an existing folder first.")
            self.hint.setStyleSheet(f"color:{RED}")
            return
        super().accept()

    def values(self):
        return self.path.text().strip(), self.name.text().strip()


class NewSnapshotDialog(BaseDialog):
    def __init__(self, parent=None, project=None):
        super().__init__(parent, "Capture a snapshot", 560)
        self.project = project
        self.title = QLineEdit()
        self.title.setPlaceholderText("e.g. Before refactoring the importer")
        self.note = QPlainTextEdit()
        self.note.setPlaceholderText("What is worth remembering about this moment?")
        self.note.setFixedHeight(110)
        self.root.addLayout(_field("TITLE", self.title))
        self.root.addLayout(_field("NOTE", self.note))
        self.preview = QLabel("")
        self.preview.setObjectName("sub")
        self.preview.setWordWrap(True)
        self.root.addWidget(self.preview)
        self.finish("Capture snapshot")
        self.describe()

    def describe(self):
        if not self.project:
            self.preview.setText("")
            return
        try:
            files = store.scan_project(self.project["path"])
        except OSError:
            files = []
        total = sum(size for _rel, _ap, size in files)
        self.preview.setText(
            f"{len(files)} tracked files · {store.fmt_bytes(total)} will be captured. "
            "Existing content is reused from the capsule store, so repeated snapshots stay small."
        )

    def values(self):
        return self.title.text().strip(), self.note.text()


class RestoreDialog(BaseDialog):
    def __init__(self, parent, snap, write, delete, whole=True, rel=None):
        what = "the whole project" if whole else rel
        super().__init__(parent, "Restore " + ("snapshot" if whole else "file"), 620)
        info = QLabel(
            f"Restore <b>{what}</b> to the state captured on <b>{store.fmt_stamp(snap['ts'])}</b>."
        )
        info.setTextFormat(Qt.RichText)
        info.setWordWrap(True)
        self.root.addWidget(info)

        counts = hbox(spacing=8)
        counts.addWidget(StatusPill("added"))
        counts.addWidget(QLabel(f"{len(write)} file{'s' if len(write) != 1 else ''} will be written"))
        counts.addSpacing(10)
        counts.addWidget(StatusPill("deleted"))
        counts.addWidget(QLabel(f"{len(delete)} file{'s' if len(delete) != 1 else ''} will be removed"))
        counts.addStretch(1)
        self.root.addLayout(counts)

        box = QFrame()
        box.setObjectName("card")
        bl = QVBoxLayout(box)
        bl.setContentsMargins(12, 10, 12, 10)
        bl.setSpacing(4)
        scroll = QScrollArea()
        scroll.setWidgetResizable(True)
        inner = QWidget()
        col = QVBoxLayout(inner)
        col.setContentsMargins(0, 0, 0, 0)
        col.setSpacing(3)
        shown = 0
        for rel_path in write[:80]:
            col.addWidget(label(("✎  " if whole else "↺  ") + rel_path, "sub"))
            shown += 1
        for rel_path in delete[:80]:
            col.addWidget(label("✕  " + rel_path, "sub"))
            shown += 1
        if shown == 0:
            col.addWidget(label("The folder already matches this snapshot.", "sub"))
        if len(write) + len(delete) > shown:
            col.addWidget(label(f"… and {len(write) + len(delete) - shown} more", "sub"))
        col.addStretch(1)
        scroll.setWidget(inner)
        scroll.setMinimumHeight(170)
        bl.addWidget(scroll)
        self.root.addWidget(box)

        self.safety = QCheckBox("Capture the current state first, so this restore can be undone")
        self.safety.setChecked(True)
        self.root.addWidget(self.safety)
        self.finish("Restore now")

    def wants_safety(self):
        return self.safety.isChecked()


class ConfirmDialog(BaseDialog):
    def __init__(self, parent, title, text, ok_text, danger=True):
        super().__init__(parent, title, 480)
        msg = QLabel(text)
        msg.setWordWrap(True)
        msg.setObjectName("sub")
        self.root.addWidget(msg)
        self.finish(ok_text, primary=not danger)
