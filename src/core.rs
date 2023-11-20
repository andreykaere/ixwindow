use i3ipc::I3Connection;
use std::sync::mpsc::{self, Receiver, Sender};

use std::fs;
use std::path::{Path, PathBuf};
use std::str;
use std::thread;
use std::time::Duration;

use x11rb::connection::Connection;
use x11rb::protocol::xproto::ConnectionExt;
use x11rb::rust_connection::RustConnection;

use crate::bspwm::BspwmConnection;
use crate::config::{self, BspwmConfig, Config, I3Config, WindowInfoType};
use crate::i3_utils;
use crate::wm_connection::WmConnection;
use crate::x11_utils;


// TODO: add icon async deriver

#[derive(Debug, Clone)]
struct Window {
    fullscreen: bool,
    id: u32,
    name: String,
}

#[derive(Debug, Clone, Default)]
struct State {
    prev_window: Option<Window>,
    curr_window: Option<Window>,
}

impl State {
    fn update_window(&mut self, new_window: &Window) {
        self.prev_window = self.curr_window.take();
        self.curr_window = Some(new_window.clone());
    }

    fn update_empty(&mut self) {
        self.prev_window = self.curr_window.take();
        self.curr_window = None;
    }
}

#[derive(Debug, Clone)]
struct Icon {
    path: PathBuf,
    id: u32,
    // app_name: String,
    x: i16,
    y: i16,
    size: u16,
    visible: bool,
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct WindowInfo {
    pub info: String,
    pub info_type: WindowInfoType,
}

impl WindowInfo {
    fn print(&self, config: &impl Config) {
        println!(
            "{}{}",
            config.gap(),
            config
                .print_info_settings()
                .format_info(&self.info, Some(self.info_type))
        );
    }
}

#[derive(Clone, Default, Debug)]
pub struct EmptyInfo {
    pub info: String,
}

impl EmptyInfo {
    fn print(&self, config: &impl Config) {
        println!("{}{}", config.gap(), &self.info);
    }
}


#[derive(Debug, Clone)]
enum Info {
    WindowInfo(WindowInfo),
    EmptyInfo(EmptyInfo),
}

impl Default for Info {
    fn default() -> Self {
        Self::EmptyInfo(EmptyInfo::default())
    }
}

impl Info {
    fn print(&self, config: &impl Config) {
        if let Info::EmptyInfo(empty_info) = self {
            empty_info.print(config);
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Signal {
    Stop,
}

#[derive(Debug, Clone, Default)]
struct Bar {
    icon: Option<Icon>,
    info: Info,
    state: State,
    info_controller: Option<Sender<Signal>>,
}

impl Bar {
    fn set_empty_info(&mut self, empty_info: &str) {
        self.info = Info::EmptyInfo(EmptyInfo {
            info: empty_info.to_string(),
        });
    }
}


#[derive(Debug, Clone, Default)]
struct Monitor {
    name: String,
    // desktops_number: u32,  // maybe will be useful for i3-desk changing
    bar: Bar,
}

impl Monitor {
    fn init(monitor_name: Option<&str>) -> Self {
        let name = match monitor_name {
            Some(x) => x.to_string(),
            None => x11_utils::get_primary_monitor_name()
                .expect("Couldn't get name of the primary monitor"),
        };

        Self {
            name,
            ..Default::default()
        }
    }
}

pub struct WmCore<W, C>
where
    W: WmConnection,
    C: Config,
{
    config: C,
    wm_connection: W,
    x11rb_connection: RustConnection,
    monitor: Monitor,
}

impl<W, C> WmCore<W, C>
where
    W: WmConnection,
    C: Config + Clone + std::marker::Send + 'static,
    WmCore<W, C>: WmCoreFeatures<W, C>,
{
    fn watch_and_print_info(&self, signal_recv: Receiver<Signal>) {
        let window_id = self.monitor.bar.state.curr_window.clone().unwrap().id;
        let info_types = self.config.print_info_settings().info_types.clone();
        let config = self.config.clone();
        let mut window_info = Default::default();
        let mut prev_window_info = Default::default();

        thread::spawn(move || loop {
            if signal_recv.try_recv().is_ok() {
                break;
            }

            // TODO: add logging
            match x11_utils::get_window_info(window_id, &info_types) {
                Ok(win_info) => {
                    prev_window_info = window_info;
                    window_info = win_info;
                }
                Err(_) => {}
            }

            if window_info != prev_window_info {
                window_info.print(&config);
            }

            thread::sleep(Duration::from_millis(100));
        });
    }

    fn destroy_icon(&mut self) {
        let conn = &self.x11rb_connection;
        let bar = &mut self.monitor.bar;

        if let Some(icon) = &bar.icon {
            // TODO: add logging
            conn.destroy_window(icon.id).ok(); // If couldn't destroy, don't do anything
            conn.flush().unwrap();

            bar.icon = None;
        }
    }

    fn stop_watch_and_print_info(&self) {
        if let Some(controller) = &self.monitor.bar.info_controller {
            controller.send(Signal::Stop).unwrap();
        }
    }

    // fn drop_window(&mut self) {
    //     self.stop_watch_and_print_info();
    //     self.destroy_icon();
    // }

    fn display_icon(&mut self) {
        let bar = &mut self.monitor.bar;

        if let Some(icon) = bar.icon.as_mut() {
            if !icon.visible {
                return;
            }

            if icon.path.is_file() {
                // TODO: add logging if couldn't display icon
                if let Ok(new_icon_id) = x11_utils::display_icon(
                    &self.x11rb_connection,
                    &icon.path,
                    icon.x,
                    icon.y,
                    icon.size,
                    &self.monitor.name,
                ) {
                    icon.id = new_icon_id;
                }
            }
        }
    }

    pub fn process_start(&mut self) {
        if let Some(window_id) = self.get_focused_window_id() {
            self.process_focused_window(window_id);
        } else {
            self.process_empty_desktop();
        }
    }

    fn new_window(&self, window_id: u32) -> Window {
        let window_name = self
            .wm_connection
            .get_window_name(window_id)
            .unwrap_or(String::new());

        Window {
            id: window_id,
            name: window_name,
            fullscreen: self.wm_connection.is_window_fullscreen(window_id),
        }
    }

    fn curr_desk_contains_fullscreen(&mut self) -> bool {
        let current_desktop = self
            .wm_connection
            .get_focused_desktop_id(&self.monitor.name);

        if let Some(desktop) = current_desktop {
            self.wm_connection
                .get_fullscreen_window_id(desktop)
                .is_some()
        } else {
            false
        }
    }

    fn is_icon_visible(&mut self) -> bool {
        !self.curr_desk_contains_fullscreen()
    }

    fn gen_icon_name(&self, window_id: u32) -> String {
        // TODO: add logging in case of no window name
        self.wm_connection
            .get_window_name(window_id)
            .unwrap_or(String::new())
    }

    fn gen_icon_path(&self, window_id: u32) -> PathBuf {
        PathBuf::from(format!(
            "{}/{}.jpg",
            self.config.cache_dir().to_string_lossy(),
            self.gen_icon_name(window_id)
        ))
    }

    fn new_icon(&mut self, window_id: u32) -> Icon {
        let x = self.config.x();
        let y = self.config.y();
        let size = self.config.size();
        let icon_path = self.gen_icon_path(window_id);

        // It's okay to put 0 for id here, because it will be changed when
        // displaying the icon
        Icon {
            path: icon_path,
            id: 0,
            x,
            y,
            size,
            visible: self.is_icon_visible(),
        }
    }

    fn update_icon(&mut self, window_id: u32) {
        let icon = self.new_icon(window_id);

        if !icon.path.is_file() {
            self.try_generate_icon(window_id);
            thread::sleep(Duration::from_millis(100)); // let icon be generated
        }

        self.monitor.bar.icon = Some(icon);
        self.update_icon_position();
        self.display_icon();
    }

    fn try_generate_icon(&self, window_id: u32) {
        if !self.config.cache_dir().is_dir() {
            fs::create_dir(self.config.cache_dir())
                .expect("Failed to create nonexisting cache directory");
        }

        let config = self.config.clone();
        let icon_name = self.gen_icon_name(window_id);
        let icon_path = self.gen_icon_path(window_id);

        thread::spawn(move || {
            let mut timeout = 3000;
            let mut response;

            while timeout > 0 && !icon_path.is_file() {
                response = x11_utils::generate_icon(
                    &icon_name,
                    config.cache_dir(),
                    config.color(),
                    window_id,
                );

                if response.is_ok() {
                    break;
                }

                thread::sleep(Duration::from_millis(100));
                timeout -= 100;
            }
        });
    }

    pub fn process_focused_window(&mut self, window_id: u32) {
        let window = self.new_window(window_id);
        self.monitor.bar.state.update_window(&window);

        if self.monitor.bar.state.prev_window.is_some() {
            self.stop_watch_and_print_info();
        }

        let (info_sender, info_receiver) = mpsc::channel();
        let info = Info::WindowInfo(WindowInfo::default()); // Real info will be set later

        let bar = &mut self.monitor.bar;
        bar.info = info;
        bar.info_controller = Some(info_sender);

        if bar.state.prev_window.is_none() {
            self.update_icon(window_id);
        } else {
            let prev_window = bar.state.prev_window.clone().unwrap();
            let curr_window = bar.state.curr_window.clone().unwrap();

            // TODO: think through HANDLE fullscreen toggle of the same app
            if prev_window.name == curr_window.name {
                if prev_window.fullscreen && !curr_window.fullscreen {
                    self.destroy_icon();
                    self.update_icon(window_id);
                }

                if !prev_window.fullscreen && curr_window.fullscreen {
                    self.destroy_icon();
                }
            } else {
                self.destroy_icon();
                self.update_icon(window_id);
            }
        }

        // println!("icon: {:#?}", self.monitor.bar.icon);

        self.watch_and_print_info(info_receiver);
    }

    // TODO: think through
    pub fn process_fullscreen_window(&mut self) {
        self.destroy_icon();
    }

    fn set_empty_info(&mut self) {
        let empty_info =
            self.config.print_info_settings().get_empty_desk_info();
        self.monitor.bar.set_empty_info(empty_info);
    }

    pub fn process_empty_desktop(&mut self) {
        self.monitor.bar.state.update_empty();

        if self.monitor.bar.state.prev_window.is_some() {
            self.stop_watch_and_print_info();
            self.destroy_icon();
        }

        self.set_empty_info();
        self.monitor.bar.info.print(&self.config);
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


pub trait WmCoreFeatures<W, C>
where
    W: WmConnection,
    C: Config,
{
    fn init(monitor_name: Option<&str>, config: Option<&Path>) -> WmCore<W, C>;

    fn update_icon_position(&mut self);
}

impl WmCoreFeatures<I3Connection, I3Config> for WmCore<I3Connection, I3Config> {
    fn init(monitor_name: Option<&str>, config_file: Option<&Path>) -> Self {
        let wm_connection =
            I3Connection::connect().expect("Failed to connect to i3");
        let config = config::load_i3(config_file);
        let monitor = Monitor::init(monitor_name);
        let (x11rb_connection, _) = x11rb::connect(None).unwrap();

        Self {
            config,
            wm_connection,
            monitor,
            x11rb_connection,
        }
    }

    fn update_icon_position(&mut self) {
        let config = &self.config;
        let desks_num = i3_utils::get_desktops_number(
            &mut self.wm_connection,
            &self.monitor.name,
        );

        if let Some(icon) = self.monitor.bar.icon.as_mut() {
            icon.x = ((config.x() as f32)
                + config.gap_per_desk * (desks_num as f32))
                as i16;
        }
    }
}

impl WmCoreFeatures<BspwmConnection, BspwmConfig>
    for WmCore<BspwmConnection, BspwmConfig>
{
    fn init(monitor_name: Option<&str>, config_file: Option<&Path>) -> Self {
        let wm_connection = BspwmConnection::new();
        let config = config::load_bspwm(config_file);
        let monitor = Monitor::init(monitor_name);
        let (x11rb_connection, _) = x11rb::connect(None).unwrap();

        Self {
            config,
            wm_connection,
            monitor,
            x11rb_connection,
        }
    }

    fn update_icon_position(&mut self) {}
}
