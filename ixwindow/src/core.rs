use i3ipc::I3Connection;

use std::fs;
use std::io::{self, Write};
use std::path::Path;

use std::str;
use std::thread;
use std::time::Duration;

use x11rb::connection::Connection;
use x11rb::protocol::xproto::ConnectionExt;
use x11rb::rust_connection::RustConnection;

use crate::bspwm::BspwmConnection;
use crate::config::{self, BspwmConfig, Config, I3Config};
use crate::i3_utils;
use crate::wm_connection::WMConnection;
use crate::x11_utils;

#[derive(Debug, Default)]
struct State {
    curr_icon: Option<String>,
    prev_icon: Option<String>,
    curr_x: i16,
}

impl State {
    fn init() -> Self {
        // We don't care about `curr_x` value, because it will be set to real
        // one in `process_start`
        Self::default()
    }

    fn update_icon(&mut self, icon_name: Option<&str>) {
        self.prev_icon = self.curr_icon.take();
        self.curr_icon = icon_name.map(|x| x.to_string());
    }
}

#[derive(Debug)]
struct Monitor {
    state: State,
    name: String,
    prev_icon_id: Option<u32>,
    prev_window_fullscreen: bool,
    curr_window_fullscreen: bool,
}

impl Monitor {
    fn init(monitor_name: Option<String>) -> Self {
        let name = match monitor_name {
            Some(x) => x,
            None => x11_utils::get_primary_monitor_name()
                .expect("Couldn't get name of primary monitor"),
        };

        let state = State::init();
        let prev_icon_id = None;
        let prev_window_fullscreen = false;
        let curr_window_fullscreen = false;

        Self {
            name,
            state,
            prev_icon_id,
            prev_window_fullscreen,
            curr_window_fullscreen,
        }
    }

    fn update_fullscreen(&mut self, flag: bool) {
        self.prev_window_fullscreen = self.curr_window_fullscreen;
        self.curr_window_fullscreen = flag;
    }
}

pub struct Core<W, C>
where
    W: WMConnection,
    C: Config,
{
    config: C,
    pub wm_connection: W,
    x11rb_connection: RustConnection,
    monitor: Monitor,
}

pub trait ConfigFeatures<W, C>
where
    W: WMConnection,
    C: Config,
{
    fn init(monitor_name: Option<String>) -> Core<W, C>;
    fn update_x(&mut self);
}

impl ConfigFeatures<I3Connection, I3Config> for Core<I3Connection, I3Config> {
    fn init(monitor_name: Option<String>) -> Self {
        let wm_connection =
            I3Connection::connect().expect("Failed to connect to i3");
        let config = config::load_i3();
        let monitor = Monitor::init(monitor_name);
        let (x11rb_connection, _) = x11rb::connect(None).unwrap();

        Self {
            config,
            wm_connection,
            monitor,
            x11rb_connection,
        }
    }

    fn update_x(&mut self) {
        let config = &self.config;
        let desks_num = i3_utils::get_desks_on_mon(
            &mut self.wm_connection,
            &self.monitor.name,
        )
        .len();

        self.monitor.state.curr_x = ((config.x() as f32)
            + config.gap_per_desk * (desks_num as f32))
            as i16;
    }
}

impl ConfigFeatures<BspwmConnection, BspwmConfig>
    for Core<BspwmConnection, BspwmConfig>
{
    fn init(monitor_name: Option<String>) -> Self {
        let wm_connection = BspwmConnection::new();
        let config = config::load_bspwm();
        let monitor = Monitor::init(monitor_name);
        let (x11rb_connection, _) = x11rb::connect(None).unwrap();

        Self {
            config,
            wm_connection,
            monitor,
            x11rb_connection,
        }
    }

    fn update_x(&mut self) {
        self.monitor.state.curr_x = self.config.x();
    }
}

