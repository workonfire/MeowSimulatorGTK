#![windows_subsystem = "windows"]
use std::cell::Cell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::fs;

use gtk4::prelude::*;
use gtk4::{
    ApplicationWindow, Box as GtkBox, Button, Image, Label, MenuButton,
    Orientation, HeaderBar, gio, glib,
};
#[cfg(target_os = "linux")]
use libadwaita as adw;
use rand::Rng;
use serde::{Deserialize, Serialize};
use gstreamer as gst;
use gst::prelude::*;

const APP_ID: &str = "com.wzium.MeowSimulator";

#[cfg(target_os = "linux")]
type AppType = adw::Application;
#[cfg(not(target_os = "linux"))]
type AppType = gtk4::Application;

#[derive(Serialize, Deserialize, Default)]
struct Config {
    meows: u64,
}

fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("meow-simulator")
        .join("config.toml")
}

fn load_config() -> Config {
    let path = config_path();
    fs::read_to_string(&path)
        .ok()
        .and_then(|s| toml::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_config(config: &Config) {
    let path = config_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(s) = toml::to_string(config) {
        let _ = fs::write(path, s);
    }
}

fn resolve_assets() -> PathBuf {
    #[cfg(target_os = "linux")]
    {
        let system = Path::new("/usr/share/meow-simulator");
        if system.is_dir() {
            return system.to_path_buf();
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let local = dir.join("assets");
            if local.is_dir() {
                return local;
            }
        }
    }
    panic!("could not locate assets directory");
}

fn play_sound(path: &Path) {
    let uri = format!("file://{}", path.display());
    match gst::ElementFactory::make("playbin")
        .property("uri", &uri)
        .build()
    {
        Ok(playbin) => {
            let _ = playbin.set_state(gst::State::Playing);
            let bus = playbin.bus().unwrap();
            glib::MainContext::default().spawn_local(async move {
                let mut stream = bus.stream();
                use futures_util::StreamExt;
                while let Some(msg) = stream.next().await {
                    use gst::MessageView;
                    match msg.view() {
                        MessageView::Eos(_) | MessageView::Error(_) => break,
                        _ => {}
                    }
                }
                let _ = playbin.set_state(gst::State::Null);
            });
        }
        Err(e) => eprintln!("play_sound: {e}"),
    }
}

fn play_purr(path: &Path) -> Option<gst::Element> {
    let uri = format!("file://{}", path.display());
    let playbin = gst::ElementFactory::make("playbin")
        .property("uri", &uri)
        .build()
        .ok()?;
    let playbin_clone = playbin.clone();
    playbin.connect("about-to-finish", false, move |_| {
        playbin_clone.seek_simple(
            gst::SeekFlags::FLUSH | gst::SeekFlags::KEY_UNIT,
            gst::ClockTime::ZERO,
        ).ok();
        None
    });
    let _ = playbin.set_state(gst::State::Playing);
    Some(playbin)
}

fn stop_purr(playbin: &gst::Element) {
    let _ = playbin.set_state(gst::State::Null);
}

fn build_ui(app: &AppType) {
    let assets = resolve_assets();
    let config = load_config();
    let meows = Rc::new(Cell::new(config.meows));

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Meow Simulator")
        .default_width(300)
        .default_height(300)
        .build();

    let icon1 = assets.join("static.png");
    let icon2 = assets.join("static2.png");

    let image = Image::from_file(&icon1);
    image.set_pixel_size(180);

    let button = Button::builder()
        .child(&image)
        .css_classes(["flat"])
        .build();

    let meow_label = Label::new(Some(&format!("Meows: {}", meows.get())));

    let vbox = GtkBox::new(Orientation::Vertical, 12);
    vbox.set_margin_top(20);
    vbox.set_margin_bottom(20);
    vbox.set_margin_start(20);
    vbox.set_margin_end(20);
    vbox.set_halign(gtk4::Align::Center);
    vbox.set_valign(gtk4::Align::Center);
    vbox.append(&button);
    vbox.append(&meow_label);

    let header = HeaderBar::new();
    let menu_button = MenuButton::new();
    menu_button.set_icon_name("open-menu-symbolic");

    let menu = gio::Menu::new();
    menu.append(Some("Purr"), Some("app.purr"));
    let popover = gtk4::PopoverMenu::from_model(Some(&menu));
    menu_button.set_popover(Some(&popover));
    header.pack_start(&menu_button);

    window.set_titlebar(Some(&header));
    window.set_child(Some(&vbox));

    // purr action
    {
        let assets = assets.clone();
        let window_weak = window.downgrade();
        let purr_action = gio::SimpleAction::new("purr", None);

        purr_action.connect_activate(move |_, _| {
            let purr = play_purr(&assets.join("purr.mp3"));

            if let Some(win) = window_weak.upgrade() {
                let dialog = gtk4::AlertDialog::builder()
                    .message("UwU")
                    .detail("*purrs*")
                    .build();
                dialog.choose(Some(&win), None::<&gio::Cancellable>, move |_| {
                    if let Some(p) = purr {
                        stop_purr(&p);
                    }
                });
            } else if let Some(p) = purr {
                stop_purr(&p);
            }
        });
        app.add_action(&purr_action);
    }

    // meow button
    {
        let assets = assets.clone();
        let image = image.clone();
        let icon1 = icon1.clone();
        let icon2 = icon2.clone();
        let meows = Rc::clone(&meows);
        let meow_label = meow_label.clone();

        button.connect_clicked(move |_| {
            let count = meows.get() + 1;
            meows.set(count);
            save_config(&Config { meows: count });
            meow_label.set_text(&format!("Meows: {count}"));

            let n = rand::thread_rng().gen_range(1..=4);
            play_sound(&assets.join(format!("meow{n}.mp3")));

            image.set_from_file(Some(&icon2));
            let image = image.clone();
            let icon1 = icon1.clone();
            glib::timeout_add_local_once(std::time::Duration::from_millis(100), move || {
                image.set_from_file(Some(&icon1));
            });
        });
    }

    window.present();
}

#[cfg(target_os = "windows")]
fn setup_windows_env(exe_dir: &Path) {
    // safe: called before gst::init() and GTK init, still single-threaded
    unsafe {
        std::env::set_var("GST_PLUGIN_PATH", exe_dir.join("lib/gstreamer-1.0"));
        std::env::set_var("GST_PLUGIN_SCANNER", exe_dir.join("gst-plugin-scanner.exe"));
    }

    let loaders_dir = exe_dir.join("lib/gdk-pixbuf-2.0/2.10.0/loaders");
    let cache = exe_dir.join("lib/gdk-pixbuf-2.0/2.10.0/loaders.cache");
    let query_tool = exe_dir.join("gdk-pixbuf-query-loaders.exe");

    if query_tool.exists() {
        if let Ok(entries) = fs::read_dir(&loaders_dir) {
            let dlls: Vec<_> = entries
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.extension().map_or(false, |e| e == "dll"))
                .collect();
            if let Ok(out) = std::process::Command::new(&query_tool).args(&dlls).output() {
                let _ = fs::write(&cache, &out.stdout);
            }
        }
    }

    unsafe { std::env::set_var("GDK_PIXBUF_MODULE_FILE", &cache); }
}

fn main() {
    #[cfg(target_os = "windows")]
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            setup_windows_env(dir);
        }
    }

    gst::init().expect("GStreamer init failed");

    glib::set_application_name("Meow Simulator");
    #[cfg(target_os = "linux")]
    let app = adw::Application::builder().application_id(APP_ID).build();
    #[cfg(not(target_os = "linux"))]
    let app = gtk4::Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run();
}
