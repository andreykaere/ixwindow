use bspc_rs::properties::State;
use i3ipc::I3Connection;

use std::str;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use x11rb::rust_connection::RustConnection;

use crate::bspwm::BspwmConnection;
use crate::config::{
    self, BspwmConfig, Config, EmptyInfo, I3Config, WindowInfo,
};
use crate::i3_utils;
use crate::wm_connection::WMConnection;
use crate::x11_utils;

mod icon;

use icon::IconState;

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
    info: WindowInfo,
    id: u32,
}

#[derive(Debug)]
enum CurrentFocus {
    EmptyDesk(EmptyInfo),
    Window(Window),
}

// This is just dummy implementation, because we just need it for Monitor
// init
impl Default for CurrentFocus {
    fn default() -> Self {
        let empty_info = EmptyInfo {
            info: String::new(),
        };

        Self::EmptyDesk(empty_info)
    }
}

#[derive(Debug, Default)]
struct Monitor {
    name: String,
    desktops_number: u32,
    fullscreen_state: FullscreenState,
    current_focus: Arc<Mutex<CurrentFocus>>,
}

impl Monitor {
    fn init(monitor_name: Option<&str>) -> Self {
        let name = match monitor_name {
            Some(x) => x.to_string(),
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

    fn update_window(
        &mut self,
        win_id: u32,
        icon_name: &str,
        window_info: &WindowInfo,
    ) {
        let mut current_focus = self.current_focus.lock().unwrap();

        if let CurrentFocus::Window(ref mut window) = *current_focus {
            window.info = window_info.to_owned();
            window.id = win_id;
            window.icon.update_name(icon_name);
        } else {
            let mut window = Window {
                icon: IconState::default(),
                info: window_info.to_owned(),
                id: win_id,
            };

            window.icon.update_name(icon_name);

            *current_focus = CurrentFocus::Window(window);
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
    x11rb_connection: Arc<RustConnection>,
    monitor: Monitor,
}

pub trait CoreFeatures<W, C>
where
    W: WMConnection,
    C: Config,
{
    fn init(
        monitor_name: Option<&str>,
        config_option: Option<&str>,
    ) -> Core<W, C>;
    fn update_icon_coords(&mut self);
}

impl CoreFeatures<I3Connection, I3Config> for Core<I3Connection, I3Config> {
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
            x11rb_connection: Arc::new(x11rb_connection),
        }
    }

    fn update_icon_coords(&mut self) {
        let mut current_focus = self.monitor.current_focus.lock().unwrap();

        if let CurrentFocus::Window(ref mut window) = *current_focus {
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

impl Core<BspwmConnection, BspwmConfig> {
    pub fn process_state_change(&mut self, window_id: u32, state: State) {
        // Replace later with normal equality, when State will implement
        // PartialEq
        if let State::Fullscreen = state {
            self.process_fullscreen_window();
        } else {
            self.update_icon(window_id);
        }
    }
}

impl CoreFeatures<BspwmConnection, BspwmConfig>
    for Core<BspwmConnection, BspwmConfig>
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
            x11rb_connection: Arc::new(x11rb_connection),
        }
    }

    fn update_icon_coords(&mut self) {
        let mut current_focus = self.monitor.current_focus.lock().unwrap();

        if let CurrentFocus::Window(ref mut window) = *current_focus {
            window.icon.curr_x = self.config.x();
        }
    }
}

impl<W, C> Core<W, C>
where
    W: WMConnection,
    C: Config,
    Core<W, C>: CoreFeatures<W, C>,
{
    pub fn process_start(&mut self) {
        if let Some(window_id) = self.get_focused_window_id() {
            self.process_focused_window(window_id);
        } else {
            self.process_empty_desktop();
        }

        // Run process for watching for window's info change and
        // printing info to the bar
        self.watch_and_print_info();
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

    fn update_current_focus(&mut self, window_id: Option<u32>) {
        match window_id {
            Some(win_id) => {
                let window_info_types =
                    &self.config.print_info_settings().info_types;

                let window_info =
                    match x11_utils::get_window_info(win_id, window_info_types)
                    {
                        Ok(x) => x,

                        // If this is error, then it is because window was removed
                        // while program was waiting. So we just have to continue
                        // and recognize new focused window on new iteration
                        Err(_) => return,
                    };

                let icon_name = self
                    .wm_connection
                    .get_icon_name(win_id)
                    .unwrap_or(String::new());

                self.monitor.update_window(win_id, &icon_name, &window_info);
            }

            None => {
                let mut current_focus =
                    self.monitor.current_focus.lock().unwrap();
                let empty_info = self
                    .config
                    .print_info_settings()
                    .get_empty_desk_info()
                    .to_string();

                *current_focus =
                    CurrentFocus::EmptyDesk(EmptyInfo { info: empty_info });
            }
        }
    }

    fn print_info(&mut self, window_id: Option<u32>) {
        let old_info = {
            let current_focus = self.monitor.current_focus.lock().unwrap();

            match *current_focus {
                CurrentFocus::EmptyDesk(ref empty_info) => {
                    empty_info.info.to_string()
                }
                CurrentFocus::Window(ref window) => {
                    window.info.info.to_string()
                }
            }
        };

        let mut new_window_info = None;

        let new_info = if let Some(win_id) = window_id {
            match x11_utils::get_window_info(
                win_id,
                &self.config.print_info_settings().info_types,
            ) {
                Ok(x) => {
                    new_window_info = Some(x);

                    &new_window_info.as_ref().unwrap().info
                }

                // If this is error, then it is because window was removed
                // while program was waiting. So we just have to continue
                // and recognize new focused window on new iteration
                Err(_) => return,
            }
        } else {
            self.config.print_info_settings().get_empty_desk_info()
        };

        if old_info != new_info {
            match window_id {
                Some(_) => {
                    let formatter =
                        Some(new_window_info.as_ref().unwrap().info_type);

                    self.config.print_info_util(new_info, formatter);

                    // TODO
                    // Run process for watching for window's info change and
                    // printing info to the bar
                    // self.watch_and_print_info();
                }

                None => {
                    self.config.print_info_util(new_info, None);
                }
            }
        }
    }

    // This function prints info of the window and watches over the info of
    // window and prints it if it changes.
    //
    // If there is no window focused, then it is repeatedly waiting for new
    // window, doing nothing.
    fn watch_and_print_info(&mut self) {
        let current_focus = Arc::clone(&self.monitor.current_focus);
        let gap = self.config.gap().to_string();
        let print_info_settings = self.config.print_info_settings().clone();

        thread::spawn(move || loop {
            {
                let mut current_focus_lock = current_focus.lock().unwrap();

                let window = match *current_focus_lock {
                    CurrentFocus::Window(ref mut x) => x,
                    CurrentFocus::EmptyDesk(_) => {
                        {
                            drop(current_focus_lock);
                        }

                        thread::sleep(Duration::from_millis(100));
                        continue;
                    }
                };

                let window_id = window.id;

                match x11_utils::get_window_info(
                    window_id,
                    &print_info_settings.info_types,
                ) {
                    Ok(new_window_info) => {
                        if window.info.info != new_window_info.info {
                            println!(
                                "{}{}",
                                gap,
                                print_info_settings.format_info(
                                    &new_window_info.info,
                                    Some(new_window_info.info_type)
                                )
                            );

                            window.info = new_window_info;
                        }
                    }

                    // If this is error, then it is because window was removed
                    // while program was waiting. So we just have to continue
                    // and recognize new focused window on new iteration
                    Err(_) => continue,
                };
            }

            thread::sleep(Duration::from_millis(100));
        });
    }

    pub fn process_focused_window(&mut self, window_id: u32) {
        self.print_info(Some(window_id));
        self.update_current_focus(Some(window_id));

        if self.wm_connection.is_window_fullscreen(window_id) {
            self.process_fullscreen_window();
            return;
        }

        self.generate_icon_name(window_id);
        self.monitor.update_fullscreen_status(false);

        if self.is_icon_visible() && self.need_update_icon() {
            self.update_icon(window_id);
        }
    }

    pub fn process_fullscreen_window(&mut self) {
        self.monitor.update_fullscreen_status(true);
        self.destroy_prev_icon();
    }

    pub fn process_empty_desktop(&mut self) {
        self.destroy_prev_icon();
        self.print_info(None);
        self.update_current_focus(None);
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
