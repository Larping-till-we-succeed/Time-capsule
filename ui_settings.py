"""Settings page: auto-save configuration as page content, not a dialog."""

from PySide6.QtCore import Signal
from PySide6.QtWidgets import (
    QCheckBox, QFrame, QHBoxLayout, QLabel, QPushButton, QVBoxLayout, QWidget,
)

import store
from theme import DIM, GREEN, MUTED, hbox, label
from widgets import SpinBox


class SettingsView(QWidget):
    save_requested = Signal()

    def __init__(self, parent=None):
        super().__init__(parent)
        root = QVBoxLayout(self)
        root.setContentsMargins(0, 0, 0, 0)
        root.setSpacing(16)

        card = QFrame()
        card.setObjectName("card")
        cl = QVBoxLayout(card)
        cl.setContentsMargins(20, 18, 20, 18)
        cl.setSpacing(12)

        cap = QLabel("AUTOMATIC SNAPSHOTS")
        cap.setObjectName("tiny")
        cl.addWidget(cap)
        title = QLabel("Auto-save")
        title.setStyleSheet("font-size:17px;font-weight:800")
        cl.addWidget(title)

        self.enabled = QCheckBox("Capture snapshots automatically")
        self.minutes = SpinBox()
        self.minutes.setRange(1, 1440)
        self.minutes.setSuffix(" min")
        self.minutes.setFixedWidth(120)
        self.enabled.toggled.connect(self.minutes.setEnabled)
        row = hbox(self.enabled, self.minutes, spacing=12)
        row.addStretch(1)
        cl.addLayout(row)

        self.only_changes = QCheckBox("Skip when the folder has not changed since the last snapshot")
        cl.addWidget(self.only_changes)

        hint = QLabel(
            "Automatic snapshots are marked “Auto-save” in the timeline and never replace the "
            "moments you capture by hand. They run for every project while TimeCapsule is open."
        )
        hint.setObjectName("sub")
        hint.setWordWrap(True)
        cl.addWidget(hint)

        foot = QHBoxLayout()
        foot.setSpacing(12)
        self.state = QLabel("")
        self.state.setObjectName("sub")
        self.state.setWordWrap(True)
        foot.addWidget(self.state, 1)
        self.saved = QLabel("")
        self.saved.setStyleSheet(f"color:{GREEN};font-size:12px;font-weight:700")
        foot.addWidget(self.saved)
        self.save = QPushButton("Save settings")
        self.save.setObjectName("primary")
        self.save.setMinimumHeight(38)
        self.save.clicked.connect(self.save_requested.emit)
        foot.addWidget(self.save)
        cl.addLayout(foot)

        root.addWidget(card)
        root.addStretch(1)
        self.load()

    def load(self, cfg=None, last=None):
        cfg = cfg or store.autosave_config()
        self.enabled.blockSignals(True)
        self.enabled.setChecked(cfg["enabled"])
        self.minutes.blockSignals(True)
        self.minutes.setValue(cfg["minutes"])
        self.minutes.blockSignals(False)
        self.enabled.blockSignals(False)
        self.minutes.setEnabled(cfg["enabled"])
        self.only_changes.setChecked(cfg["only_changes"])
        self.saved.setText("")
        self.set_state(cfg, last)

    def set_state(self, cfg, last=None):
        if not cfg.get("enabled"):
            self.state.setText("Auto-save is off — snapshots are only captured when you ask.")
        else:
            text = f"Auto-save is on · every {cfg['minutes']} min"
            if last:
                text += f" · last capture {store.fmt_time(last)}"
            self.state.setText(text)

    def mark_saved(self):
        self.saved.setText("Saved")

    def values(self):
        return {
            "auto_save": self.enabled.isChecked(),
            "auto_save_minutes": self.minutes.value(),
            "auto_save_only_changes": self.only_changes.isChecked(),
        }
