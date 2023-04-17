use std::fs;
use std::path::Path;
use std::str;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use x11rb::connection::Connection;
use x11rb::protocol::xproto::ConnectionExt;
use x11rb::rust_connection::RustConnection;

use crate::config::Config;
use crate::wm_connection::WMConnection;
use crate::x11_utils;

use super::{Core, CoreFeatures, CurrentFocus};

#[derive(Debug, Default)]
pub struct IconState {
    pub curr_name: Option<String>,
    pub prev_name: Option<String>,
    pub prev_id: Option<u32>,
    pub curr_x: i16,
}

impl IconState {
    pub fn update_name(&mut self, icon_name: &str) {
        self.prev_name = self.curr_name.take();
        self.curr_name = Some(icon_name.to_string());
    }

    pub fn get_curr_name(&self) -> &str {
        self.curr_name
            .as_ref()
            .expect("No icon name, it is set to Empty")
    }
}

impl<W, C> Core<W, C>
where
    W: WMConnection,
    C: Config,
    Core<W, C>: CoreFeatures<W, C>,
{
    fn get_curr_icon_name(current_focus: Arc<Mutex<CurrentFocus>>) -> String {
        let current_focus_lock = current_focus.lock().unwrap();

        if let CurrentFocus::Window(ref window) = *current_focus_lock {
            window.icon.get_curr_name().to_string()
        } else {
            panic!("Don't generate icon for empty desktop");
        }
    }

    pub fn generate_and_display_icon(
        &mut self,
        window_id: u32,
        icon_path: &str,
        initial_icon_name: &str,
    ) {
        let config = &self.config;
        let cache_dir = config.cache_dir().to_string();
        let color = config.color().to_string();
        let icon_path = icon_path.to_string();
        let initial_icon_name = initial_icon_name.to_string();

        if !Path::new(&cache_dir).is_dir() {
            fs::create_dir(&cache_dir)
                .expect("No cache folder was detected and couldn't create it");
        }

        let current_focus = Arc::clone(&self.monitor.current_focus);
        let x11rb_connection = Arc::clone(&self.x11rb_connection);

        let (y, size, monitor_name) =
            (config.y(), config.size(), self.monitor.name.to_string());

        thread::spawn(move || loop {
            let curr_icon_name =
                Self::get_curr_icon_name(Arc::clone(&current_focus));

            // We return, because it means, that another window app is focused
            // and therefore in this case another thread will be started
            if curr_icon_name != initial_icon_name {
                return;
            }

            // println!("try_generate");

            if x11_utils::generate_icon(
                &curr_icon_name,
                &cache_dir,
                &color,
                window_id,
            )
            .is_ok()
            {
                Self::display_icon_util(
                    x11rb_connection,
                    current_focus,
                    &icon_path,
                    y,
                    size,
                    &monitor_name,
                );

                return;
            }

            thread::sleep(Duration::from_millis(100));
        });
    }

    pub fn display_icon(&mut self, icon_path: &str) {
        let config = &self.config;

        let (y, size, monitor_name) =
            (config.y(), config.size(), &self.monitor.name);

        let current_focus = Arc::clone(&self.monitor.current_focus);
        let x11rb_connection = Arc::clone(&self.x11rb_connection);

        Self::display_icon_util(
            x11rb_connection,
            current_focus,
            icon_path,
            y,
            size,
            monitor_name,
        );
    }

    fn display_icon_util(
        x11rb_connection: Arc<RustConnection>,
        current_focus: Arc<Mutex<CurrentFocus>>,
        icon_path: &str,
        y: i16,
        size: u16,
        monitor_name: &str,
    ) {
        let mut current_focus = current_focus.lock().unwrap();

        if let CurrentFocus::Window(ref mut window) = *current_focus {
            let icon_id = x11_utils::display_icon(
                &*x11rb_connection,
                icon_path,
                window.icon.curr_x,
                y,
                size,
                monitor_name,
            )
            .ok();

            window.icon.prev_id = icon_id;
        }
    }

    pub fn is_icon_visible(&mut self) -> bool {
        !self.curr_desk_contains_fullscreen()
    }

    pub fn need_update_icon(&mut self) -> bool {
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

        let current_focus = self.monitor.current_focus.lock().unwrap();

        if let CurrentFocus::Window(ref window) = *current_focus {
            window.icon.prev_name != window.icon.curr_name
        } else {
            false
        }
    }

    pub fn update_icon(&mut self, window_id: u32) {
        // println!("update_icon");
        let config = &self.config;

        let current_focus = &self.monitor.current_focus;
        let icon_name = Self::get_curr_icon_name(Arc::clone(current_focus));

        let icon_path = format!("{}/{}.jpg", &config.cache_dir(), &icon_name);

        // Destroy icon first, before trying to extract it. Fixes the icon
        // still showing, when switching from window that has icon to the
        // widow that doens't.
        self.destroy_prev_icon();

        // Update icon coordinates is needed for any type of displaying icon:
        // either it is before displaying icon right away or trying to
        // retrieve the icon and then displaying
        self.update_icon_coords();

        if Path::new(&icon_path).exists() {
            self.display_icon(&icon_path);
        } else {
            self.generate_and_display_icon(window_id, &icon_path, &icon_name);
        }
    }

    pub fn destroy_prev_icon(&mut self) {
        let conn = &self.x11rb_connection;
        let mut current_focus = self.monitor.current_focus.lock().unwrap();

        if let CurrentFocus::Window(ref mut window) = *current_focus {
            if let Some(id) = window.icon.prev_id {
                conn.destroy_window(id)
                    .expect("Failed to destroy previous icon");
                conn.flush().unwrap();

                window.icon.prev_id = None;
            }
        }
    }

    pub(crate) fn generate_icon_name(&mut self, window_id: u32) {
        let mut timeout_name = 500;
        while timeout_name > 0 {
            // println!("timeout_name");

            if let Some(name) = self.wm_connection.get_icon_name(window_id) {
                if !name.is_empty() {
                    break;
                }
            }

            thread::sleep(Duration::from_millis(100));
            timeout_name -= 100;
        }
    }
}
