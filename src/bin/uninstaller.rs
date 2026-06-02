#![cfg_attr(windows, windows_subsystem = "windows")]

fn main() {
    #[cfg(windows)]
    windows::run();
}

#[cfg(windows)]
mod windows {
    use std::ffi::OsStr;
    use std::iter::once;
    use std::os::windows::ffi::OsStrExt;
    use std::os::windows::process::CommandExt;
    use std::path::{Path, PathBuf};
    use winreg::enums::*;
    use winreg::RegKey;

    const MB_OK: u32 = 0x00;
    const MB_YESNO: u32 = 0x04;
    const MB_ICONWARNING: u32 = 0x30;
    const MB_ICONINFORMATION: u32 = 0x40;
    const IDYES: i32 = 6;
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    pub const REG_KEY: &str =
        "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\MeowSimulator";

    #[link(name = "user32")]
    unsafe extern "system" {
        fn MessageBoxW(hwnd: usize, text: *const u16, caption: *const u16, utype: u32) -> i32;
    }

    fn wide(s: &str) -> Vec<u16> {
        OsStr::new(s).encode_wide().chain(once(0)).collect()
    }

    fn msgbox(text: &str, caption: &str, utype: u32) -> i32 {
        unsafe { MessageBoxW(0, wide(text).as_ptr(), wide(caption).as_ptr(), utype) }
    }

    fn install_location() -> PathBuf {
        let try_reg = || -> Option<PathBuf> {
            let key = RegKey::predef(HKEY_CURRENT_USER).open_subkey(REG_KEY).ok()?;
            let loc: String = key.get_value("InstallLocation").ok()?;
            Some(PathBuf::from(loc))
        };
        try_reg().unwrap_or_else(|| {
            std::env::current_exe().unwrap().parent().unwrap().to_path_buf()
        })
    }

    fn remove_shortcuts() {
        if let Some(d) = dirs::desktop_dir() {
            let _ = std::fs::remove_file(d.join("Meow Simulator.lnk"));
        }
        if let Some(data) = dirs::data_dir() {
            let _ = std::fs::remove_file(
                data.join("Microsoft\\Windows\\Start Menu\\Programs\\Meow Simulator.lnk"),
            );
        }
    }

    fn remove_registry() {
        let _ = RegKey::predef(HKEY_CURRENT_USER).delete_subkey_all(REG_KEY);
    }

    fn schedule_delete(install_dir: &Path) {
        let dir = install_dir.to_string_lossy().replace('"', "\"\"");
        let _ = std::process::Command::new("cmd")
            .args(["/c", &format!("timeout /t 2 >nul & rmdir /s /q \"{dir}\"")])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn();
    }

    pub fn run() {
        let install_dir = install_location();

        let confirmed = msgbox(
            &format!(
                "Are you sure you want to uninstall Meow Simulator?\n\nThis will remove all files from:\n{}",
                install_dir.display()
            ),
            "Uninstall Meow Simulator",
            MB_YESNO | MB_ICONWARNING,
        );

        if confirmed != IDYES {
            return;
        }

        remove_shortcuts();
        remove_registry();
        schedule_delete(&install_dir);

        msgbox(
            "Meow Simulator has been uninstalled successfully.",
            "Uninstall complete",
            MB_OK | MB_ICONINFORMATION,
        );
    }
}
