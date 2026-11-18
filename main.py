"""TimeCapsule 0.0 - a time machine for project folders."""

import os
import sys
import time

from PySide6.QtCore import Qt, QTimer
from PySide6.QtGui import QKeySequence, QShortcut
from PySide6.QtWidgets import (
    QApplication, QHBoxLayout, QLabel, QMainWindow, QMessageBox, QPushButton,
    QStackedWidget, QVBoxLayout, QWidget,
)

import store
from theme import ACCENT, MUTED, QSS, chip, hbox, hline
from ui_compare import CompareView
from ui_dashboard import Dashboard, HeroHeader, StatusBar
from ui_dialogs import (
    ConfirmDialog, NewBranchDialog, NewProjectDialog, NewSnapshotDialog, RestoreDialog,
)
from ui_settings import SettingsView
from ui_sidebar import Sidebar
from ui_stats import StatsView
from ui_timeline import TimelineView
from widgets import EmptyState

PAGES = {"dashboard": 0, "timeline": 1, "compare": 2, "stats": 3, "settings": 4}
WELCOME = 5


class MainWindow(QMainWindow):
    def __init__(self):
        super().__init__()
        self.setWindowTitle("TimeCapsule")
        self.resize(1440, 900)
        self.setMinimumSize(1120, 720)
        self.pid = None
        self.branch = "main"
        self.projects = []
        self.last_auto = None
        self._overview = None
        self.auto_cfg = store.autosave_config()
        self.timer = QTimer(self)
        self.timer.timeout.connect(self.on_auto_tick)

        root = QWidget()
        root.setObjectName("root")
        outer = QVBoxLayout(root)
        outer.setContentsMargins(0, 0, 0, 0)
        outer.setSpacing(0)

        body = QWidget()
        lay = QHBoxLayout(body)
        lay.setContentsMargins(0, 0, 0, 0)
        lay.setSpacing(0)

        self.side = Sidebar()
        self.side.project_selected.connect(self.select_project)
        self.side.add_project.connect(self.on_new_project)
        self.side.view_changed.connect(self.show_view)
        lay.addWidget(self.side)

        right = QWidget()
        rl = QVBoxLayout(right)
        rl.setContentsMargins(22, 18, 22, 12)
        rl.setSpacing(12)

        self.hero = HeroHeader()
        self.hero.newproj.clicked.connect(self.on_new_project)
        self.hero.newsnap.clicked.connect(self.on_new_snapshot)
        self.hero.search.textChanged.connect(lambda text: self.dash.set_query(text))
        self.hero.search.returnPressed.connect(lambda: self.show_view("dashboard"))
        rl.addWidget(self.hero)

        self.head = QWidget()
        hl = QHBoxLayout(self.head)
        hl.setContentsMargins(2, 2, 2, 0)
        hl.setSpacing(12)
        col = QVBoxLayout()
        col.setSpacing(2)
        self.ptitle = QLabel("")
        self.ptitle.setObjectName("title")
        self.ppath = QLabel("")
        self.ppath.setObjectName("sub")
        self.pchips = hbox(spacing=8)
        col.addWidget(self.ptitle)
        col.addWidget(self.ppath)
        col.addSpacing(3)
        col.addLayout(self.pchips)
        hl.addLayout(col, 1)
        rl.addWidget(self.head)

        self.stack = QStackedWidget()
        self.dash = Dashboard()
        self.dash.new_snapshot.connect(self.on_new_snapshot)
        self.dash.compare.connect(self.compare_latest)
        self.dash.restore.connect(self.restore_latest)
        self.dash.open_timeline.connect(lambda sid: self.open_timeline(sid))
        self.timeline = TimelineView()
        self.timeline.new_snapshot.connect(self.on_new_snapshot)
        self.timeline.restore_all.connect(self.on_restore_all)
        self.timeline.restore_file.connect(self.on_restore_file)
        self.timeline.compare_requested.connect(self.on_compare)
        self.timeline.note_saved.connect(lambda: self.refresh(keep=True))
        self.timeline.delete_requested.connect(self.on_delete)
        self.timeline.branch_selected.connect(self.on_branch_selected)
        self.timeline.new_branch.connect(self.on_new_branch)
        self.compare = CompareView()
        self.stats = StatsView()
        self.settings = SettingsView()
        self.settings.save_requested.connect(self.save_settings)
        for w in (self.dash, self.timeline, self.compare, self.stats, self.settings):
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
        outer.addWidget(body, 1)

        self.status = StatusBar()
        self.status.settings_clicked.connect(lambda: self.show_view("settings"))
        self.status.autosave_toggled.connect(self.set_autosave)
        outer.addWidget(self.status)
        self.setCentralWidget(root)

        for seq, slot in (
            ("Ctrl+S", self.on_new_snapshot),
            ("Ctrl+Shift+C", self.compare_latest),
            ("Ctrl+R", self.restore_latest),
            ("Ctrl+T", lambda: self.show_view("timeline")),
            ("Ctrl+D", lambda: self.show_view("dashboard")),
            ("Ctrl+K", self.focus_search),
        ):
            QShortcut(QKeySequence(seq), self).activated.connect(slot)

        self.refresh()
        self.apply_autosave()

    # ------------------------------------------------------------------ state
    def totals(self, overview=None):
        """Sidebar counters built from a single library pass per refresh."""
        view = overview or store.library_stats()
        counts = dict(view["counts"])
        counts["__storage__"] = f"Project storage · {store.fmt_bytes(view['used'])}"
        return counts

    def focus_search(self):
        self.show_view("dashboard")
        self.hero.search.setFocus()
        self.hero.search.selectAll()

    def _sync_head(self):
        index = self.stack.currentIndex()
        self.head.setVisible(self.pid is not None and index not in (PAGES["dashboard"], WELCOME))

    def refresh(self, select=None, keep=False):
        self.projects = store.load_projects()
        ids = [p["id"] for p in self.projects]
        if self.pid not in ids:
            self.pid = ids[0] if ids else None
        view = store.library_stats()
        self._overview = view
        self.side.reload(self.projects, self.pid, self.totals(view))
        has = self.pid is not None
        self.hero.newsnap.setEnabled(has)
        if not has:
            self.side.set_view("dashboard")
            self.stack.setCurrentIndex(WELCOME)
            self._sync_head()
            self._overview = view
            self.dash.load(None)
            self.status.update_state(None, self.auto_cfg, self.timer.remainingTime(), self.last_auto)
            return
        self.branch = store.active_branch(self.pid)
        proj = store.get_project(self.pid)
        self.ptitle.setText(proj["name"])
        self.ppath.setText(proj["path"])
        snaps = store.load_snapshots(self.pid)
        while self.pchips.count():
            item = self.pchips.takeAt(0)
            w = item.widget()
            if w:
                w.deleteLater()
        self.pchips.addWidget(chip(f"{len(snaps)} snapshots", ACCENT, "#0e3350"))
        if snaps:
            self.pchips.addWidget(chip(f"latest {store.ago(snaps[-1]['ts'])}", MUTED))
            self.pchips.addWidget(chip(store.fmt_bytes(store.storage_bytes(self.pid)), MUTED))
        else:
            self.pchips.addWidget(chip("no snapshots yet", MUTED))
        self.pchips.addStretch(1)
        index = self.stack.currentIndex()
        if index == WELCOME:
            self.side.set_view("dashboard")
            self.stack.setCurrentIndex(PAGES["dashboard"])
            index = PAGES["dashboard"]
        if index == PAGES["dashboard"]:
            self.dash.load(self.pid, self.branch, view)
        if index == PAGES["timeline"] or select is not None:
            self.timeline.load(self.pid, select, self.branch)
        if index == PAGES["compare"]:
            self.compare.load(self.pid, branch=self.branch)
        elif index == PAGES["stats"]:
            self.stats.load(self.pid, self.branch)
        self._sync_head()
        self.status.update_state(self.pid, self.auto_cfg, self.timer.remainingTime(), self.last_auto)

    def select_project(self, pid):
        if pid == self.pid:
            return
        self.pid = pid
        self.branch = store.active_branch(pid)
        self.side.set_view("dashboard")
        self.stack.setCurrentIndex(PAGES["dashboard"])
        self.refresh()

    def show_view(self, key):
        if key not in PAGES or not self.pid:
            return
        self.side.set_view(key)
        if key == "dashboard":
            self.dash.load(self.pid, self.branch, self._overview)
            self.stack.setCurrentIndex(PAGES["dashboard"])
            self._sync_head()
            self.status.update_state(self.pid, self.auto_cfg, self.timer.remainingTime(),
                                     self.last_auto)
            return
        if key == "timeline":
            self.stack.setCurrentIndex(PAGES["timeline"])
            self.refresh(keep=True)
            return
        self.stack.setCurrentIndex(PAGES[key])
        self._sync_head()
        if key == "compare":
            self.compare.load(self.pid, branch=self.branch)
        elif key == "stats":
            self.stats.load(self.pid, self.branch)
        elif key == "settings":
            self.settings.load(self.auto_cfg, self.last_auto)

    def open_timeline(self, sid=None):
        if not self.pid:
            return
        self.side.set_view("timeline")
        self.stack.setCurrentIndex(PAGES["timeline"])
        self.timeline.load(self.pid, sid or None, self.branch)
        self._sync_head()

    def compare_latest(self):
        if not self.pid:
            return
        chain = store.branch_chain(self.pid, self.branch)
        if len(chain) < 2:
            return
        self.on_compare(chain[-2]["id"], chain[-1]["id"])

    def restore_latest(self):
        if not self.pid:
            return
        chain = store.branch_chain(self.pid, self.branch)
        if chain:
            self.on_restore_all(chain[-1]["id"])

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
            snap = store.take_snapshot(self.pid, note=note, title=title, branch=self.branch)
        except store.StoreError as e:
            QMessageBox.warning(self, "TimeCapsule", str(e))
            return
        self.side.set_view("timeline")
        self.stack.setCurrentIndex(PAGES["timeline"])
        self.refresh(select=snap["id"])

    def on_branch_selected(self, bid):
        if not self.pid:
            return
        store.set_active_branch(self.pid, bid)
        self.branch = bid
        self.refresh()

    def on_new_branch(self, from_sid):
        if not self.pid:
            return
        chain = store.branch_chain(self.pid, self.branch)
        from_sid = from_sid or (chain[-1]["id"] if chain else "")
        if not from_sid:
            QMessageBox.information(
                self, "New branch",
                "Capture at least one snapshot first — a branch starts from a moment in the timeline.",
            )
            return
        snap = store.get_snapshot(self.pid, from_sid)
        branch_name = (store.get_branch(self.pid, self.branch) or {}).get("name", "main")
        dlg = NewBranchDialog(self, snap, branch_name)
        if not dlg.exec():
            return
        try:
            branch = store.add_branch(self.pid, dlg.values(), from_sid)
        except store.StoreError as e:
            QMessageBox.warning(self, "TimeCapsule", str(e))
            return
        store.set_active_branch(self.pid, branch["id"])
        self.branch = branch["id"]
        self.side.set_view("timeline")
        self.stack.setCurrentIndex(PAGES["timeline"])
        self.refresh()

    def save_settings(self):
        cfg = store.settings()
        cfg.update(self.settings.values())
        store.save_settings(cfg)
        self.apply_autosave()
        self.settings.mark_saved()

    def set_autosave(self, on):
        cfg = store.settings()
        cfg["auto_save"] = bool(on)
        store.save_settings(cfg)
        self.apply_autosave()
        self.refresh(keep=True)

    # ---------------------------------------------------------------- auto-save
    def apply_autosave(self):
        self.auto_cfg = store.autosave_config()
        if self.auto_cfg["enabled"]:
            self.timer.start(self.auto_cfg["minutes"] * 60_000)
        else:
            self.timer.stop()
        self.settings.set_state(self.auto_cfg, self.last_auto)
        self.status.update_state(self.pid, self.auto_cfg, self.timer.remainingTime(),
                                 self.last_auto)

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
        self.status.update_state(self.pid, self.auto_cfg, self.timer.remainingTime(),
                                 self.last_auto)

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
        self.compare.load(self.pid, a, b, self.branch)
        self._sync_head()


def main():
    app = QApplication(sys.argv)
    app.setApplicationName("TimeCapsule")
    app.setStyleSheet(QSS)
    win = MainWindow()
    win.show()
    return app.exec()


if __name__ == "__main__":
    sys.exit(main())
