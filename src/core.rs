use i3ipc::I3Connection;

use std::error::Error;
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

#[derive(Debug, PartialEq)]
enum IconName {
    Empty, // For empty desktop, it's not a real icon, just empty space
    Name(String),
}

#[derive(Debug, Default)]
struct State {
    curr_icon_name: Option<IconName>,
    prev_icon_name: Option<IconName>,
    curr_x: i16,
}

impl State {
    fn init() -> Self {
        // We don't care about `curr_x` value here, because it will be set to
        // real one in `process_start`
        Self::default()
    }

    fn update_icon_name(&mut self, icon_name: IconName) {
        self.prev_icon_name = self.curr_icon_name.take();
        self.curr_icon_name = Some(icon_name);
    }

    fn get_curr_icon_name(&self) -> &str {
        match self.curr_icon_name.as_ref().unwrap() {
            IconName::Empty => panic!("No icon name, it is set to Empty"),
            IconName::Name(name) => name,
        }
    }
}

#[derive(Debug, Default)]
struct Monitor {
    state: State,
    name: String,
    prev_icon_id: Option<u32>,
    prev_window_fullscreen: Option<bool>,
    curr_window_fullscreen: Option<bool>,
    desktops_number: u32,
}

impl Monitor {
    fn init(monitor_name: Option<String>) -> Self {
        let name = match monitor_name {
            Some(x) => x,
            None => x11_utils::get_primary_monitor_name()
                .expect("Couldn't get name of primary monitor"),
        };

        let state = State::init();

        Self {
            name,
            state,
            ..Default::default()
        }
    }

    fn update_fullscreen_status(&mut self, flag: bool) {
        self.prev_window_fullscreen = self.curr_window_fullscreen;
        self.curr_window_fullscreen = Some(flag);
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

pub trait CoreFeatures<W, C>
where
    W: WMConnection,
    C: Config,
{
    fn init(
        monitor_name: Option<String>,
        config_option: Option<&str>,
    ) -> Core<W, C>;
    fn update_x(&mut self);
}

impl CoreFeatures<I3Connection, I3Config> for Core<I3Connection, I3Config> {
    fn init(monitor_name: Option<String>, config_option: Option<&str>) -> Self {
        let wm_connection =
            I3Connection::connect().expect("Failed to connect to i3");
        let config = config::load_i3(config_option);
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
        let desks_num = i3_utils::get_desktops_number(
            &mut self.wm_connection,
            &self.monitor.name,
        );

        self.monitor.state.curr_x = ((config.x() as f32)
            + config.gap_per_desk * (desks_num as f32))
            as i16;
    }
}

impl CoreFeatures<BspwmConnection, BspwmConfig>
    for Core<BspwmConnection, BspwmConfig>
{
    fn init(monitor_name: Option<String>, config_option: Option<&str>) -> Self {
        let wm_connection = BspwmConnection::new();
        let config = config::load_bspwm(config_option);
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
    Core<W, C>: CoreFeatures<W, C>,
{
    pub fn process_start(&mut self) {
        self.update_x();

        if let Some(window_id) = self.get_focused_window_id() {
            self.process_focused_window(window_id);
        } else {
            self.process_empty_desktop();
        }
    }

    fn generate_icon(&self, window_id: u32) -> Result<(), Box<dyn Error>> {
        let config = &self.config;

        if !Path::new(config.cache_dir()).is_dir() {
            fs::create_dir_all(config.cache_dir())
                .expect("No cache folder was detected and couldn't create it");
        }

        x11_utils::generate_icon(
            self.monitor.state.get_curr_icon_name(),
            config.cache_dir(),
            config.color(),
            window_id,
        )
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

    fn update_desktops_number(&mut self) {
        self.monitor.desktops_number =
            self.wm_connection.get_desktops_number(&self.monitor.name);
    }

    fn show_icon(&mut self) -> bool {
        !self.curr_desk_contains_fullscreen()
    }

    fn need_update_icon(&mut self) -> bool {
        if self.monitor.prev_window_fullscreen == Some(true)
            && self.monitor.curr_window_fullscreen == Some(false)
        {
            return true;
        }

        let old_desk_count = self.monitor.desktops_number;
        self.update_desktops_number();

        if old_desk_count != self.monitor.desktops_number {
            return true;
        }

        self.monitor.state.prev_icon_name != self.monitor.state.curr_icon_name
    }

    fn update_icon(&mut self, window_id: u32) {
        let state = &self.monitor.state;
        let config = &self.config;
        let icon_name = state.get_curr_icon_name();
        let icon_path = format!("{}/{}.jpg", &config.cache_dir(), icon_name);

        // Destroy icon first, before trying to extract it. Fixes the icon
        // still showing, when switching from window that has icon to the
        // widow that doens't.
        self.destroy_prev_icon();

        if !Path::new(&icon_path).exists() {
            // Repeatedly try to retrieve icon and save it
            let mut timeout_icon = 1000;
            while self.generate_icon(window_id).is_err() && timeout_icon > 0 {
                thread::sleep(Duration::from_millis(100));
                timeout_icon -= 100;
            }
        }

        self.update_x();
        self.display_icon(&icon_path);
    }

    fn print_info(&mut self) {
        let state = &self.monitor.state;

        if state.curr_icon_name.is_none() {
            println!("Desktop");
            return;
        }

        if state.prev_icon_name == state.curr_icon_name {
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
        print!(
            "{}",
            match state.curr_icon_name.as_ref().unwrap() {
                IconName::Empty => "",
                IconName::Name(_) => self.config.gap(),
            }
        );
        io::stdout().flush().unwrap();

        match state.curr_icon_name.as_ref().unwrap() {
            IconName::Empty => println!("Desktop"),

            IconName::Name(icon_name) => match icon_name.as_str() {
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

    pub fn process_focused_window(&mut self, window_id: u32) {
        let mut timeout_name = 1000;
        while timeout_name > 0 {
            if let Some(name) = self.wm_connection.get_icon_name(window_id) {
                if !name.is_empty() {
                    break;
                }
            }

            thread::sleep(Duration::from_millis(100));
            timeout_name -= 100;
        }

        let icon_name = self
            .wm_connection
            .get_icon_name(window_id)
            .unwrap_or(String::new());

        self.monitor
            .state
            .update_icon_name(IconName::Name(icon_name));

        self.print_info();

        if self.wm_connection.is_window_fullscreen(window_id) {
            self.monitor.update_fullscreen_status(true);
            self.process_fullscreen_window();
        } else {
            self.monitor.update_fullscreen_status(false);

            if self.show_icon() && self.need_update_icon() {
                self.update_icon(window_id);
            }
        }
    }

    pub fn process_fullscreen_window(&mut self) {
        self.destroy_prev_icon();
    }

    pub fn process_empty_desktop(&mut self) {
        self.destroy_prev_icon();
        self.monitor.state.update_icon_name(IconName::Empty);
        self.print_info();
    }

    pub fn get_focused_desktop_id(&mut self) -> Option<u32> {
        self.wm_connection
            .get_focused_desktop_id(&self.monitor.name)
    }

    pub fn get_focused_window_id(&mut self) -> Option<u32> {
        self.wm_connection.get_focused_window_id(&self.monitor.name)
    }

    pub fn get_fullscreen_window_id(&mut self, desktop_id: u32) -> Option<u32> {
        self.wm_connection.get_fullscreen_window_id(desktop_id)
    }

    pub fn is_desk_empty(&mut self, desktop_id: u32) -> bool {
        self.wm_connection.is_desk_empty(desktop_id)
    }
}
