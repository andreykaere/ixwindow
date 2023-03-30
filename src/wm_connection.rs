use bspc_rs::errors::ReplyError;
use bspc_rs::query;
use bspc_rs::selectors::{DesktopSelector, MonitorSelector, NodeSelector};
use bspc_rs::Id;

use i3ipc::I3Connection;

use std::str;

use crate::bspwm::BspwmConnection;
use crate::{i3_utils, x11_utils};

pub trait WMConnection {
    fn is_window_fullscreen(&mut self, window_id: u32) -> bool {
        // We can't just use unwrap here, because some apps (at least Discord
        // and Zoom) that are changing its window_id as it is running.
        // Probably there are more apps like that
        x11_utils::is_window_fullscreen(window_id).unwrap_or(false)
    }

    fn get_icon_name(&mut self, window_id: u32) -> Option<String> {
        Some(x11_utils::get_wm_class(window_id).ok()?.replace(' ', "-"))
    }

    fn get_focused_desktop_id(&mut self, monitor_name: &str) -> Option<u32>;
    fn is_desk_empty(&mut self, desktop_id: u32) -> bool;
    fn get_focused_window_id(&mut self, monitor_name: &str) -> Option<u32>;
    fn get_fullscreen_window_id(&mut self, desktop_id: u32) -> Option<u32>;
    fn get_desktops_number(&mut self, monitor_name: &str) -> u32;
}

impl WMConnection for I3Connection {
    fn get_focused_desktop_id(&mut self, monitor_name: &str) -> Option<u32> {
        let desktops = self
            .get_workspaces()
            .expect("Couldn't read information about tree")
            .workspaces;

        for desktop in desktops {
            if desktop.focused && monitor_name == desktop.output {
                return Some(desktop.num as u32);
            }
        }

        // Zero desktops on given monitor
        // TODO: check if it is possible on multi monitors setup
        None
    }

    fn is_desk_empty(&mut self, desktop_id: u32) -> bool {
        let node = i3_utils::convert_desk_id_to_node(self, desktop_id as i32);

        node.nodes.is_empty()
    }

    fn get_focused_window_id(&mut self, monitor_name: &str) -> Option<u32> {
        let nodes = i3_utils::get_all_nodes_on_mon(self, monitor_name);

        for node in nodes {
            if node.focused {
                return node.window.map(|x| x as u32);
            }
        }

        // If no window is focused
        None
    }

    fn get_fullscreen_window_id(&mut self, desktop_id: u32) -> Option<u32> {
        let nodes = i3_utils::get_desktop_windows(self, desktop_id as i32);

        for node in nodes {
            if let Some(id) = node.window {
                if self.is_window_fullscreen(id as u32) {
                    return Some(id as u32);
                }
            }
        }

        // If no fullscreen window is found in this desktop
        None
    }

    fn get_desktops_number(&mut self, monitor_name: &str) -> u32 {
        i3_utils::get_desktops_number(self, monitor_name)
    }
}

impl WMConnection for BspwmConnection {
    fn get_focused_desktop_id(&mut self, monitor_name: &str) -> Option<u32> {
        let query_result = query::query_desktops(
            false,
            None,
            Some(MonitorSelector(monitor_name)),
            Some(DesktopSelector("focused")),
            None,
        );

        from_query_result_to_id(query_result)
    }

    fn is_desk_empty(&mut self, desktop_id: u32) -> bool {
        let desk_id = desktop_id.to_string();
        let query_result = query::query_nodes(
            None,
            None,
            Some(DesktopSelector(&desk_id)),
            Some(NodeSelector(".window.!hidden")),
        );

        from_query_result_to_id(query_result).is_none()
    }

    fn get_focused_window_id(&mut self, monitor_name: &str) -> Option<u32> {
        let query_result = query::query_nodes(
            None,
            Some(MonitorSelector(monitor_name)),
            None,
            Some(NodeSelector("focused.window")),
        );

        from_query_result_to_id(query_result)
    }

    fn get_fullscreen_window_id(&mut self, desktop_id: u32) -> Option<u32> {
        let desk_id = desktop_id.to_string();
        let query_result = query::query_nodes(
            None,
            None,
            Some(DesktopSelector(&desk_id)),
            Some(NodeSelector(".fullscreen.window")),
        );

        from_query_result_to_id(query_result)
    }

    fn get_desktops_number(&mut self, monitor_name: &str) -> u32 {
        let query_result = query::query_desktops(
            false,
            None,
            Some(MonitorSelector(monitor_name)),
            None,
            None,
        );

        match query_result {
            Ok(ids) => ids.len() as u32,
            Err(err) => {
                panic!("Query request failed with error {err}");
            }
        }
    }
}

fn from_query_result_to_id(
    query_result: Result<Vec<Id>, ReplyError>,
) -> Option<u32> {
    match query_result {
        Ok(ids) => Some(ids[0]),

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
