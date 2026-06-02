#![cfg_attr(windows, windows_subsystem = "windows")]

static BUNDLE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/bundle.zip"));

fn main() {
    #[cfg(windows)]
    {
        use std::io::Cursor;

        let tmp = std::env::temp_dir().join(format!("meow-setup-{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();

        let mut archive = zip::ZipArchive::new(Cursor::new(BUNDLE)).unwrap();
        archive.extract(&tmp).unwrap();

        let installer = tmp.join("windows").join("installer.exe");
        let _ = std::process::Command::new(&installer)
            .current_dir(tmp.join("windows"))
            .status();

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
