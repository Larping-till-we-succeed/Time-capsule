"""TimeCapsule 0.0 - a time machine for project folders."""

import os
import sys
import time

from PySide6.QtCore import Qt, QTimer
from PySide6.QtWidgets import (
    QApplication, QHBoxLayout, QLabel, QMainWindow, QMessageBox, QPushButton,
    QStackedWidget, QVBoxLayout, QWidget,
)

import store
from theme import ACCENT, MUTED, QSS, chip, hbox
from ui_compare import CompareView
from ui_dialogs import (
    ConfirmDialog, NewProjectDialog, NewSnapshotDialog, RestoreDialog,
)
from ui_settings import SettingsView
from ui_sidebar import Sidebar
from ui_stats import StatsView
from ui_timeline import TimelineView
from widgets import EmptyState

PAGES = {"timeline": 0, "compare": 1, "stats": 2, "settings": 3}
WELCOME = 4


class MainWindow(QMainWindow):
    def __init__(self):
        super().__init__()
        self.setWindowTitle("TimeCapsule")
        self.resize(1440, 900)
        self.pid = None
        self.projects = []
        self.last_auto = None
        self.auto_cfg = store.autosave_config()
        self.timer = QTimer(self)
        self.timer.timeout.connect(self.on_auto_tick)

        root = QWidget()
        root.setObjectName("root")
        lay = QHBoxLayout(root)
        lay.setContentsMargins(0, 0, 0, 0)
        lay.setSpacing(0)

        self.side = Sidebar()
        self.side.project_selected.connect(self.select_project)
        self.side.add_project.connect(self.on_new_project)
        self.side.view_changed.connect(self.show_view)
        lay.addWidget(self.side)

        right = QWidget()
        rl = QVBoxLayout(right)
        rl.setContentsMargins(28, 22, 28, 22)
        rl.setSpacing(16)

        self.head = QWidget()
        hl = QHBoxLayout(self.head)
        hl.setContentsMargins(0, 0, 0, 0)
        hl.setSpacing(12)
        col = QVBoxLayout()
        col.setSpacing(3)
        self.ptitle = QLabel("")
        self.ptitle.setObjectName("title")
        self.ppath = QLabel("")
        self.ppath.setObjectName("sub")
        self.pchips = hbox(spacing=8)
        col.addWidget(self.ptitle)
        col.addWidget(self.ppath)
        col.addSpacing(4)
        col.addLayout(self.pchips)
        hl.addLayout(col, 1)
        self.newprojbtn = QPushButton("＋   New project")
        self.newprojbtn.setObjectName("ghost")
        self.newprojbtn.setCursor(Qt.PointingHandCursor)
        self.newprojbtn.setMinimumHeight(42)
        self.newprojbtn.setToolTip("Create a project and start capturing it")
        self.newprojbtn.clicked.connect(self.on_new_project)
        self.snapbtn = QPushButton("＋   New snapshot")
        self.snapbtn.setObjectName("primary")
        self.snapbtn.setCursor(Qt.PointingHandCursor)
        self.snapbtn.setMinimumHeight(42)
        self.snapbtn.clicked.connect(self.on_new_snapshot)
        hl.addWidget(self.newprojbtn, 0, Qt.AlignTop)
        hl.addWidget(self.snapbtn, 0, Qt.AlignTop)
        rl.addWidget(self.head)

        self.stack = QStackedWidget()
        self.timeline = TimelineView()
        self.timeline.new_snapshot.connect(self.on_new_snapshot)
        self.timeline.restore_all.connect(self.on_restore_all)
        self.timeline.restore_file.connect(self.on_restore_file)
        self.timeline.compare_requested.connect(self.on_compare)
        self.timeline.note_saved.connect(lambda: self.refresh(keep=True))
        self.timeline.delete_requested.connect(self.on_delete)
        self.compare = CompareView()
        self.stats = StatsView()
        self.settings = SettingsView()
        self.settings.save_requested.connect(self.save_settings)
        for w in (self.timeline, self.compare, self.stats, self.settings):
            self.stack.addWidget(w)

        self.welcome = QWidget()
        wl = QVBoxLayout(self.welcome)
        wl.setContentsMargins(0, 0, 0, 0)
        state = EmptyState(
            "⏳",
            "A time machine for your projects",
            "Create a project for a folder, capture snapshots whenever something matters, "
            "and travel back to any moment — whole projects or single files.",
            "Create your first project",
        )
        state.clicked.connect(self.on_new_project)
        wl.addWidget(state)
        self.stack.addWidget(self.welcome)
        rl.addWidget(self.stack, 1)
        lay.addWidget(right, 1)
        self.setCentralWidget(root)

        self.refresh()
        self.apply_autosave()

    # ------------------------------------------------------------------ state
    def totals(self):
        counts = {}
        used = 0
        for p in self.projects:
            counts[p["id"]] = len(store.load_snapshots(p["id"]))
            used += store.storage_bytes(p["id"])
        counts["__storage__"] = f"Project storage · {store.fmt_bytes(used)}"
        return counts

    def refresh(self, select=None, keep=False):
        self.projects = store.load_projects()
        ids = [p["id"] for p in self.projects]
        if self.pid not in ids:
            self.pid = ids[0] if ids else None
        self.side.reload(self.projects, self.pid, self.totals())
        has = self.pid is not None
        self.head.setVisible(has)
        self.snapbtn.setEnabled(has)
        if not has:
            self.stack.setCurrentIndex(WELCOME)
            self.side.set_view("timeline")
            return
        proj = store.get_project(self.pid)
        self.ptitle.setText(proj["name"])
        self.ppath.setText(proj["path"])
        snaps = store.load_snapshots(self.pid)
        while self.pchips.count():
            item = self.pchips.takeAt(0)
            w = item.widget()
            if w:
                w.deleteLater()
        self.pchips.addWidget(chip(f"{len(snaps)} snapshots", ACCENT, "#2a2112"))
        if snaps:
            self.pchips.addWidget(chip(f"latest {store.ago(snaps[-1]['ts'])}", MUTED))
            self.pchips.addWidget(chip(store.fmt_bytes(store.storage_bytes(self.pid)), MUTED))
        else:
            self.pchips.addWidget(chip("no snapshots yet", MUTED))
        self.pchips.addStretch(1)
        self.timeline.load(self.pid, select)
        index = self.stack.currentIndex()
        if index == PAGES["compare"]:
            self.compare.load(self.pid)
        elif index == PAGES["stats"]:
            self.stats.load(self.pid)

    def select_project(self, pid):
        if pid == self.pid:
            return
        self.pid = pid
        self.side.set_view("timeline")
        self.stack.setCurrentIndex(PAGES["timeline"])
        self.refresh()

    def show_view(self, key):
        if key not in PAGES or not self.pid:
            if self.pid:
                self.side.set_view("timeline")
            return
        self.stack.setCurrentIndex(PAGES[key])
        if key == "compare":
            self.compare.load(self.pid)
        elif key == "stats":
            self.stats.load(self.pid)
        elif key == "settings":
            self.settings.load(self.auto_cfg, self.last_auto)
        else:
            self.refresh(keep=True)

    # ---------------------------------------------------------------- actions
    def on_new_project(self):
        dlg = NewProjectDialog(self)
        if not dlg.exec():
            return
        folder, name, create = dlg.values()
        if create and not os.path.isdir(folder):
            try:
                os.makedirs(folder, exist_ok=True)
            except OSError as e:
                QMessageBox.warning(self, "TimeCapsule", f"Could not create that folder:\n{e}")
                return
        try:
            proj = store.add_project(folder, name)
        except store.StoreError as e:
            QMessageBox.warning(self, "TimeCapsule", str(e))
            return
        self.pid = proj["id"]
        self.side.set_view("timeline")
        self.stack.setCurrentIndex(PAGES["timeline"])
        self.refresh()

    def on_new_snapshot(self):
        if not self.pid:
            return
        proj = store.get_project(self.pid)
        dlg = NewSnapshotDialog(self, proj)
        if not dlg.exec():
            return
        title, note = dlg.values()
        try:
            snap = store.take_snapshot(self.pid, note=note, title=title)
        except store.StoreError as e:
            QMessageBox.warning(self, "TimeCapsule", str(e))
            return
        self.side.set_view("timeline")
        self.stack.setCurrentIndex(PAGES["timeline"])
        self.refresh(select=snap["id"])

    def save_settings(self):
        cfg = store.settings()
        cfg.update(self.settings.values())
        store.save_settings(cfg)
        self.apply_autosave()
        self.settings.mark_saved()

    # ---------------------------------------------------------------- auto-save
    def apply_autosave(self):
        self.auto_cfg = store.autosave_config()
        if self.auto_cfg["enabled"]:
            self.timer.start(self.auto_cfg["minutes"] * 60_000)
        else:
            self.timer.stop()
        self.side.set_autosave(self.auto_cfg, self.last_auto)
        self.settings.set_state(self.auto_cfg, self.last_auto)

    def on_auto_tick(self):
        if not self.projects:
            return
        captured = False
        for proj in self.projects:
            pid = proj["id"]
            try:
                if self.auto_cfg["only_changes"] and not store.has_changes(pid):
                    continue
                store.take_snapshot(pid, title="Auto-save", kind="auto")
            except store.StoreError:
                continue
            captured = True
        if captured:
            self.last_auto = time.time()
            self.refresh(keep=True)
        self.side.set_autosave(self.auto_cfg, self.last_auto)
        self.settings.set_state(self.auto_cfg, self.last_auto)

    def on_restore_all(self, sid):
        snap = store.get_snapshot(self.pid, sid)
        if not snap:
            return
        try:
            write, delete = store.preview_restore(self.pid, snap)
        except store.StoreError as e:
            QMessageBox.warning(self, "TimeCapsule", str(e))
            return
        dlg = RestoreDialog(self, snap, write, delete, whole=True)
        if not dlg.exec():
            return
        try:
            actions, safety = store.restore(self.pid, snap, whole=True, safety=dlg.wants_safety())
        except (store.StoreError, OSError) as e:
            QMessageBox.warning(self, "TimeCapsule", f"Restore failed:\n{e}")
            return
        self.refresh()
        QMessageBox.information(
            self, "Restored",
            f"{len(actions)} file{'s' if len(actions) != 1 else ''} restored from {store.fmt_stamp(snap['ts'])}."
            + ("\nThe previous state was kept as a safety snapshot." if safety else ""),
        )

    def on_restore_file(self, sid, rel):
        snap = store.get_snapshot(self.pid, sid)
        if not snap:
            return
        try:
            write, delete = store.preview_restore(self.pid, snap)
        except store.StoreError as e:
            QMessageBox.warning(self, "TimeCapsule", str(e))
            return
        w = [rel] if rel in write else []
        dl = [rel] if rel in delete else []
        dlg = RestoreDialog(self, snap, w, dl, whole=False, rel=rel)
        if not dlg.exec():
            return
        try:
            actions, _safety = store.restore(self.pid, snap, whole=False, rels=[rel], safety=False)
        except (store.StoreError, OSError) as e:
            QMessageBox.warning(self, "TimeCapsule", f"Restore failed:\n{e}")
            return
        self.refresh(keep=True)
        QMessageBox.information(self, "Restored", rel + (" restored." if actions else " already matches this snapshot."))

    def on_delete(self, sid):
        snap = store.get_snapshot(self.pid, sid)
        if not snap:
            return
        dlg = ConfirmDialog(
            self, "Delete snapshot",
            f"Delete the snapshot from {store.fmt_stamp(snap['ts'])}? "
            "Files already captured by other snapshots stay in the project; this moment’s note disappears.",
            "Delete snapshot",
        )
        if not dlg.exec():
            return
        store.delete_snapshot(self.pid, sid)
        self.refresh()

    def on_compare(self, a, b):
        if not self.pid:
            return
        self.side.set_view("compare")
        self.stack.setCurrentIndex(PAGES["compare"])
        self.compare.load(self.pid, a, b)


def main():
    app = QApplication(sys.argv)
    app.setApplicationName("TimeCapsule")
    app.setStyleSheet(QSS)
    win = MainWindow()
    win.show()
    return app.exec()


if __name__ == "__main__":
    sys.exit(main())
