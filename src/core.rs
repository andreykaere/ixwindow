use i3ipc::I3Connection;

use anyhow::bail;
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
use crate::config::{
    self, BspwmConfig, Config, EmptyInfo, I3Config, WindowInfo,
};
use crate::i3_utils;
use crate::wm_connection::WmConnection;
use crate::x11_utils;

#[derive(Debug, Clone, Default)]
struct State {
    prev_window: Option<Window>,
    curr_window: Option<Window>,
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
enum Info {
    Text(String),
    EmptyLabel(String),
}

impl Default for Info {
    fn default() -> Self {
        Self::EmptyLabel("Empty".to_string())
    }
}

#[derive(Debug, Clone, Default)]
struct Bar {
    icon: Option<Icon>,
    info: Info,
    state: State,
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
    C: Config,
    WmCore<W, C>: WmCoreFeatures<W, C>,
{
    pub fn process_start(&mut self) {
        todo!();
    }

    pub fn process_focused_window(&mut self, window_id: u32) {
        todo!();
    }

    pub fn process_fullscreen_window(&mut self) {
        todo!();
    }

    pub fn process_empty_desktop(&mut self) {
        todo!();
    }

    pub fn get_focused_desktop_id(&mut self) -> Option<u32> {
        todo!();
    }

    pub fn get_focused_window_id(&mut self) -> Option<u32> {
        todo!();
    }

    pub fn get_fullscreen_window_id(&mut self, desktop_id: u32) -> Option<u32> {
        todo!();
    }

    pub fn is_desk_empty(&mut self, desktop_id: u32) -> bool {
        todo!();
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
