use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let out = env::var("OUT_DIR").unwrap();
    // OUT_DIR is target/<profile>/build/.../out — go up 3 levels to get target/<profile>/
    let target_dir = Path::new(&out).ancestors().nth(3).unwrap().to_path_buf();
    let dest = target_dir.join("assets");

    fs::create_dir_all(&dest).unwrap();

    for entry in fs::read_dir("assets").unwrap() {
        let entry = entry.unwrap();
        let dest_file = dest.join(entry.file_name());
        fs::copy(entry.path(), dest_file).unwrap();
    }

    println!("cargo:rerun-if-changed=assets");

    #[cfg(target_os = "windows")]
    {
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
