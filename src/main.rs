use std::cell::Cell;
use std::fs;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use cpal::traits::HostTrait;
use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box as GtkBox, Button, Image, Label, MenuButton,
    Orientation, HeaderBar, gio, glib,
};
use rand::Rng;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use serde::{Deserialize, Serialize};

const APP_ID: &str = "com.wzium.MeowSimulator";

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
    let system = Path::new("/usr/share/meow-simulator");
    if system.is_dir() {
        system.to_path_buf()
    } else {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("assets")
    }
}

fn play_sound(handle: &OutputStreamHandle, path: &Path) {
    if let Ok(file) = fs::File::open(path) {
        if let Ok(decoder) = Decoder::new(BufReader::new(file)) {
            if let Ok(sink) = Sink::try_new(handle) {
                sink.append(decoder);
                sink.detach();
            }
        }
    }
}

fn try_open_output_stream() -> Option<(OutputStream, OutputStreamHandle)> {
    if let Ok(result) = OutputStream::try_default() {
        return Some(result);
    }
    for host_id in cpal::available_hosts() {
        if let Ok(host) = cpal::host_from_id(host_id) {
            if let Ok(mut devices) = host.output_devices() {
                if let Some(device) = devices.next() {
                    if let Ok(result) = OutputStream::try_from_device(&device) {
                        return Some(result);
                    }
                }
            }
        }
    }
    None
}

fn build_ui(app: &Application) {
    let assets = resolve_assets();
    let config = load_config();
    let meows = Rc::new(Cell::new(config.meows));

    let (stream, stream_handle) = try_open_output_stream().expect("audio output");
    Box::leak(Box::new(stream)); // must outlive all sinks
    let stream_handle = Arc::new(stream_handle);

    // purr sink — kept alive for dialog duration
    let purr_sink: Arc<Mutex<Option<Sink>>> = Arc::new(Mutex::new(None));

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Meow Simulator")
        .default_width(300)
        .default_height(300)
        .build();

    // icons
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
    vbox.append(&button);
    vbox.append(&meow_label);

    // header bar with About menu
    let header = HeaderBar::new();
    let menu_button = MenuButton::new();
    menu_button.set_icon_name("open-menu-symbolic");

    let menu = gio::Menu::new();
    menu.append(Some("About / Purr"), Some("app.purr"));
    let popover = gtk4::PopoverMenu::from_model(Some(&menu));
    menu_button.set_popover(Some(&popover));
    header.pack_end(&menu_button);

    window.set_titlebar(Some(&header));
    window.set_child(Some(&vbox));

    // purr action
    {
        let assets = assets.clone();
        let stream_handle = Arc::clone(&stream_handle);
        let purr_sink = Arc::clone(&purr_sink);
        let window_weak = window.downgrade();
        let purr_action = gio::SimpleAction::new("purr", None);
        purr_action.connect_activate(move |_, _| {
            let path = assets.join("purr.mp3");
            if let Ok(file) = fs::File::open(&path) {
                if let Ok(decoder) = Decoder::new(BufReader::new(file)) {
                    if let Ok(sink) = Sink::try_new(&stream_handle) {
                        sink.append(decoder);
                        *purr_sink.lock().unwrap() = Some(sink);
                    }
                }
            }
            if let Some(win) = window_weak.upgrade() {
                let dialog = gtk4::AlertDialog::builder()
                    .message("UwU")
                    .detail("*purrs*")
                    .build();
                let purr_sink = Arc::clone(&purr_sink);
                dialog.choose(Some(&win), None::<&gio::Cancellable>, move |_| {
                    if let Some(sink) = purr_sink.lock().unwrap().take() {
                        sink.stop();
                    }
                });
            }
        });
        app.add_action(&purr_action);
    }

    // meow button
    {
        let assets = assets.clone();
        let stream_handle = Arc::clone(&stream_handle);
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
            play_sound(&stream_handle, &assets.join(format!("meow{n}.mp3")));

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

fn main() {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run();
}
