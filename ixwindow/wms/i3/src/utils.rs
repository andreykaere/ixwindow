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
    pub fn update(&mut self, icon_name: &str) {
        self.prev_icon = self.curr_icon.as_ref().map(|x| x.to_string());
        self.curr_icon = Some(icon_name.to_string());
    }

    pub fn reset(&mut self) {
        self.prev_icon = None;
        self.curr_icon = None;
    }
}

impl Core {
    pub fn generate_icon(&self, icon_name: &str) {
        let config = &self.config;

        Command::new(format!("{}/generate_icon", config.prefix))
            .arg(&config.cache_dir)
            .arg(format!("{}", config.size))
            .arg(&config.color)
            .arg(icon_name);
    }

    pub fn display_icon(&mut self, icon_name: &str) {
        self.destroy_prev_icons();

        let config = &self.config;
        let icon_path =
            format!("{}/{}.jpg", format_filename(&config.cache_dir), icon_name);

        if !Path::new(&icon_path).exists() {
            println!("generate {icon_path}");
            self.generate_icon(icon_name);
        }

        println!("{}/polybar-xwindow-icon", format_filename(&config.prefix));
        println!("{icon_path}");

        Command::new(format!(
            "{}/polybar-xwindow-icon",
            format_filename(&config.prefix)
        ))
        .arg(&icon_path)
        .arg(format!("{}", config.x))
        .arg(format!("{}", config.y))
        .arg(format!("{}", config.size));
    }

    pub fn print_info(&mut self, window: Option<i32>) {
        print!("{}", self.config.gap);

        match window {
            None => print!("Empty"),
            Some(id) => {
                let icon_name = &get_icon_name(id);

                match icon_name.as_ref() {
                    "Brave-browser" => print!("Brave"),
                    "TelegramDesktop" => print!("Telegram"),
                    _ => print!("{}", capitalize_first(icon_name)),
                }
            }
        }

        io::stdout().flush().unwrap();
    }

    pub fn destroy_prev_icons(&mut self) {
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

    pub fn process_window(&mut self, id: i32) {
        let icon_name = get_icon_name(id);

        self.state.update(&icon_name);
        self.display_icon(&icon_name);
    }
    pub fn process_desktop(&mut self) {
        todo!();
    }

    pub fn exists_fullscreen_node(&mut self) {
        todo!();
    }

    pub fn get_current_desktop(&mut self) -> i32 {
        let connection = &mut self.connection;
        let desktops = connection
            .get_workspaces()
            .expect("Couldn't read information about desktops")
            .workspaces;

        for desktop in desktops {
            if desktop.focused {
                return desktop.num;
            }
        }

        panic!("Zero desktops!");
    }
}

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

pub fn get_icon_name(id: i32) -> String {
    get_wm_class(id)
}

pub fn format_filename(filename: &str) -> String {
    let home = std::env::var("HOME").unwrap();
    let filename = &shellexpand::env(filename).unwrap();
    let filename = shellexpand::tilde(filename).to_string();

    filename
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_filename_works() {
        let config = Config::init();

        assert_eq!(
            format_filename(&config.cache_dir),
            "/home/andrey/.config/polybar/scripts/ixwindow/polybar-icons"
        );
    }
}
