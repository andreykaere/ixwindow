use i3ipc::I3Connection;

use std::fs;
use std::io::{self, Write};
use std::mem;
use std::path::Path;
use std::process::{Command, Stdio};
use std::str;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use super::config::{Config, I3Config};
use super::i3_utils as i3;
use super::x11_utils;

#[derive(Debug)]
pub struct State {
    pub curr_icon: Option<String>,
    pub prev_icon: Option<String>,
    pub dyn_x: i16,
}

impl State {
    pub fn init(
        conn: &mut I3Connection,
        config: &Config,
        monitor_name: &str,
    ) -> Self {
        if let Config::I3Config(config) = config {
            let dyn_x = i3::calculate_dyn_x(conn, config, monitor_name);

            return Self {
                curr_icon: None,
                prev_icon: None,
                dyn_x,
            };
        }

        unreachable!();
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

#[derive(Debug)]
pub struct Monitor {
    pub state: State,
    pub name: Arc<String>,
    icons_threads: Vec<thread::JoinHandle<()>>,
    destroy_icons_flag: Arc<AtomicBool>,
}

impl Monitor {
    pub fn init(
        conn: &mut I3Connection,
        config: &Config,
        monitor_name: Option<String>,
    ) -> Self {
        let name = match monitor_name {
            Some(x) => x,
            None => x11_utils::get_primary_monitor_name()
                .expect("Couldn't get name of primary monitor"),
        };

        let name = Arc::new(name);
        let state = State::init(conn, config, &name);
        let icons_threads = Vec::new();
        let destroy_icons_flag = Arc::new(AtomicBool::new(false));

        Self {
            name,
            state,
            icons_threads,
            destroy_icons_flag,
        }
    }
}

pub struct Core {
    pub config: I3Config,
    pub connection: I3Connection,
    pub monitor: Monitor,
}

impl Core {
    pub fn init(monitor_name: Option<String>) -> Self {
        let mut connection =
            I3Connection::connect().expect("Failed to connect to i3");
        let config = Config::load_i3();
        let monitor = Monitor::init(&mut connection, &config, monitor_name);

        if let Config::I3Config(config) = config {
            return Self {
                config,
                connection,
                monitor,
            };
        }
        unreachable!();
    }

    pub fn process_start(&mut self) {
        if let Some(window_id) = self.get_focused_window_id() {
            self.process_focused_window(window_id);
        } else {
            self.process_empty_desktop();
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

    pub fn update_dyn_x(&mut self) {
        self.monitor.state.dyn_x = i3::calculate_dyn_x(
            &mut self.connection,
            &self.config,
            &self.monitor.name,
        );
    }

    pub fn show_icon(&mut self, icon_path: Arc<String>) {
        let config = &self.config;

        let (dyn_x, y, size, monitor_name, flag) = (
            self.monitor.state.dyn_x,
            config.y,
            config.size,
            Arc::clone(&self.monitor.name),
            Arc::clone(&self.monitor.destroy_icons_flag),
        );

        let icon_thread = thread::spawn(move || {
            let result = x11_utils::display_icon(
                icon_path,
                dyn_x,
                y,
                size,
                monitor_name,
                flag,
            );

            // We don't want to throw a error if displaying icon fails for
            // some reason
            result.unwrap_or(());
        });

        self.monitor.icons_threads.push(icon_thread);
    }

    pub fn process_icon(&mut self, window_id: i32) {
        let state = &self.monitor.state;

        if state.prev_icon == state.curr_icon {
            return;
        }

        // curr_icon is not `None`, because we put the current icon name before
        // calling `process_icon`
        let icon_name = state.curr_icon.as_ref().unwrap();

        let config = &self.config;
        let icon_path = format!("{}/{}.jpg", &config.cache_dir, icon_name);

        if !Path::new(&icon_path).exists() {
            self.generate_icon(window_id);
        }

        self.destroy_prev_icons();
        self.show_icon(Arc::new(icon_path));
    }

    pub fn print_info(&self, window: Option<i32>) {
        // Capitalizes first letter of the string, i.e. converts foo to Foo
        let capitalize_first = |s: &str| {
            let mut c = s.chars();

            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().chain(c).collect(),
            }
        };

        // Don't add '\n' at the end, so that it will appear in front of icon
        // name, printed after it
        print!("{}", self.config.gap);
        io::stdout().flush().unwrap();

        match window {
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
        self.monitor
            .destroy_icons_flag
            .store(true, Ordering::SeqCst);

        let icons_threads = mem::take(&mut self.monitor.icons_threads);

        for thread in icons_threads {
            thread.join().unwrap();
        }

        self.monitor
            .destroy_icons_flag
            .store(false, Ordering::SeqCst);
    }

    pub fn process_focused_window(&mut self, window_id: i32) {
        if i3::is_window_fullscreen(window_id) {
            self.process_fullscreen_window();
            return;
        }

        let icon_name = i3::get_icon_name(window_id);

        self.print_info(Some(window_id));
        self.monitor.state.update_icon(&icon_name);
        self.process_icon(window_id);
    }

    pub fn process_fullscreen_window(&mut self) {
        self.destroy_prev_icons();

        // Reset icons, so that we can use process_focused_window
        // after. Otherwise it will not display icon, since app
        // name didn't change during fullscreen toggling
        self.monitor.state.reset_icons();
    }

    pub fn process_empty_desktop(&mut self) {
        self.destroy_prev_icons();
        self.monitor.state.reset_icons();
        self.print_info(None);
    }

    pub fn get_focused_desktop_id(&mut self) -> Option<i32> {
        i3::get_focused_desktop_id(&mut self.connection, &self.monitor.name)
    }

    pub fn get_focused_window_id(&mut self) -> Option<i32> {
        i3::get_focused_window_id(&mut self.connection, &self.monitor.name)
    }

    pub fn is_curr_desk_empty(&mut self) -> bool {
        match self.get_focused_desktop_id() {
            Some(curr_desk) => {
                i3::is_desk_empty(&mut self.connection, curr_desk)
            }
            None => panic!("Can't know if non-existing desktop empty or not"),
        }
    }
}
