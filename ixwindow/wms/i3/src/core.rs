use i3ipc::reply::Node;
use i3ipc::I3Connection;

use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::str;
use std::thread;

use super::config::Config;
use super::display_icon::display_icon;
use super::i3_utils as i3;

#[derive(Clone)]
pub struct MonitorState {
    pub curr_icon: Option<String>,
    pub prev_icon: Option<String>,
    pub curr_window: Option<i32>,
    pub curr_desktop_id: Option<i32>,
    pub dyn_x: u16,
}

impl MonitorState {
    pub fn init(
        conn: &mut I3Connection,
        config: &Config,
        monitor_name: &str,
    ) -> Self {
        let curr_desktop_id = i3::get_focused_desktop_id(conn, monitor_name);
        let dyn_x = i3::calculate_dyn_x(conn, config, monitor_name);

        Self {
            curr_icon: None,
            prev_icon: None,
            curr_window: None,
            curr_desktop_id,
            dyn_x,
        }
    }

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
    pub monitors_states: HashMap<String, MonitorState>,
    pub connection: I3Connection,
    pub curr_mon_name: String,
}

impl Core {
    pub fn init() -> Self {
        let mut connection =
            I3Connection::connect().expect("Failed to connect to i3");
        let config = Config::load();
        let mut monitors_states: HashMap<String, MonitorState> = HashMap::new();
        let curr_mon_name = i3::get_focused_monitor(&mut connection);

        for monitor_name in config.monitors_names.clone() {
            let monitor_state =
                MonitorState::init(&mut connection, &config, &monitor_name);

            monitors_states.insert(monitor_name, monitor_state);
        }

        Self {
            config,
            monitors_states,
            connection,
            curr_mon_name,
        }
    }

    pub fn generate_icon(&self, window_id: i32) {
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
                .arg(window_id.to_string())
                .stderr(Stdio::null())
                .spawn()
                .expect("Couldn't generate icon");

        generate_icon_child.wait().expect("Failed to wait on child");
    }

    pub fn update_dyn_x(&mut self, monitor_name: &str) {
        let monitor_state = self
            .monitors_states
            .get_mut(monitor_name)
            .expect("Can't update dyn_x for non existing monitor");

        monitor_state.dyn_x = i3::calculate_dyn_x(
            &mut self.connection,
            &self.config,
            monitor_name,
        );
    }

    // Get rid off clone
    pub fn show_icon(&mut self, icon_path: String) {
        let config = &self.config;

        let (icon, dyn_x, y, size, curr_mon, mut curr_mon_state) = (
            icon_path,
            self.curr_mon_state().dyn_x,
            config.y,
            config.size,
            self.curr_mon_name.clone(),
            self.curr_mon_state_mut().clone(),
        );

        // let x: i32 = curr_mon_state;

        thread::spawn(move || {
            display_icon(&mut curr_mon_state, &icon, dyn_x, y, size, &curr_mon);
        });
    }

    pub fn process_icon(&mut self, window_id: i32) {
        let icon_name = i3::get_icon_name(window_id);
        // let mon_state = self.monitors_states.get(monitor_name).unwrap();

        if let Some(prev_icon) = &self.curr_mon_state().prev_icon {
            // If icon is the same, don't do anything
            if &icon_name == prev_icon {
                return;
            }
        }

        let config = &self.config;
        let icon_path = format!("{}/{}.jpg", &config.cache_dir, icon_name);

        if !Path::new(&icon_path).exists() {
            self.generate_icon(window_id);
        }

        self.destroy_prev_icons();
        self.show_icon(icon_path)
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

            Some(window_id) => {
                let icon_name = &i3::get_icon_name(window_id);

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

    pub fn process_focused_window(&mut self, window_id: i32) {
        if i3::is_window_fullscreen(window_id) {
            self.process_fullscreen_window();
            return;
        }

        let icon_name = i3::get_icon_name(window_id);

        self.print_info(Some(window_id));
        self.curr_mon_state_mut().update_icon(&icon_name);
        self.process_icon(window_id)
    }

    fn curr_mon_state(&self) -> &MonitorState {
        self.monitors_states.get(&self.curr_mon_name).unwrap()
    }

    fn curr_mon_state_mut(&mut self) -> &mut MonitorState {
        self.monitors_states.get_mut(&self.curr_mon_name).unwrap()
    }

    // Come up with a better name
    pub fn process_fullscreen_window(&mut self) {
        self.destroy_prev_icons();

        // Reset icons, so that we can use process_focused_window
        // after. Otherwise it will not display icon, since app
        // name didn't change during fullscreen toggling
        self.curr_mon_state_mut().reset_icons();
    }

    pub fn process_empty_desktop(&mut self) {
        self.destroy_prev_icons();
        self.curr_mon_state_mut().reset_icons();
        self.print_info(None);
    }
}
