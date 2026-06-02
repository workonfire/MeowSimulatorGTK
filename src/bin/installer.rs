#![cfg_attr(windows, windows_subsystem = "windows")]

use std::path::{Path, PathBuf};
use gtk4::prelude::*;
use gtk4::{
    ApplicationWindow, Box as GtkBox, Button, Entry, Label,
    Orientation, HeaderBar, FileDialog, gio, glib,
};
use libadwaita as adw;

const APP_ID: &str = "com.wzium.MeowSimulatorInstaller";

#[cfg(windows)]
fn setup_env(exe_dir: &Path) {
    unsafe {
        std::env::set_var("GDK_PIXBUF_MODULEDIR", exe_dir.join("lib/gdk-pixbuf-2.0/2.10.0/loaders"));
    }
}

fn default_install_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("MeowSimulator")
}

fn copy_dir(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let dst_path = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &dst_path)?;
        } else {
            std::fs::copy(entry.path(), dst_path)?;
        }
    }
    Ok(())
}

fn build_ui(app: &adw::Application) {
    let src = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Meow Simulator — Installer")
        .default_width(520)
        .default_height(160)
        .resizable(false)
        .build();

    let header = HeaderBar::new();
    window.set_titlebar(Some(&header));

    let vbox = GtkBox::new(Orientation::Vertical, 12);
    vbox.set_margin_top(20);
    vbox.set_margin_bottom(20);
    vbox.set_margin_start(20);
    vbox.set_margin_end(20);

    let path_row = GtkBox::new(Orientation::Horizontal, 8);
    let entry = Entry::builder()
        .text(default_install_path().to_string_lossy().as_ref())
        .hexpand(true)
        .build();
    let browse = Button::with_label("Browse...");
    path_row.append(&entry);
    path_row.append(&browse);

    let install_btn = Button::builder()
        .label("Install")
        .css_classes(["suggested-action"])
        .build();

    let status = Label::new(None);

    vbox.append(&path_row);
    vbox.append(&install_btn);
    vbox.append(&status);
    window.set_child(Some(&vbox));

    // Browse
    {
        let entry = entry.clone();
        let window_weak = window.downgrade();
        browse.connect_clicked(move |_| {
            let Some(win) = window_weak.upgrade() else { return };
            let dialog = FileDialog::builder().title("Choose install folder").build();
            let entry = entry.clone();
            dialog.select_folder(Some(&win), None::<&gio::Cancellable>, move |result| {
                if let Ok(file) = result {
                    if let Some(path) = file.path() {
                        entry.set_text(&path.to_string_lossy());
                    }
                }
            });
        });
    }

    // Install
    {
        let install_btn = install_btn.clone();
        let status = status.clone();
        install_btn.connect_clicked(move |btn| {
            let dest = PathBuf::from(entry.text().as_str());
            btn.set_sensitive(false);
            status.set_text("Installing...");
            match copy_dir(&src, &dest) {
                Ok(()) => status.set_text("Installed successfully!"),
                Err(e) => {
                    status.set_text(&format!("Error: {e}"));
                    btn.set_sensitive(true);
                }
            }
        });
    }

    window.present();
}

fn main() {
    #[cfg(windows)]
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            setup_env(dir);
        }
    }

    glib::set_application_name("Meow Simulator Installer");
    let app = adw::Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run();
}
