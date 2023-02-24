use i3ipc::reply::Node;
use i3ipc::I3Connection;

use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::str;
use std::thread;

use super::config::Config;
use super::display_icon::display_icon;
use super::i3_utils as i3;

pub struct State {
    pub curr_icon: Option<String>,
    pub prev_icon: Option<String>,
    pub curr_window: Option<i32>,
    pub curr_desktop: i32,
    pub dyn_x: u16,
}

impl State {
    pub fn update_icon(&mut self, icon_name: &str) {
        self.prev_icon = self.curr_icon.as_ref().map(|x| x.to_string());
        self.curr_icon = Some(icon_name.to_string());
    }

    pub fn reset_icons(&mut self) {
        self.prev_icon = None;
        self.curr_icon = None;
    }
}

pub struct Core {
    pub config: Config,
    pub state: State,
    pub connection: I3Connection,
}

impl Core {
    pub fn init() -> Self {
        let connection =
            I3Connection::connect().expect("Failed to connect to i3");
        let config = Config::load();

        let state = State {
            curr_icon: None,
            prev_icon: None,
            curr_window: None,
            curr_desktop: 1,
            dyn_x: config.x,
        };

        Self {
            config,
            connection,
            state,
        }
    }

    pub fn generate_icon(&self, window: i32) {
        let config = &self.config;

        if !Path::new(&config.cache_dir).is_dir() {
            fs::create_dir(&config.cache_dir)
                .expect("No cache folder was detected and couldn't create it");
        }

        let mut generate_icon_child =
            Command::new(format!("{}/generate-icon", &config.prefix))
                .arg(&config.cache_dir)
                .arg(config.size.to_string())
                .arg(&config.color)
                .arg(window.to_string())
                .stderr(Stdio::null())
                .spawn()
                .expect("Couldn't generate icon");

        generate_icon_child.wait().expect("Failed to wait on child");
    }

    pub fn update_dyn_x(&mut self, monitor: &str) {
        // -1 because of scratchpad desktop
        let desks_num =
            i3::get_desks_on_mon(&mut self.connection, monitor).len() - 1;
        let config = &self.config;
        let new_x = config.x + config.gap_per_desk * (desks_num as u16);

        self.state.dyn_x = new_x;
    }

    pub fn show_icon(&self, icon_path: String) {
        let config = &self.config;

        let (icon, dyn_x, y, size) = (
            icon_path,
            self.state.dyn_x.clone(),
            config.y.clone(),
            config.size.clone(),
        );

        thread::spawn(move || {
            display_icon(&icon, dyn_x, y, size);
        });
    }

    pub fn process_icon(&mut self, window: i32) {
        let icon_name = i3::get_icon_name(window);

        // If icon is the same, don't do anything
        if let Some(prev_icon) = &self.state.prev_icon {
            if &icon_name == prev_icon {
                return;
            }
        }

        let config = &self.config;
        let icon_path = format!("{}/{}.jpg", &config.cache_dir, icon_name);

        if !Path::new(&icon_path).exists() {
            self.generate_icon(window);
        }

        self.destroy_prev_icons();
        self.show_icon(icon_path);
    }

    pub fn print_info(&self, maybe_window: Option<i32>) {
        // Capitalizes first letter of the string, i.e. converts foo to Foo
        fn capitalize_first(s: &str) -> String {
            let mut c = s.chars();

            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().chain(c).collect(),
            }
        }

        // Don't add '\n' at the end, so that it will appear in front of icon
        // name, printed after it
        print!("{}", self.config.gap);
        io::stdout().flush().unwrap();

        match maybe_window {
            None => println!("Empty"),

            Some(window) => {
                let icon_name = &i3::get_icon_name(window);

                match icon_name.as_ref() {
                    "Brave-browser" => println!("Brave"),
                    "TelegramDesktop" => println!("Telegram"),
                    _ => println!("{}", capitalize_first(icon_name)),
                }
            }
        }
    }

    pub fn destroy_prev_icons(&mut self) {
        let icons_ids_raw = Command::new("xdo")
            .arg("id")
            .arg("-n")
            .arg("polybar-ixwindow-icon")
            .stderr(Stdio::null())
            .output()
            .expect("Couldn't detect any 'polybar-xwindow-icon' windows");

        let output = match String::from_utf8(icons_ids_raw.stdout) {
            Ok(v) => v,
            Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
        };

        let icons_ids = output.trim().split('\n');

        for id in icons_ids {
            let mut xdo_kill_child = Command::new("xdo")
                .arg("kill")
                .arg(id)
                .stderr(Stdio::null())
                .spawn()
                .expect("xdo couldn't kill icon window");

            xdo_kill_child.wait().expect("Failed to wait on child");
        }
    }

    pub fn process_focused_window(&mut self, window: i32) {
        if i3::is_window_fullscreen(window) {
            self.process_fullscreen_window();
            return;
        }

        let icon_name = i3::get_icon_name(window);

        self.print_info(Some(window));
        self.state.update_icon(&icon_name);
        self.process_icon(window);
    }

    // Come up with a better name
    pub fn process_fullscreen_window(&mut self) {
        self.destroy_prev_icons();

        // Reset icons, so that we can use process_focused_window
        // after. Otherwise it will not display icon, since app
        // name didn't change during fullscreen toggling
        self.state.reset_icons();
    }

    pub fn process_empty_desktop(&mut self) {
        self.destroy_prev_icons();
        self.state.reset_icons();
        self.print_info(None);
    }
}
