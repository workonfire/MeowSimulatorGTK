#!/usr/bin/env python

import os
import random
import sys

from PySide6.QtGui import QIcon, QCursor, QAction
from PySide6.QtMultimedia import QMediaPlayer, QAudioOutput
from PySide6.QtWidgets import QApplication, QPushButton, QLabel, QStatusBar, QMessageBox
from PySide6.QtUiTools import QUiLoader
from PySide6.QtCore import Qt, QFile, QIODevice, QTimer, QUrl, QSettings


def resolve_assets():
    system_assets = "/usr/share/meow-simulator"
    base = system_assets if os.path.isdir(system_assets) else os.path.join(os.path.dirname(__file__), "assets")
    return base + "/"


class MeowSimulator:
    MEOWS_TEXT = "Meows: "

    def __init__(self, assets):
        self.assets = assets
        self.settings = QSettings("wzium", "MeowSimulator")
        self._load_ui()
        self._setup_players()
        self._setup_icons()
        self._setup_widgets()

    def _load_ui(self):
        loader = QUiLoader()
        ui_file = QFile(self.assets + "window.ui")
        ui_file.open(QIODevice.ReadOnly)
        self.window = loader.load(ui_file)
        ui_file.close()

    def _setup_players(self):
        self._meow_audio = QAudioOutput()
        self.meow_player = QMediaPlayer()
        self.meow_player.setAudioOutput(self._meow_audio)

        self._purr_audio = QAudioOutput()
        self.purr_player = QMediaPlayer()
        self.purr_player.setAudioOutput(self._purr_audio)
        self.purr_player.setSource(QUrl.fromLocalFile(self.assets + "purr.mp3"))
        self.purr_player.setLoops(QMediaPlayer.Infinite)

    def _setup_icons(self):
        self.icon1 = QIcon()
        self.icon1.addFile(self.assets + "static.png")
        self.icon2 = QIcon()
        self.icon2.addFile(self.assets + "static2.png")
        QApplication.instance().setWindowIcon(self.icon1)

    def _setup_widgets(self):
        self.button = self.window.findChild(QPushButton, "meowButton")
        self.button.setIcon(self.icon1)
        self.button.clicked.connect(self._on_meow)

        self.meows = int(self.settings.value("meows") or 0)
        self.status_label = QLabel(self.MEOWS_TEXT + str(self.meows))
        self.window.findChild(QStatusBar, "statusbar").addWidget(self.status_label)

        self.window.findChild(QAction, "actionAbout").triggered.connect(self._on_purr)

    def _restore_button(self):
        self.button.setIcon(self.icon1)
        self.button.setCursor(QCursor(Qt.OpenHandCursor))

    def _on_meow(self):
        self.meows += 1
        self.settings.setValue("meows", self.meows)
        self.meow_player.setSource(QUrl.fromLocalFile(self.assets + f"meow{random.randrange(1, 5)}.mp3"))
        self.meow_player.play()
        self.button.setIcon(self.icon2)
        self.button.setCursor(QCursor(Qt.ClosedHandCursor))
        QTimer.singleShot(100, self._restore_button)
        self.status_label.setText(self.MEOWS_TEXT + str(self.meows))

    def _on_purr(self):
        self.purr_player.play()
        QMessageBox.information(self.window, 'UwU', "*purrs*", QMessageBox.Ok)
        self.purr_player.stop()

    def run(self):
        self.window.show()
        return QApplication.instance().exec()


if __name__ == "__main__":
    app = QApplication(sys.argv)
    sim = MeowSimulator(resolve_assets())
    sys.exit(sim.run())
