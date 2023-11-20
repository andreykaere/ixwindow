use anyhow::bail;
use i3ipc::I3Connection;
use std::sync::mpsc::{self, Receiver, Sender};

use std::fs;
use std::path::{Path, PathBuf};
use std::str;
use std::sync::{Arc, Mutex};
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
    app_name: String,
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
        match self {
            Info::EmptyInfo(empty_info) => empty_info.print(config),
            Info::WindowInfo(window_info) => unimplemented!(),
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
    sender: Option<Sender<Signal>>,
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
    desktops_number: u32,
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
    fn watch_and_print_info(&self, mut signal_recv: Receiver<Signal>) {
        let window_id = self.monitor.bar.state.curr_window.clone().unwrap().id;
        let info_types = self.config.print_info_settings().info_types.clone();
        let config = self.config.clone();
        let mut window_info = Default::default();
        let mut prev_window_info = Default::default();


        thread::spawn(move || loop {
            if let Ok(signal) = signal_recv.try_recv() {
                if let Signal::Stop = signal {
                    break;
                }
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

    // fn watch_info(&self, mut watcher: mpsc::Receiver<Package<Info>>) {
    // }

    fn destroy_icon(&mut self) -> anyhow::Result<()> {
        let conn = &self.x11rb_connection;
        let bar = &mut self.monitor.bar;

        if let Some(icon) = &bar.icon {
            conn.destroy_window(icon.id).ok(); // TODO: add logging
            conn.flush()?;

            bar.icon = None;
        }

        Ok(())
    }

    fn stop_watch_and_print_info(&self) -> anyhow::Result<()> {
        if let Some(sender) = &self.monitor.bar.sender {
            sender.send(Signal::Stop)?;
        }

        Ok(())
    }

    fn drop_window(&mut self) -> anyhow::Result<()> {
        self.destroy_icon();
        self.stop_watch_and_print_info()?;

        Ok(())
    }

    fn display_icon(&mut self) -> anyhow::Result<()> {
        let bar = &mut self.monitor.bar;

        if let Some(icon) = bar.icon.as_mut() {
            if !icon.visible {
                return Ok(());
            }

            let old_icon_id = icon.id;

            let new_icon_id = x11_utils::display_icon(
                &self.x11rb_connection,
                &icon.path,
                icon.x,
                icon.y,
                icon.size,
                &self.monitor.name,
            )?;
            icon.id = new_icon_id;

            // let conn = &self.x11rb_connection;

            // Destroy previous icon, because otherwise there will be 100500
            // icons and it will slow down WM
            // conn.destroy_window(old_icon_id).ok(); // TODO: add logging
            // conn.flush()?;
        }

        Ok(())
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
            .get_icon_name(window_id)
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

    fn new_icon(&mut self, window_id: u32) -> Icon {
        // TODO: add logging in case of no icon name
        let icon_name = self
            .wm_connection
            .get_icon_name(window_id)
            .unwrap_or(String::new());

        let app_name = icon_name.clone();

        let cache_dir = self.config.cache_dir().to_string_lossy().to_string();
        let x = self.config.x();
        let y = self.config.y();
        let size = self.config.size();
        let icon_path = format!("{}/{}.jpg", cache_dir, &icon_name);

        // It's okay to put 0 for id here, because it will be changed when
        // displaying the icon
        Icon {
            path: PathBuf::from(&icon_path),
            id: 0,
            app_name,
            x,
            y,
            size,
            visible: self.is_icon_visible(),
        }
    }

    pub fn process_focused_window(&mut self, window_id: u32) {
        self.stop_watch_and_print_info();

        let (sender, receiver) = mpsc::channel();

        // Real info will be set later
        let info = Info::WindowInfo(WindowInfo::default());

        let window = self.new_window(window_id);


        let bar = &mut self.monitor.bar;
        bar.info = info;
        bar.sender = Some(sender);
        bar.state.prev_window = bar.state.curr_window.take();
        bar.state.curr_window = Some(window);

        // println!(
        //     "prev: {:?}, curr: {:?}",
        //     bar.state.prev_window, bar.state.curr_window
        // );

        if bar.state.prev_window.is_none() {
            let icon = self.new_icon(window_id);
            self.monitor.bar.icon = Some(icon);

            self.display_icon();
        } else {
            let prev_window = bar.state.prev_window.clone().unwrap();
            let curr_window = bar.state.curr_window.clone().unwrap();


            // TODO: think through HANDLE fullscreen toggle of the same app
            if prev_window.name == curr_window.name {
                if prev_window.fullscreen && !curr_window.fullscreen {
                    self.destroy_icon();

                    let icon = self.new_icon(window_id);
                    self.monitor.bar.icon = Some(icon);

                    self.display_icon();
                }

                if !prev_window.fullscreen && curr_window.fullscreen {
                    self.destroy_icon();
                }
            } else {
                self.destroy_icon();

                let icon = self.new_icon(window_id);
                self.monitor.bar.icon = Some(icon);

                self.display_icon();
            }
        }

        // println!("icon: {:#?}", self.monitor.bar.icon);

        self.watch_and_print_info(receiver);
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
        self.drop_window();
        self.set_empty_info();

        let bar = &mut self.monitor.bar;
        bar.info.print(&self.config);
        bar.state.update_empty();
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

    // TODO: remove default blank implementation
    fn update_icon_position(&mut self) {}
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
}
