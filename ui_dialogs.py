"""Dialogs: create a project, describe a snapshot, confirm restores."""

import os

from PySide6.QtCore import Qt
from PySide6.QtWidgets import (
    QCheckBox, QDialog, QFileDialog, QFrame, QHBoxLayout, QLabel, QLayout, QLineEdit,
    QPlainTextEdit, QPushButton, QScrollArea, QVBoxLayout, QWidget,
)

import store
from theme import RED, hbox, label
from widgets import StatusPill


class BaseDialog(QDialog):
    def __init__(self, parent=None, title="", width=520, modal=True):
        super().__init__(parent)
        self.setWindowTitle(title)
        self.setMinimumWidth(width)
        self.setModal(modal)
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


class NewProjectDialog(BaseDialog):
    """Create a project: a name plus the folder TimeCapsule should watch."""

    def __init__(self, parent=None):
        super().__init__(parent, "New project", 580)
        self.name = QLineEdit()
        self.name.setPlaceholderText("e.g. The importer rewrite")
        self.path = QLineEdit()
        self.path.setPlaceholderText("/home/you/projects/my-app")
        self.path.textChanged.connect(self._suggest_name)
        browse = QPushButton("Browse…")
        browse.setObjectName("ghost")
        browse.setCursor(Qt.PointingHandCursor)
        browse.clicked.connect(self.pick)
        prow = hbox(self.path, browse, spacing=8)

        self.create = QCheckBox("Create the folder if it does not exist yet")
        self.create.setChecked(True)

        self.root.addLayout(_field("NAME", self.name))
        self.root.addLayout(_field("PROJECT FOLDER", prow))
        self.root.addWidget(self.create)
        self.hint = QLabel(
            "A project is a folder TimeCapsule keeps an eye on. File contents are copied into its "
            "own store, so snapshots survive later edits. Caches, build output and files over 2 MB "
            "are skipped."
        )
        self.hint.setObjectName("sub")
        self.hint.setWordWrap(True)
        self.root.addWidget(self.hint)
        self.finish("Create project")

    def _suggest_name(self, text):
        if self.name.text().strip():
            return
        base = os.path.basename(text.rstrip("/"))
        if base:
            self.name.setText(base)

    def pick(self):
        path = QFileDialog.getExistingDirectory(self, "Choose a project folder", os.path.expanduser("~"))
        if path:
            self.path.setText(path)

    def folder(self):
        return os.path.abspath(os.path.expanduser(self.path.text().strip()))

    def accept(self):
        if not self.path.text().strip():
            return self._fail("Choose a folder for this project first.")
        folder = self.folder()
        if os.path.exists(folder) and not os.path.isdir(folder):
            return self._fail("That path is a file, not a folder.")
        if not os.path.isdir(folder) and not self.create.isChecked():
            return self._fail("That folder does not exist — tick the box to create it, or browse elsewhere.")
        super().accept()

    def _fail(self, text):
        self.hint.setText(text)
        self.hint.setStyleSheet(f"color:{RED}")

    def values(self):
        return self.folder(), self.name.text().strip(), self.create.isChecked()


class NewBranchDialog(BaseDialog):
    """Name a new branch of an existing project, starting at a chosen moment."""

    def __init__(self, parent=None, snap=None, branch_name="main"):
        super().__init__(parent, "New branch", 540)
        self.name = QLineEdit()
        self.name.setPlaceholderText("e.g. try-a-different-approach")
        self.root.addLayout(_field("BRANCH NAME", self.name))
        when = store.fmt_stamp(snap["ts"]) if snap else "the latest moment"
        info = QLabel(
            f"The branch starts at {when} and keeps the history it grew from. Snapshots you "
            f"capture on it are stored in the same project and never touch “{branch_name}”."
        )
        info.setObjectName("sub")
        info.setWordWrap(True)
        self.root.addWidget(info)
        self.hint = QLabel("You can switch between branches at any time in the timeline.")
        self.hint.setObjectName("sub")
        self.hint.setWordWrap(True)
        self.root.addWidget(self.hint)
        self.finish("Create branch")

    def accept(self):
        if not self.name.text().strip():
            self.hint.setText("Give the branch a name.")
            self.hint.setStyleSheet(f"color:{RED}")
            return
        super().accept()

    def values(self):
        return self.name.text().strip()


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
            "Existing content is reused from the project store, so repeated snapshots stay small."
        )

    def values(self):
        return self.title.text().strip(), self.note.toPlainText()


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
