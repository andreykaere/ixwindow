use anyhow::bail;
use i3ipc::I3Connection;
use tokio::sync::mpsc;

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

#[derive(Debug, Clone)]
enum InfoRaw {
    WindowInfo(WindowInfo),
    EmptyInfo(EmptyInfo),
}

impl Default for InfoRaw {
    fn default() -> Self {
        Self::EmptyInfo(EmptyInfo::default())
    }
}

impl InfoRaw {
    fn print(&self) {
        println!("{:?}", self);
    }
}

#[derive(Debug, Clone, Default)]
struct Info {
    info: InfoRaw,
    sender: Option<mpsc::Sender<Package<InfoRaw>>>,
}


#[derive(Debug, Clone, Default)]
struct Bar {
    icon: Option<Icon>,
    info: Info,
    state: State,
}

impl Bar {
    fn set_empty_info(&mut self) {
        // self.info = InfoRaw::EmptyInfo(EmptyInfo::default());
        todo!();
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

enum Package<T> {
    Stop,
    Message(T),
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
    fn watch_info(&self, mut watcher: mpsc::Receiver<Package<InfoRaw>>) {
        tokio::spawn(async move {
            while let Some(Package::Message(info)) = watcher.recv().await {
                info.print();
            }
        });
    }

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

    async fn drop_window(&mut self) -> anyhow::Result<()> {
        self.destroy_icon();

        let info = &self.monitor.bar.info;

        if let Some(sender) = &info.sender {
            sender.send(Package::Stop).await?;
        }

        Ok(())
    }

    fn display_icon(&mut self) -> anyhow::Result<()> {
        let bar = &mut self.monitor.bar;

        if let Some(icon) = bar.icon.as_mut() {
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

            let conn = &self.x11rb_connection;

            // Destroy previous icon, because otherwise there will be 100500
            // icons and it will slow down WM
            conn.destroy_window(old_icon_id).ok(); // TODO: add logging
            conn.flush()?;
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

    pub fn process_focused_window(&mut self, window_id: u32) {
        let (sender, mut receiver) = mpsc::channel(100);

        // TODO: getting real info
        let raw_info = InfoRaw::WindowInfo(WindowInfo {
            info: "Foo".to_string(),
            ..Default::default()
        });

        let info = Info {
            info: raw_info,
            sender: Some(sender),
        };

        // TODO: getting real icon
        // It's okay to put 0 for id here, because it will be changed when
        // displaying the icon
        let icon = Icon {
            path: PathBuf::from("/home/andrey/.config/polybar/scripts/ixwindow/polybar-icons/Alacritty.jpg"),
            id: 0,
            app_name: "Bar".to_string(),
            x: 100,
            y: 100,
            size: 20,
            visible: true,
        };

        let bar = &mut self.monitor.bar;

        bar.info = info;
        bar.icon = Some(icon);

        self.display_icon();

        self.watch_info(receiver);
    }

    pub fn process_fullscreen_window(&mut self) {
        todo!();
    }

    pub fn process_empty_desktop(&mut self) {
        self.drop_window();

        let bar = &mut self.monitor.bar;
        bar.set_empty_info();
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