impl<W, C> Core<W, C>
where
    W: WMConnection,
    C: Config,
    Core<W, C>: ConfigFeatures<W, C>,
{
    pub fn process_start(&mut self) {
        self.update_x();

        if let Some(window_id) = self.get_focused_window_id() {
            self.process_focused_window(window_id);
        } else {
            self.process_empty_desktop();
        }
    }

    fn generate_icon(&self, window_id: i32) {
        let config = &self.config;

        if !Path::new(config.cache_dir()).is_dir() {
            fs::create_dir(config.cache_dir())
                .expect("No cache folder was detected and couldn't create it");
        }

        x11_utils::generate_icon(
            self.monitor.state.curr_icon.as_ref().unwrap(),
            config.cache_dir(),
            config.color(),
            window_id,
        )
        .unwrap_or(());
    }

    fn display_icon(&mut self, icon_path: &str) {
        let config = &self.config;

        let (curr_x, y, size, monitor_name) = (
            self.monitor.state.curr_x,
            config.y(),
            config.size(),
            &self.monitor.name,
        );

        let icon_id = x11_utils::display_icon(
            &self.x11rb_connection,
            icon_path,
            curr_x,
            y,
            size,
            monitor_name,
        )
        .ok();

        self.monitor.prev_icon_id = icon_id;
    }

    fn curr_desk_contains_fullscreen(&mut self) -> bool {
        let current_desktop = self
            .wm_connection
            .get_focused_desktop_id(&self.monitor.name);

        if let Some(desktop) = current_desktop {
            return self
                .wm_connection
                .get_fullscreen_window_id(desktop)
                .is_some();
        }

        false
    }

    fn show_icon(&mut self) -> bool {
        !self.curr_desk_contains_fullscreen()
            && (self.monitor.state.prev_icon != self.monitor.state.curr_icon
                || self.monitor.prev_window_fullscreen)

        // if self.curr_desk_contains_fullscreen() {
        //     return false;
        // } else if self.monitor.state.prev_icon == self.monitor.state.curr_icon
        //     && !self.monitor.prev_window_fullscreen
        // {
        //     return false;
        // }

        // true
    }

    fn process_icon(&mut self, window_id: i32) {
        if !self.show_icon() {
            return;
        }

        let state = &self.monitor.state;

        // curr_icon is not `None`, because we put the current icon name before
        // calling `process_icon`
        let icon_name = state.curr_icon.as_ref().unwrap();

        let config = &self.config;
        let icon_path = format!("{}/{}.jpg", &config.cache_dir(), icon_name);

        if !Path::new(&icon_path).exists() {
            self.generate_icon(window_id);
        }

        self.destroy_prev_icon();
        self.display_icon(&icon_path);
    }

    fn print_info(&mut self) {
        let state = &self.monitor.state;

        if state.prev_icon == state.curr_icon {
            return;
        }

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
        print!("{}", self.config.gap());
        io::stdout().flush().unwrap();

        match &state.curr_icon {
            None => println!("Empty"),

            Some(icon_name) => match icon_name.as_str() {
                "Brave-browser" => println!("Brave"),
                "TelegramDesktop" => println!("Telegram"),
                _ => println!("{}", capitalize_first(icon_name)),
            },
        }
    }

    fn destroy_prev_icon(&mut self) {
        let conn = &self.x11rb_connection;

        if let Some(id) = self.monitor.prev_icon_id {
            conn.destroy_window(id)
                .expect("Failed to destroy previous icon");
            conn.flush().unwrap();

            self.monitor.prev_icon_id = None;
        }
    }

    pub fn process_focused_window(&mut self, window_id: i32) {
        // let icon_name = self.wm_connection.get_icon_name(window_id).unwrap();
        let mut timeout = 2000;

        while self.wm_connection.get_icon_name(window_id).is_none()
            && timeout > 0
        {
            thread::sleep(Duration::from_millis(100));
            timeout -= 100;
        }

        let icon_name = self
            .wm_connection
            .get_icon_name(window_id)
            .unwrap_or(String::new());

        self.monitor.state.update_icon(Some(&icon_name));
        self.print_info();

        if self.wm_connection.is_window_fullscreen(window_id) {
            self.monitor.update_fullscreen(true);
            self.process_fullscreen_window();
        } else {
            self.monitor.update_fullscreen(false);
            self.process_icon(window_id);
        }
    }

    pub fn process_fullscreen_window(&mut self) {
        self.destroy_prev_icon();
    }

    pub fn process_empty_desktop(&mut self) {
        self.destroy_prev_icon();
        self.monitor.state.update_icon(None);
        self.print_info();
    }

    pub fn get_focused_desktop_id(&mut self) -> Option<i32> {
        self.wm_connection
            .get_focused_desktop_id(&self.monitor.name)
    }

    pub fn get_focused_window_id(&mut self) -> Option<i32> {
        self.wm_connection.get_focused_window_id(&self.monitor.name)
    }

    pub fn is_curr_desk_empty(&mut self) -> bool {
        match self.get_focused_desktop_id() {
            Some(curr_desk) => self.wm_connection.is_desk_empty(curr_desk),
            None => panic!("Can't know if non-existing desktop empty or not"),
        }
    }

    pub fn get_fullscreen_window_id(&mut self, desktop_id: i32) -> Option<i32> {
        self.wm_connection.get_fullscreen_window_id(desktop_id)
    }

    pub fn is_desk_empty(&mut self, desktop_id: i32) -> bool {
        self.wm_connection.is_desk_empty(desktop_id)
    }
}
