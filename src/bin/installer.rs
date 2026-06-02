#![cfg_attr(windows, windows_subsystem = "windows")]

fn main() {
    #[cfg(windows)]
    windows::run();
}

#[cfg(windows)]
mod windows {
    use std::os::windows::process::CommandExt;
    use std::path::{Path, PathBuf};
    use winreg::enums::*;
    use winreg::RegKey;
    use gtk4::prelude::*;
    use gtk4::{
        ApplicationWindow, Box as GtkBox, Button, CheckButton, Entry, FileDialog,
        HeaderBar, Image, Label, Orientation, Separator, Stack, StackTransitionType,
        AlertDialog, gio, glib,
    };
    use libadwaita as adw;

    const APP_ID: &str = "com.wzium.MeowSimulatorInstaller";
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    fn setup_env(exe_dir: &Path) {
        unsafe {
            std::env::set_var(
                "GDK_PIXBUF_MODULEDIR",
                exe_dir.join("lib/gdk-pixbuf-2.0/2.10.0/loaders"),
            );
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

    fn create_shortcut(target: &Path, lnk: &Path) {
        let t = target.to_string_lossy().replace('\'', "''");
        let l = lnk.to_string_lossy().replace('\'', "''");
        let script = format!(
            "$s=(New-Object -ComObject WScript.Shell).CreateShortcut('{l}');$s.TargetPath='{t}';$s.Save()"
        );
        let _ = std::process::Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", &script])
            .creation_flags(CREATE_NO_WINDOW)
            .status();
    }

    fn write_registry(dest: &Path) {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        if let Ok((key, _)) = hkcu.create_subkey(
            "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\MeowSimulator",
        ) {
            let _ = key.set_value("DisplayName", &"Meow Simulator");
            let _ = key.set_value("Publisher", &"wzium");
            let _ = key.set_value("InstallLocation", &dest.to_string_lossy().as_ref());
            let _ = key.set_value(
                "DisplayIcon",
                &dest.join("MeowSimulatorRust.exe").to_string_lossy().as_ref(),
            );
            let _ = key.set_value(
                "UninstallString",
                &dest.join("uninstaller.exe").to_string_lossy().as_ref(),
            );
            let _ = key.set_value("NoModify", &1u32);
            let _ = key.set_value("NoRepair", &1u32);
        }
    }

    fn do_install(src: &Path, dest: &Path, desktop: bool, startmenu: bool) -> std::io::Result<()> {
        copy_dir(src, dest)?;
        write_registry(dest);
        let exe = dest.join("MeowSimulatorRust.exe");
        if desktop {
            if let Some(d) = dirs::desktop_dir() {
                create_shortcut(&exe, &d.join("Meow Simulator.lnk"));
            }
        }
        if startmenu {
            if let Some(data) = dirs::data_dir() {
                let programs = data.join("Microsoft\\Windows\\Start Menu\\Programs");
                let _ = std::fs::create_dir_all(&programs);
                create_shortcut(&exe, &programs.join("Meow Simulator.lnk"));
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
            .default_height(400)
            .resizable(false)
            .build();

        let header = HeaderBar::new();
        window.set_titlebar(Some(&header));

        let stack = Stack::new();
        stack.set_transition_duration(250);

        // ── Welcome ──────────────────────────────────────────────────────────
        let welcome = GtkBox::new(Orientation::Vertical, 0);
        welcome.set_margin_top(30);
        welcome.set_margin_bottom(20);
        welcome.set_margin_start(30);
        welcome.set_margin_end(30);

        let wc = GtkBox::new(Orientation::Vertical, 12);
        wc.set_vexpand(true);
        wc.set_valign(gtk4::Align::Center);
        wc.set_halign(gtk4::Align::Center);

        let app_icon = Image::from_file(src.join("assets").join("static.png"));
        app_icon.set_pixel_size(96);
        wc.append(&app_icon);

        let title = Label::new(Some("Meow Simulator"));
        title.add_css_class("title-1");
        wc.append(&title);

        let subtitle = Label::new(Some("Click. Meow. Repeat."));
        subtitle.add_css_class("dim-label");
        wc.append(&subtitle);

        welcome.append(&wc);

        let wb = GtkBox::new(Orientation::Horizontal, 0);
        let next_btn = Button::builder()
            .label("Next")
            .css_classes(["suggested-action"])
            .halign(gtk4::Align::End)
            .hexpand(true)
            .build();
        wb.append(&next_btn);
        welcome.append(&wb);

        stack.add_named(&welcome, Some("welcome"));

        // ── Options ──────────────────────────────────────────────────────────
        let options = GtkBox::new(Orientation::Vertical, 0);
        options.set_margin_top(20);
        options.set_margin_bottom(20);
        options.set_margin_start(30);
        options.set_margin_end(30);

        let oc = GtkBox::new(Orientation::Vertical, 12);
        oc.set_vexpand(true);
        oc.set_valign(gtk4::Align::Center);

        let path_label = Label::builder()
            .label("Install location:")
            .halign(gtk4::Align::Start)
            .build();
        oc.append(&path_label);

        let path_row = GtkBox::new(Orientation::Horizontal, 8);
        let entry = Entry::builder()
            .text(default_install_path().to_string_lossy().as_ref())
            .hexpand(true)
            .build();
        let browse_btn = Button::with_label("Browse...");
        path_row.append(&entry);
        path_row.append(&browse_btn);
        oc.append(&path_row);

        oc.append(&Separator::new(Orientation::Horizontal));

        let desktop_cb = CheckButton::builder()
            .label("Create desktop shortcut")
            .active(true)
            .build();
        let startmenu_cb = CheckButton::builder()
            .label("Create Start Menu shortcut")
            .active(true)
            .build();
        oc.append(&desktop_cb);
        oc.append(&startmenu_cb);

        options.append(&oc);

        let ob = GtkBox::new(Orientation::Horizontal, 8);
        ob.set_margin_top(8);
        let prev_btn = Button::with_label("Previous");
        let spacer = Label::new(None);
        spacer.set_hexpand(true);
        let install_btn = Button::builder()
            .label("Install")
            .css_classes(["suggested-action"])
            .build();
        ob.append(&prev_btn);
        ob.append(&spacer);
        ob.append(&install_btn);
        options.append(&ob);

        stack.add_named(&options, Some("options"));

        // ── Complete ──────────────────────────────────────────────────────────
        let complete = GtkBox::new(Orientation::Vertical, 0);
        complete.set_margin_top(30);
        complete.set_margin_bottom(30);
        complete.set_margin_start(30);
        complete.set_margin_end(30);

        let cc = GtkBox::new(Orientation::Vertical, 16);
        cc.set_vexpand(true);
        cc.set_valign(gtk4::Align::Center);
        cc.set_halign(gtk4::Align::Center);

        let check_icon = Image::from_icon_name("process-completed-symbolic");
        check_icon.set_pixel_size(96);
        cc.append(&check_icon);

        let done_label = Label::new(Some("Installation complete!"));
        done_label.add_css_class("title-1");
        cc.append(&done_label);

        complete.append(&cc);

        let cb = GtkBox::new(Orientation::Horizontal, 0);
        let cs1 = Label::new(None);
        cs1.set_hexpand(true);
        let finish_btn = Button::builder()
            .label("Finish")
            .css_classes(["suggested-action", "pill"])
            .width_request(120)
            .build();
        let cs2 = Label::new(None);
        cs2.set_hexpand(true);
        cb.append(&cs1);
        cb.append(&finish_btn);
        cb.append(&cs2);
        complete.append(&cb);

        stack.add_named(&complete, Some("complete"));

        window.set_child(Some(&stack));

        // ── Next: welcome → options ───────────────────────────────────────────
        {
            let stack = stack.clone();
            next_btn.connect_clicked(move |_| {
                stack.set_transition_type(StackTransitionType::SlideLeft);
                stack.set_visible_child_name("options");
            });
        }

        // ── Previous: options → welcome ───────────────────────────────────────
        {
            let stack = stack.clone();
            prev_btn.connect_clicked(move |_| {
                stack.set_transition_type(StackTransitionType::SlideRight);
                stack.set_visible_child_name("welcome");
            });
        }

        // ── Browse ────────────────────────────────────────────────────────────
        {
            let entry = entry.clone();
            let window_weak = window.downgrade();
            browse_btn.connect_clicked(move |_| {
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

        // ── Install ───────────────────────────────────────────────────────────
        {
            let stack = stack.clone();
            let entry = entry.clone();
            let desktop_cb = desktop_cb.clone();
            let startmenu_cb = startmenu_cb.clone();
            let prev_btn = prev_btn.clone();
            let window_weak = window.downgrade();
            let src = src.clone();

            install_btn.connect_clicked(move |btn| {
                let dest = PathBuf::from(entry.text().as_str());
                let desktop = desktop_cb.is_active();
                let startmenu = startmenu_cb.is_active();

                btn.set_label("Installing...");
                btn.set_sensitive(false);
                prev_btn.set_sensitive(false);

                let (tx, rx) = glib::MainContext::channel::<std::io::Result<()>>(glib::Priority::DEFAULT);
                let src = src.clone();

                std::thread::spawn(move || {
                    let _ = tx.send(do_install(&src, &dest, desktop, startmenu));
                });

                let stack = stack.clone();
                let btn = btn.clone();
                let prev_btn = prev_btn.clone();
                let window_weak = window_weak.clone();

                rx.attach(None, move |result| {
                    match result {
                        Ok(()) => {
                            stack.set_transition_type(StackTransitionType::SlideLeft);
                            stack.set_visible_child_name("complete");
                        }
                        Err(e) => {
                            btn.set_label("Install");
                            btn.set_sensitive(true);
                            prev_btn.set_sensitive(true);
                            if let Some(win) = window_weak.upgrade() {
                                AlertDialog::builder()
                                    .message("Installation failed")
                                    .detail(&e.to_string())
                                    .build()
                                    .show(Some(&win));
                            }
                        }
                    }
                    glib::ControlFlow::Break
                });
            });
        }

        // ── Finish ────────────────────────────────────────────────────────────
        {
            let window_weak = window.downgrade();
            finish_btn.connect_clicked(move |_| {
                let dest = PathBuf::from(entry.text().as_str());
                let _ = std::process::Command::new(dest.join("MeowSimulatorRust.exe"))
                    .current_dir(&dest)
                    .spawn();
                if let Some(win) = window_weak.upgrade() {
                    win.close();
                }
            });
        }

        window.present();
    }

    pub fn run() {
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
}
