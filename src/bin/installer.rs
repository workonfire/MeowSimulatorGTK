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

    const REG_KEY: &str =
        "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\MeowSimulator";

    fn existing_install() -> Option<PathBuf> {
        let key = RegKey::predef(HKEY_CURRENT_USER).open_subkey(REG_KEY).ok()?;
        let loc: String = key.get_value("InstallLocation").ok()?;
        Some(PathBuf::from(loc))
    }

    fn write_registry(dest: &Path) {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        if let Ok((key, _)) = hkcu.create_subkey(REG_KEY) {
            let _ = key.set_value("DisplayName", &"Meow Simulator");
            let _ = key.set_value("Publisher", &"wzium");
            let _ = key.set_value("InstallLocation", &dest.to_string_lossy().as_ref());
            let _ = key.set_value(
                "DisplayIcon",
                &dest.join("MeowSimulatorRust.exe").to_string_lossy().as_ref(),
            );
            let _ = key.set_value(
                "UninstallString",
                &dest.join("installer.exe").to_string_lossy().as_ref(),
            );
            let _ = key.set_value("NoModify", &1u32);
            let _ = key.set_value("NoRepair", &1u32);
        }
    }

    fn tracked_paths(install_dir: &Path) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        fn scan_dir(dir: &Path, out: &mut Vec<PathBuf>) {
            let Ok(entries) = std::fs::read_dir(dir) else { return };
            for e in entries.flatten() {
                let p = e.path();
                if p.is_dir() { scan_dir(&p, out); }
                else { out.push(p); }
            }
        }
        if install_dir.exists() { scan_dir(install_dir, &mut paths); }

        if let Some(d) = dirs::desktop_dir() {
            let p = d.join("Meow Simulator.lnk");
            if p.exists() { paths.push(p); }
        }
        if let Some(d) = dirs::data_dir() {
            let p = d.join("Microsoft\\Windows\\Start Menu\\Programs\\Meow Simulator.lnk");
            if p.exists() { paths.push(p); }
        }
        paths
    }

    fn locked_by_other_process(install_dir: &Path) -> bool {
        use std::os::windows::ffi::OsStrExt;
        use windows::Win32::System::RestartManager::*;
        use windows::core::{PCWSTR, PWSTR};

        if !install_dir.exists() { return false; }

        let all_files: Vec<Vec<u16>> = tracked_paths(install_dir)
            .iter()
            .map(|p| {
                let mut w: Vec<u16> = p.as_os_str().encode_wide().collect();
                w.push(0);
                w
            })
            .collect();
        if all_files.is_empty() { return false; }

        let own_pid = std::process::id();

        (|| -> windows::core::Result<bool> {
            let mut session = 0u32;
            let mut key = [0u16; 33];
            unsafe { RmStartSession(&mut session, Some(0), PWSTR(key.as_mut_ptr())).ok()? };

            struct RmSession(u32);
            impl Drop for RmSession {
                fn drop(&mut self) { unsafe { let _ = RmEndSession(self.0); } }
            }
            let _guard = RmSession(session);

            let ptrs: Vec<PCWSTR> = all_files.iter().map(|f| PCWSTR(f.as_ptr())).collect();
            unsafe { RmRegisterResources(session, Some(&ptrs), None, None).ok()? };

            let mut needed = 0u32;
            let mut actual = 0u32;
            let mut reboot = 0u32;
            let _ = unsafe { RmGetList(session, &mut needed, &mut actual, None, &mut reboot) };
            if needed == 0 { return Ok(false); }

            let mut infos = vec![RM_PROCESS_INFO::default(); needed as usize];
            actual = needed;
            unsafe { RmGetList(session, &mut needed, &mut actual, Some(infos.as_mut_ptr()), &mut reboot).ok()? };

            Ok(infos[..actual as usize].iter().any(|p| p.Process.dwProcessId != own_pid))
        })().unwrap_or(false)
    }

    fn temp_installer_dir() -> PathBuf {
        std::env::temp_dir().join("MeowSimulator-Installer")
    }


    fn show_native_error(msg: &str) {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};
        use windows::core::PCWSTR;
        let title: Vec<u16> = OsStr::new("Meow Simulator").encode_wide().chain(std::iter::once(0)).collect();
        let text: Vec<u16> = OsStr::new(msg).encode_wide().chain(std::iter::once(0)).collect();
        unsafe { let _ = MessageBoxW(None, PCWSTR(text.as_ptr()), PCWSTR(title.as_ptr()), MB_OK | MB_ICONERROR); }
    }

    fn do_uninstall(install_dir: &Path) -> std::io::Result<()> {
        if let Some(d) = dirs::desktop_dir() {
            let _ = std::fs::remove_file(d.join("Meow Simulator.lnk"));
        }
        if let Some(data) = dirs::data_dir() {
            let _ = std::fs::remove_file(
                data.join("Microsoft\\Windows\\Start Menu\\Programs\\Meow Simulator.lnk"),
            );
        }
        let _ = RegKey::predef(HKEY_CURRENT_USER).delete_subkey_all(REG_KEY);
        std::fs::remove_dir_all(install_dir)
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
        let src = std::env::current_exe().unwrap().parent().unwrap().to_path_buf();

        let existing = existing_install();

        let window = ApplicationWindow::builder()
            .application(app)
            .title("Meow Simulator Installer")
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

        let check_icon = Image::from_file(src.join("assets").join("static2.png"));
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

        // ── Uninstall page ────────────────────────────────────────────────────
        let uninstall_page = GtkBox::new(Orientation::Vertical, 0);
        uninstall_page.set_margin_top(30);
        uninstall_page.set_margin_bottom(30);
        uninstall_page.set_margin_start(30);
        uninstall_page.set_margin_end(30);

        let uc = GtkBox::new(Orientation::Vertical, 12);
        uc.set_vexpand(true);
        uc.set_valign(gtk4::Align::Center);
        uc.set_halign(gtk4::Align::Center);

        let already_label = Label::new(Some("Meow Simulator is already installed"));
        already_label.add_css_class("title-2");
        uc.append(&already_label);

        let install_path_label = Label::new(
            existing.as_deref().and_then(|p| p.to_str()),
        );
        install_path_label.add_css_class("dim-label");
        uc.append(&install_path_label);

        uninstall_page.append(&uc);

        let ub = GtkBox::new(Orientation::Horizontal, 0);
        let us1 = Label::new(None);
        us1.set_hexpand(true);
        let uninstall_btn = Button::builder()
            .label("Uninstall")
            .css_classes(["destructive-action", "pill"])
            .width_request(120)
            .build();
        let us2 = Label::new(None);
        us2.set_hexpand(true);
        ub.append(&us1);
        ub.append(&uninstall_btn);
        ub.append(&us2);
        uninstall_page.append(&ub);

        stack.add_named(&uninstall_page, Some("uninstall"));

        window.set_child(Some(&stack));

        if existing.is_some() {
            stack.set_visible_child_name("uninstall");
        }

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

                let result: std::sync::Arc<std::sync::Mutex<Option<std::io::Result<()>>>> =
                    std::sync::Arc::new(std::sync::Mutex::new(None));
                let result_thread = std::sync::Arc::clone(&result);
                let src = src.clone();

                std::thread::spawn(move || {
                    *result_thread.lock().unwrap() = Some(do_install(&src, &dest, desktop, startmenu));
                });

                let stack = stack.clone();
                let btn = btn.clone();
                let prev_btn = prev_btn.clone();
                let window_weak = window_weak.clone();

                glib::idle_add_local(move || {
                    if let Some(res) = result.lock().unwrap().take() {
                        match res {
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
                        return glib::ControlFlow::Break;
                    }
                    glib::ControlFlow::Continue
                });
            });
        }

        // ── Uninstall ─────────────────────────────────────────────────────────
        if let Some(install_path) = existing {
            let window_weak = window.downgrade();
            uninstall_btn.connect_clicked(move |btn| {
                if locked_by_other_process(&install_path) {
                    if let Some(win) = window_weak.upgrade() {
                        AlertDialog::builder()
                            .message("Meow Simulator is still running")
                            .detail("Please close Meow Simulator before uninstalling.")
                            .build()
                            .show(Some(&win));
                    }
                    return;
                }
                btn.set_label("Uninstalling...");
                btn.set_sensitive(false);

                let result: std::sync::Arc<std::sync::Mutex<Option<std::io::Result<()>>>> =
                    std::sync::Arc::new(std::sync::Mutex::new(None));
                let result_thread = std::sync::Arc::clone(&result);
                let path = install_path.clone();
                std::thread::spawn(move || {
                    *result_thread.lock().unwrap() = Some(do_uninstall(&path));
                });

                let btn = btn.clone();
                let window_weak = window_weak.clone();
                glib::idle_add_local(move || {
                    let mut guard = result.lock().unwrap();
                    if let Some(res) = guard.take() {
                        drop(guard);
                        match res {
                            Ok(()) => {
                                // Schedule temp dir self-cleanup after this process exits
                                if let Ok(exe) = std::env::current_exe() {
                                    if exe.starts_with(std::env::temp_dir()) {
                                        if let Some(tmp) = exe.parent() {
                                            let pid = std::process::id();
                                            let t = tmp.to_string_lossy().replace('\'', "''");
                                            let script = format!(
                                                "Wait-Process -Id {pid} -ErrorAction SilentlyContinue; \
                                                 Remove-Item -Recurse -Force '{t}'"
                                            );
                                            let _ = std::process::Command::new("powershell")
                                                .args(["-NoProfile", "-NonInteractive", "-Command", &script])
                                                .creation_flags(CREATE_NO_WINDOW)
                                                .spawn();
                                        }
                                    }
                                }
                                if let Some(win) = window_weak.upgrade() {
                                    let win_weak = win.downgrade();
                                    AlertDialog::builder()
                                        .message("Meow Simulator has been uninstalled.")
                                        .build()
                                        .choose(Some(&win), None::<&gio::Cancellable>, move |_| {
                                            if let Some(w) = win_weak.upgrade() { w.close(); }
                                        });
                                }
                            }
                            Err(e) => {
                                btn.set_label("Uninstall");
                                btn.set_sensitive(true);
                                if let Some(win) = window_weak.upgrade() {
                                    AlertDialog::builder()
                                        .message("Uninstallation failed")
                                        .detail(&e.to_string())
                                        .build()
                                        .show(Some(&win));
                                }
                            }
                        }
                        return glib::ControlFlow::Break;
                    }
                    glib::ControlFlow::Continue
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
        let self_exe = std::env::current_exe().unwrap_or_default();
        let in_temp = self_exe.starts_with(std::env::temp_dir());

        if !in_temp {
            if let Some(install_dir) = existing_install() {
                if self_exe.starts_with(&install_dir) {
                    // Running from the install dir — copy to temp and relaunch from there
                    // so the install dir has no DLL locks when we delete it.
                    let tmp = temp_installer_dir();
                    let tmp_exe = tmp.join("installer.exe");
                    if tmp.exists() {
                        if locked_by_other_process(&tmp) {
                            show_native_error("Meow Simulator installer is already running.");
                            return;
                        }
                        let _ = std::fs::remove_dir_all(&tmp);
                    }
                    if copy_dir(&install_dir, &tmp).is_ok() {
                        let _ = std::process::Command::new(&tmp_exe)
                            .env("MEOW_INSTALL_DIR", &install_dir)
                            .creation_flags(CREATE_NO_WINDOW)
                            .spawn();
                    }
                    return;
                }
            }
        }

        setup_env(&self_exe.parent().map(|p| p.to_path_buf()).unwrap_or_default());
        glib::set_application_name("Meow Simulator Installer");
        let app = adw::Application::builder().application_id(APP_ID).build();
        app.connect_activate(build_ui);
        app.run();
    }
}
