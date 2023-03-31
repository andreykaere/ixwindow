use i3ipc::I3Connection;

use std::error::Error;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::str;
use std::sync::Mutex;
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
struct IconState {
    curr_name: Option<String>,
    prev_name: Option<String>,
    prev_id: Option<u32>,
    curr_x: i16,
}

impl IconState {
    fn update_name(&mut self, icon_name: &str) {
        self.prev_name = self.curr_name.take();
        self.curr_name = Some(icon_name.to_string());
    }

    fn get_curr_name(&self) -> &str {
        self.curr_name
            .as_ref()
            .expect("No icon name, it is set to Empty")
    }
}

#[derive(Debug, Default)]
struct FullscreenState {
    prev_window: Option<bool>,
    curr_window: Option<bool>,
}

impl FullscreenState {
    fn update_fullscreen_state(&mut self, flag: bool) {
        self.prev_window = self.curr_window;
        self.curr_window = Some(flag);
    }
}

#[derive(Debug, Default)]
struct Window {
    icon: IconState,
    info: String,
}

#[derive(Debug, Default)]
struct Monitor {
    name: String,
    desktops_number: u32,
    fullscreen_state: FullscreenState,
    window: Option<Window>,
    // window: Mutex<Window>,
}

impl Monitor {
    fn init(monitor_name: Option<String>) -> Self {
        let name = match monitor_name {
            Some(x) => x,
            None => x11_utils::get_primary_monitor_name()
                .expect("Couldn't get name of primary monitor"),
        };

        Self {
            name,
            ..Default::default()
        }
    }

    fn update_fullscreen_status(&mut self, flag: bool) {
        self.fullscreen_state.update_fullscreen_state(flag);
    }

    fn update_window(&mut self, icon_name: &str, window_info: &str) {
        if let Some(window) = self.window.as_mut() {
            window.info = window_info.to_string();
            window.icon.update_name(icon_name);
        } else {
            let mut window = Window {
                info: window_info.to_string(),
                ..Default::default()
            };
            window.icon.update_name(icon_name);

            self.window = Some(window);
        }
    }

    fn update_window_info(&mut self, window_info: &str) {
        if let Some(window) = self.window.as_mut() {
            window.info = window_info.to_string();
        }
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
        if let Some(window) = self.monitor.window.as_mut() {
            let config = &self.config;
            let desks_num = i3_utils::get_desktops_number(
                &mut self.wm_connection,
                &self.monitor.name,
            );

            window.icon.curr_x = ((config.x() as f32)
                + config.gap_per_desk * (desks_num as f32))
                as i16;
        }
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
        if let Some(window) = self.monitor.window.as_mut() {
            window.icon.curr_x = self.config.x();
        }
    }
}

impl<W, C> Core<W, C>
where
    W: WMConnection + std::marker::Send,
    C: Config + std::marker::Send,
    Core<W, C>: CoreFeatures<W, C>,
{
    pub fn process_start(&mut self) {
        // Run process for watching for window's info change and printing info
        // to the bar
        self.update_and_print_info();

        if let Some(window_id) = self.get_focused_window_id() {
            self.process_focused_window(window_id);
        } else {
            self.process_empty_desktop();
        }
    }

    fn generate_icon(&self, window_id: u32) -> Result<(), Box<dyn Error>> {
        let config = &self.config;
        let window = self
            .monitor
            .window
            .as_ref()
            .expect("Don't generate icon for not window");

        if !Path::new(config.cache_dir()).is_dir() {
            fs::create_dir(config.cache_dir())
                .expect("No cache folder was detected and couldn't create it");
        }

        x11_utils::generate_icon(
            window.icon.get_curr_name(),
            config.cache_dir(),
            config.color(),
            window_id,
        )
    }

    fn display_icon(&mut self, icon_path: &str) {
        let config = &self.config;
        let window = self
            .monitor
            .window
            .as_mut()
            .expect("Don't display icon for not window");

        let (curr_x, y, size, monitor_name) = (
            window.icon.curr_x,
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

        window.icon.prev_id = icon_id;
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

    fn update_window(&mut self, window_id: Option<u32>) {
        match window_id {
            Some(win_id) => {
                let print_info_type =
                    self.config.window_info_settings().info_type;

                let window_info = self
                    .wm_connection
                    .get_window_info(win_id, print_info_type)
                    .unwrap_or(String::new());

                let icon_name = self
                    .wm_connection
                    .get_icon_name(win_id)
                    .unwrap_or(String::new());

                self.monitor.update_window(&icon_name, &window_info);
            }

            None => {
                self.monitor.window = None;
            }
        }

        // TODO
        // self.update_window_info_status(WindowInfo::Info(window_info));
    }

    fn show_icon(&mut self) -> bool {
        !self.curr_desk_contains_fullscreen()
    }

    fn need_update_icon(&mut self) -> bool {
        if self.monitor.fullscreen_state.prev_window == Some(true)
            && self.monitor.fullscreen_state.curr_window == Some(false)
        {
            return true;
        }

        let old_desk_count = self.monitor.desktops_number;
        self.update_desktops_number();

        if old_desk_count != self.monitor.desktops_number {
            return true;
        }

        if let Some(window) = self.monitor.window.as_ref() {
            window.icon.prev_name != window.icon.curr_name
        } else {
            false
        }
    }

    fn update_icon(&mut self, window_id: u32) {
        let config = &self.config;
        let icon_name = match self.monitor.window.as_ref() {
            Some(window) => window.icon.get_curr_name(),

            None => {
                return;
            }
        };

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
        let window = &self.monitor.window;

        // Don't add '\n' at the end, so that it will appear in front of icon
        // name, printed after it
        print!("{}", self.config.gap());
        io::stdout().flush().unwrap();

        let info = match window {
            None => "Empty",
            Some(win) => &win.info,
        };

        println!("{}", self.config.window_info_settings().format_info(info));
    }

    fn destroy_prev_icon(&mut self) {
        let conn = &self.x11rb_connection;
        let window = self
            .monitor
            .window
            .as_mut()
            .expect("Don't destroy previous icon for not window");

        if let Some(id) = window.icon.prev_id {
            conn.destroy_window(id)
                .expect("Failed to destroy previous icon");
            conn.flush().unwrap();

            window.icon.prev_id = None;
        }
    }

    // This function prints info of the window and watches for update in the
    // info of window
    fn update_and_print_info(&mut self) {
        // TODO: Change loop to actual checking for get_property
        // thread::spawn(move || loop {
        //     self.print_info();

        //     thread::sleep(Duration::from_millis(100));
        // });
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

        self.update_window(Some(window_id));
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
        self.update_window(None);
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
