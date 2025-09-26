#!/usr/bin/env python

import random

from PySide6.QtGui import QIcon, QCursor, QAction
from PySide6.QtMultimedia import QMediaPlayer, QAudioOutput
from PySide6.QtWidgets import QApplication, QPushButton, QLabel, QStatusBar, QMessageBox
from PySide6.QtUiTools import QUiLoader
from PySide6.QtCore import Qt, QFile, QIODevice, QTimer, QUrl, QSettings

app = QApplication([])

assets = "/usr/share/meow-simulator/"

settings = QSettings("wzium", "MeowSimulator")

loader = QUiLoader()
ui_file = QFile(assets + "window.ui")
ui_file.open(QIODevice.ReadOnly)
window = loader.load(ui_file)
ui_file.close()

player = QMediaPlayer()
audio_output = QAudioOutput()
player.setAudioOutput(audio_output)

boykisser1 = QIcon()
boykisser1.addFile(assets + "static.png")
boykisser2 = QIcon()
boykisser2.addFile(assets + "static2.png")
app.setWindowIcon(boykisser1)

button = window.findChild(QPushButton, "meowButton")
button.setIcon(boykisser1)

menu_action = window.findChild(QAction, "actionAbout")

meows = int(settings.value("meows") or 0)
meows_text = "Meows: "
status_label = QLabel(meows_text + str(meows))
status_bar = window.findChild(QStatusBar, "statusbar")
status_bar.addWidget(status_label)

def restore_button_state():
    button.setIcon(boykisser1)
    button.setCursor(QCursor(Qt.OpenHandCursor))

def on_action_click():
    QMessageBox.information(window, 'UwU', "*purrs*", QMessageBox.Ok)

def on_button_click():
    global meows, settings
    meows += 1
    settings.setValue("meows", meows)
    meow_file = f"meow{random.randrange(1, 5)}.mp3"
    player.setSource(QUrl.fromLocalFile(assets + meow_file))
    player.play()
    button.setIcon(boykisser2)
    button.setCursor(QCursor(Qt.ClosedHandCursor))
    QTimer.singleShot(100, restore_button_state)
    status_label.setText(meows_text + str(meows))

button.clicked.connect(on_button_click)
menu_action.triggered.connect(on_action_click)

window.show()
app.exec()
