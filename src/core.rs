use i3ipc::I3Connection;

use anyhow::bail;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::str;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use x11rb::connection::Connection;
use x11rb::protocol::xproto::ConnectionExt;
use x11rb::rust_connection::RustConnection;

use crate::bspwm::BspwmConnection;
use crate::config::{
    self, BspwmConfig, Config, EmptyInfo, I3Config, WindowInfo,
};
use crate::i3_utils;
use crate::wm_connection::WmConnection;
use crate::x11_utils;

#[derive(Debug, Clone)]
struct State {
    prev_window: Window,
    curr_window: Window,
}

#[derive(Debug, Clone)]
struct Window {
    fullscreen: bool,
    id: u32,
    name: String,
}

#[derive(Debug, Clone)]
struct Icon {
    path: Option<PathBuf>,
    app_name: String,
    x: u32,
    y: u32,
    visible: bool,
}

#[derive(Debug, Clone)]
struct Bar {
    icon: Icon,
    info: String,
    state: State,
}

#[derive(Debug, Clone)]
struct Monitor {
    name: String,
    desktops_number: u32,
    bar: Bar,
}


pub struct WmCore<W, C>
where
    W: WmConnection,
    C: Config,
{
    config: C,
    pub wm_connection: W,
    x11rb_connection: RustConnection,
    monitor: Monitor,
}

impl<W, C> WmCore<W, C>
where
    W: WmConnection,
    C: Config,
    WmCore<W, C>: CoreFeatures<W, C>,
{
    pub fn process_start(&mut self) {
        todo!();
    }

    fn generate_icon(&self, window_id: u32) -> anyhow::Result<()> {
        todo!();
    }

    fn display_icon(&mut self, icon_path: &str) {
        todo!();
    }

    fn curr_desk_contains_fullscreen(&mut self) -> bool {
        todo!();
    }

    fn update_desktops_number(&mut self) {
        self.monitor.desktops_number =
            self.wm_connection.get_desktops_number(&self.monitor.name);
    }

    fn update_window_or_empty(&mut self, window_id: Option<u32>) {
        todo!();
    }

    fn show_icon(&mut self) -> bool {
        !self.curr_desk_contains_fullscreen()
    }

    fn need_update_icon(&mut self) -> bool {
        todo!();
    }

    fn update_icon(&mut self, window_id: u32) {
        todo!();
    }

    fn print_info(&mut self, window_id: Option<u32>) {
        todo!();
    }

    fn watch_and_print_info(&mut self) {
        todo!();
    }

    fn destroy_prev_icon(&mut self) {
        todo!();
    }

    pub fn process_focused_window(&mut self, window_id: u32) {
        todo!();
    }

    pub fn process_fullscreen_window(&mut self) {
        self.destroy_prev_icon();
    }

    pub fn process_empty_desktop(&mut self) {
        self.destroy_prev_icon();
        self.print_info(None);
        self.update_window_or_empty(None);
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


pub trait CoreFeatures<W, C>
where
    W: WmConnection,
    C: Config,
{
    fn init(
        monitor_name: Option<&str>,
        config_option: Option<&str>,
    ) -> WmCore<W, C>;

    // TODO: remove default blank implementation
    fn update_icon_position(&mut self) {}
}

impl CoreFeatures<I3Connection, I3Config> for WmCore<I3Connection, I3Config> {
    fn init(monitor_name: Option<&str>, config_option: Option<&str>) -> Self {
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
}

impl CoreFeatures<BspwmConnection, BspwmConfig>
    for WmCore<BspwmConnection, BspwmConfig>
{
    fn init(monitor_name: Option<&str>, config_option: Option<&str>) -> Self {
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
}
