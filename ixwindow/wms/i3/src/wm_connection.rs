use i3ipc::I3Connection;
use std::process::{Command, Stdio};

use super::i3_utils;

pub trait WMConnection {
    fn is_window_fullscreen(&mut self, window_id: i32) -> bool;
    fn get_icon_name(&mut self, window_id: i32) -> String;
    fn get_focused_desktop_id(&mut self, monitor_name: &str) -> Option<i32>;
    fn is_desk_empty(&mut self, desktop_id: i32) -> bool;
    fn get_focused_window_id(&mut self, monitor_name: &str) -> Option<i32>;
    fn get_fullscreen_window_id(&mut self, desktop: i32) -> Option<i32>;
}

impl WMConnection for I3Connection {
    fn is_window_fullscreen(&mut self, window_id: i32) -> bool {
        let net_wm_state = Command::new("xprop")
            .arg("-id")
            .arg(window_id.to_string())
            .arg("_NET_WM_STATE")
            .stderr(Stdio::null())
            .output()
            .expect("Failed to get WM_CLASS of the window");

        let result = match String::from_utf8(net_wm_state.stdout) {
            Ok(v) => v,
            Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
        };

        result.contains("FULLSCREEN")
    }

    fn get_icon_name(&mut self, window_id: i32) -> String {
        i3_utils::get_wm_class(window_id).replace(' ', "-")
    }

    fn get_focused_desktop_id(&mut self, monitor_name: &str) -> Option<i32> {
        let desktops = self
            .get_workspaces()
            .expect("Couldn't read information about tree")
            .workspaces;

        for desktop in desktops {
            if desktop.focused && monitor_name == desktop.output {
                return Some(desktop.num);
            }
        }

        // Zero desktops on given monitor
        // TODO: check if it is possible on multi monitors setup
        None
    }

    fn is_desk_empty(&mut self, desktop_id: i32) -> bool {
        let node = i3_utils::convert_desk_id_to_node(self, desktop_id);

        node.nodes.is_empty()
    }

    fn get_focused_window_id(&mut self, monitor_name: &str) -> Option<i32> {
        let nodes = i3_utils::get_all_nodes_on_mon(self, monitor_name);

        for node in nodes {
            if node.focused {
                return node.window;
            }
        }

        // If no window is focused
        None
    }

    fn get_fullscreen_window_id(&mut self, desktop: i32) -> Option<i32> {
        let nodes = i3_utils::get_desktop_windows(self, desktop);

        for node in nodes {
            if let Some(id) = node.window {
                if self.is_window_fullscreen(id) {
                    return Some(id);
                }
            }
        }

        // If no fullscreen window is found in this desktop
        None
    }
}
