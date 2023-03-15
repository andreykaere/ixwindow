use bspc_rs::errors::ReplyError;
use bspc_rs::{Bspc, Id};

use i3ipc::I3Connection;

use std::process::{Command, Stdio};
use std::str;

use crate::bspwm::BspwmConnection;
use crate::{i3_utils, x11_utils};

pub trait WMConnection {
    // fn is_window_fullscreen(foo: Option<&mut Self>, window_id: i32) -> bool;
    fn is_window_fullscreen(&mut self, window_id: i32) -> bool;
    fn get_icon_name(&mut self, window_id: i32) -> String;
    fn get_focused_desktop_id(&mut self, monitor_name: &str) -> Option<i32>;
    fn is_desk_empty(&mut self, desktop_id: i32) -> bool;
    fn get_focused_window_id(&mut self, monitor_name: &str) -> Option<i32>;
    fn get_fullscreen_window_id(&mut self, desktop_id: i32) -> Option<i32>;
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
        x11_utils::get_wm_class(window_id)
            .unwrap()
            .replace(' ', "-")
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

    fn get_fullscreen_window_id(&mut self, desktop_id: i32) -> Option<i32> {
        let nodes = i3_utils::get_desktop_windows(self, desktop_id);

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

impl WMConnection for BspwmConnection {
    fn is_window_fullscreen(&mut self, window_id: i32) -> bool {
        let node_request = format!("{window_id}.fullscreen.window");
        let query_result =
            Bspc::query_nodes(None, None, None, Some(&node_request));

        from_query_result_to_id(query_result).is_some()
    }

    fn get_icon_name(&mut self, window_id: i32) -> String {
        x11_utils::get_wm_class(window_id)
            .unwrap()
            .replace(' ', "-")
        // let node = Self::from_id_to_node(window_id.try_into().unwrap())
        //     .unwrap()
        //     .unwrap();

        // if let Some(client) = node.client {
        //     client.class_name
        // } else {
        //     panic!("This node is not a window!");
        // }
    }

    fn get_focused_desktop_id(&mut self, monitor_name: &str) -> Option<i32> {
        let query_result = Bspc::query_desktops(
            false,
            None,
            Some(monitor_name),
            Some("focused"),
            None,
        );

        from_query_result_to_id(query_result)
    }

    fn is_desk_empty(&mut self, desktop_id: i32) -> bool {
        let desk_id = desktop_id.to_string();
        let query_result = Bspc::query_nodes(None, None, Some(&desk_id), None);

        from_query_result_to_id(query_result).is_none()
    }

    fn get_focused_window_id(&mut self, monitor_name: &str) -> Option<i32> {
        let query_result = Bspc::query_nodes(
            None,
            Some(monitor_name),
            None,
            Some("focused.window"),
        );

        from_query_result_to_id(query_result)
    }

    fn get_fullscreen_window_id(&mut self, desktop_id: i32) -> Option<i32> {
        let desk_id = desktop_id.to_string();
        let query_result = Bspc::query_nodes(
            None,
            None,
            Some(&desk_id),
            Some(".fullscreen.window"),
        );

        from_query_result_to_id(query_result)
    }
}

fn from_query_result_to_id(
    query_result: Result<Vec<Id>, ReplyError>,
) -> Option<i32> {
    match query_result {
        Ok(ids) => Some(ids[0].try_into().unwrap()),

        Err(ReplyError::RequestFailed(err)) => {
            if err.is_empty() {
                None
            } else {
                panic!("Query request failed with error {err}");
            }
        }

        Err(err) => {
            panic!("Query request failed with error {err}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[ignore]
    // #[test]
    // fn is_desk_empty_bspwm() {
    //     // let mut conn = BspwmConnection::connect().unwrap();
    //     let mut conn = BspwmConnection::connect().unwrap();
    //     conn.send_message("");

    //     let desktop_id = BspwmConnection::query_desktops(
    //         false,
    //         None,
    //         None,
    //         Some("focused"),
    //         None,
    //     )
    //     .unwrap()[0];
    //     println!("foo");
    //     // let res = conn.is_desk_empty(desktop_id.try_into().unwrap());

    //     // assert_eq!(res, false);
    // }

    #[test]
    fn test_get_focused_desktop_id() {
        let mut conn = BspwmConnection::new();

        println!("{:#?}", conn.get_focused_desktop_id("eDP-1"));
    }
}
