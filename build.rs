use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let out = env::var("OUT_DIR").unwrap();
    // OUT_DIR is target/<profile>/build/.../out — go up 3 levels to get target/<profile>/
    let target_dir = Path::new(&out).ancestors().nth(3).unwrap().to_path_buf();
    let dest = target_dir.join("assets");

    if dest.exists() { fs::remove_dir_all(&dest).unwrap(); }
    fs::create_dir_all(&dest).unwrap();

    for entry in fs::read_dir("assets").unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !matches!(ext, "ogg" | "png") { continue; }
        println!("cargo:rerun-if-changed={}", path.display());
        fs::copy(&path, dest.join(entry.file_name())).unwrap();
    }

    #[cfg(target_os = "windows")]
    {
        // bundle.zip for setup.rs's include_bytes! — real zip injected by Makefile via BUNDLE_ZIP
        println!("cargo:rerun-if-env-changed=BUNDLE_ZIP");
        let bundle_dest = Path::new(&out).join("bundle.zip");
        if let Ok(bundle_zip) = std::env::var("BUNDLE_ZIP") {
            // Re-compress the portable zip with zstd for a smaller setup.exe embed
            let src_data = fs::read(&bundle_zip).unwrap();
            let mut src = zip::ZipArchive::new(std::io::Cursor::new(src_data)).unwrap();
            let out_file = fs::File::create(&bundle_dest).unwrap();
            let mut writer = zip::ZipWriter::new(out_file);
            let options = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Zstd);
            for i in 0..src.len() {
                let mut entry = src.by_index(i).unwrap();
                let name = entry.name().to_string();
                let name = name.strip_prefix("windows/").unwrap_or(&name);
                if name.is_empty() { continue; }
                if !entry.is_dir() {
                    writer.start_file(name, options).unwrap();
                    std::io::copy(&mut entry, &mut writer).unwrap();
                }
            }
            writer.finish().unwrap();
        } else if !bundle_dest.exists() {
            // minimal valid empty ZIP placeholder for non-setup builds
            fs::write(&bundle_dest, b"PK\x05\x06\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00").unwrap();
        }

        let ico_path = Path::new(&out).join("app.ico");
        let img = image::open("assets/static.png").expect("failed to open icon");
        let resized = img.resize(256, 256, image::imageops::FilterType::Lanczos3);
        resized.save_with_format(&ico_path, image::ImageFormat::Ico)
            .expect("failed to write ico");

        let mut res = winresource::WindowsResource::new();
        res.set_icon(ico_path.to_str().unwrap());
        res.compile().expect("failed to compile windows resources");
    }
}
