"""Left sidebar: product mark, project switcher, navigation and storage meter."""

from PySide6.QtCore import Qt, Signal
from PySide6.QtGui import QFontMetrics
from PySide6.QtWidgets import QFrame, QHBoxLayout, QLabel, QPushButton, QScrollArea, QVBoxLayout, QWidget

import store
from theme import ACCENT, DIM, MUTED, SURF, TEXT, hbox, hline, label, vbox
from widgets import ClickFrame

VIEWS = [("timeline", "🕰", "Timeline"), ("compare", "⇄", "Compare"), ("stats", "▤", "Statistics")]


class ProjectRow(ClickFrame):
    def __init__(self, proj, count, active=False, parent=None):
        super().__init__(parent)
        self.pid = proj["id"]
        self.setObjectName("projrow")
        self.setProperty("active", "true" if active else "false")
        lay = QVBoxLayout(self)
        lay.setContentsMargins(12, 10, 12, 10)
        lay.setSpacing(3)
        name = QLabel(proj["name"])
        name.setStyleSheet("font-weight:800;font-size:14px")
        path = QLabel(proj["path"])
        path.setObjectName("sub")
        path.setStyleSheet("font-size:11px")
        fm = QFontMetrics(path.font())
        path.setText(fm.elidedText(proj["path"], Qt.ElideMiddle, 168))
        path.setToolTip(proj["path"])
        meta = QLabel(f"{count} snapshot{'s' if count != 1 else ''}")
        meta.setStyleSheet(f"color:{ACCENT};font-size:11px;font-weight:700")
        lay.addWidget(name)
        lay.addWidget(path)
        lay.addWidget(meta)


class Sidebar(QWidget):
    project_selected = Signal(str)
    add_project = Signal()
    view_changed = Signal(str)

    def __init__(self, parent=None):
        super().__init__(parent)
        self.setObjectName("sidebar")
        self.setFixedWidth(258)
        self.view = "timeline"
        root = QVBoxLayout(self)
        root.setContentsMargins(16, 18, 16, 16)
        root.setSpacing(14)

        mark = QLabel("⏳")
        mark.setObjectName("logomark")
        word = QLabel("TimeCapsule")
        word.setObjectName("logo")
        title = hbox(mark, word, spacing=8)
        root.addLayout(title)
        tag = QLabel("PROJECT TIME MACHINE")
        tag.setObjectName("wordmark")
        root.addWidget(tag)

        root.addSpacing(6)
        head = hbox(spacing=8)
        cap = QLabel("PROJECTS")
        cap.setObjectName("tiny")
        head.addWidget(cap)
        head.addStretch(1)
        add = QPushButton("＋")
        add.setToolTip("Add a project folder")
        add.setCursor(Qt.PointingHandCursor)
        add.setFixedSize(26, 26)
        add.setStyleSheet(
            "QPushButton{background:#1a1d23;border:1px solid #262a33;border-radius:8px;"
            "font-size:14px;padding:0;color:#c8ccd4}"
            "QPushButton:hover{border-color:#ffb454;color:#ffb454}"
        )
        add.clicked.connect(self.add_project.emit)
        head.addWidget(add)
        root.addLayout(head)

        self.plist = QWidget()
        self.pcol = QVBoxLayout(self.plist)
        self.pcol.setContentsMargins(0, 0, 0, 0)
        self.pcol.setSpacing(8)
        pscroll = QScrollArea()
        pscroll.setWidgetResizable(True)
        pscroll.setWidget(self.plist)
        pscroll.setMaximumHeight(260)
        root.addWidget(pscroll)

        root.addWidget(hline())
        nav = QVBoxLayout()
        nav.setSpacing(4)
        self.navbtns = {}
        for key, icon, name in VIEWS:
            b = QPushButton(f"{icon}   {name}")
            b.setObjectName("nav")
            b.setCursor(Qt.PointingHandCursor)
            b.setCheckable(True)
            b.setProperty("active", "false")
            b.clicked.connect(lambda _=False, k=key: self.set_view(k, emit=True))
            self.navbtns[key] = b
            nav.addWidget(b)
        settings = QPushButton("⚙   Settings")
        settings.setObjectName("nav")
        settings.setCursor(Qt.PointingHandCursor)
        settings.setCheckable(True)
        settings.setProperty("active", "false")
        settings.clicked.connect(lambda _=False: self.set_view("settings", emit=True))
        self.navbtns["settings"] = settings
        nav.addWidget(settings)
        root.addLayout(nav)
        root.addStretch(1)

        self.autosave = QLabel("")
        self.autosave.setObjectName("sub")
        self.autosave.setWordWrap(True)
        root.addWidget(self.autosave)

        self.storage = QLabel("")
        self.storage.setObjectName("sub")
        self.storage.setWordWrap(True)
        root.addWidget(hline())
        root.addWidget(self.storage)
        ver = QLabel("TimeCapsule 0.0")
        ver.setObjectName("tiny")
        root.addWidget(ver)

    def set_view(self, key, emit=False):
        self.view = key
        for k, b in self.navbtns.items():
            active = k == key
            b.setProperty("active", "true" if active else "false")
            b.setChecked(active)
            b.style().unpolish(b)
            b.style().polish(b)
        if emit:
            self.view_changed.emit(key)

    def set_autosave(self, cfg, last=None):
        if not cfg.get("enabled"):
            self.autosave.setText("Auto-save off · set it up in Settings")
        else:
            text = f"Auto-save every {cfg['minutes']} min"
            if last:
                text += f" · last {store.fmt_time(last)}"
            self.autosave.setText(text)

    def reload(self, projects, current_pid, totals):
        while self.pcol.count():
            item = self.pcol.takeAt(0)
            w = item.widget()
            if w:
                w.deleteLater()
        if not projects:
            empty = QLabel("No projects yet.\nAdd a folder to start.")
            empty.setObjectName("sub")
            empty.setWordWrap(True)
            self.pcol.addWidget(empty)
        for p in projects:
            row = ProjectRow(p, totals.get(p["id"], 0), p["id"] == current_pid)
            row.clicked.connect(lambda pid=p["id"]: self.project_selected.emit(pid))
            self.pcol.addWidget(row)
        self.pcol.addStretch(1)
        self.storage.setText(totals.get("__storage__", ""))
