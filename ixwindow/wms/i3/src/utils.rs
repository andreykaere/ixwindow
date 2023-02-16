use i3ipc::reply::Node;

use std::io::{self, Write};
use std::path::Path;
use std::process::Command;
use std::str;

use super::Config;
use super::Core;

pub struct State {
    pub curr_icon: Option<String>,
    pub prev_icon: Option<String>,
    pub curr_window: Option<i32>,
    pub curr_desktop: i32,
}

impl Default for State {
    fn default() -> Self {
        Self {
            curr_icon: None,
            prev_icon: None,
            curr_window: None,
            curr_desktop: 1,
        }
    }
}

impl State {
    fn update(&mut self, icon_name: String) {
        self.prev_icon = self.curr_icon.as_ref().map(|x| x.to_string());
        self.curr_icon = Some(icon_name);
    }

    fn reset(&mut self) {
        self.prev_icon = None;
        self.curr_icon = None;
    }
}

pub fn generate_icon(config: &Config, name: &str) {
    Command::new(format!("{}/generate_icon", config.prefix))
        .arg(&config.cache_dir)
        .arg(format!("{}", config.size))
        .arg(&config.color)
        .arg(name);
}

pub fn display_icon(config: &Config, name: &str) {
    destroy_prev_icons(config);

    let icon_name = format!("{}/{}.jpg", config.cache_dir, name);

    if Path::new(&icon_name).exists() {
        Command::new(format!("{}/polybar-xwindow-icon", config.prefix))
            .arg(&icon_name)
            .arg(format!("{}", config.x))
            .arg(format!("{}", config.y))
            .arg(format!("{}", config.size));
    }
}

pub fn print_info(gap: &str, window: Option<&Node>) {
    print!("{gap}");

    match window {
        None => print!("Empty"),
        Some(node) => {
            let id = node.window.expect("Couldn't get window id");
            let wm_class = &get_wm_class(id);

            match wm_class.as_ref() {
                "Brave-browser" => print!("Brave"),
                "TelegramDesktop" => print!("Telegram"),
                _ => print!("{}", capitalize_first(wm_class)),
            }
        }
    }

    io::stdout().flush().unwrap();
}

pub fn destroy_prev_icons(config: &Config) {
    let icons_ids_raw = Command::new("xdo")
        .arg("id")
        .arg("-n")
        .arg("polybar-xwindow-icon")
        .output()
        .expect("Couldn't detect any 'polybar-xwindow-icon' windows");

    let output = match String::from_utf8(icons_ids_raw.stdout) {
        Ok(v) => v,
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };

    let icons_ids = output.split(' ');

    for id in icons_ids {
        Command::new("xdo").arg("kill").arg(id);
    }
}

pub fn process_window(core: &Core) {}
pub fn process_desktop(core: &Core) {}

pub fn exists_fullscreen_node() {}

pub fn capitalize_first(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().chain(c).collect(),
    }
}

pub fn get_wm_class(id: i32) -> String {
    let wm_class = Command::new("xprop")
        .arg("-id")
        .arg(id.to_string())
        .arg("WM_CLASS")
        .output()
        .expect("Failed to get WM_CLASS of the window");

    let stdout = match String::from_utf8(wm_class.stdout) {
        Ok(v) => v,
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };

    stdout
        .split(' ')
        .last()
        .expect("WM_CLASS is empty")
        .to_string()
        .trim()
        .replace('"', "")
}

fn get_current_desktop(core: &Core) -> i32 {
    todo!();
}
