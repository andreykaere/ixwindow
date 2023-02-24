use i3ipc::reply::{Node, NodeType};
use i3ipc::I3Connection;

use std::process::{Command, Stdio};
use std::str;

pub fn is_window_fullscreen(window: i32) -> bool {
    let net_wm_state = Command::new("xprop")
        .arg("-id")
        .arg(window.to_string())
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

pub fn get_wm_class(window: i32) -> String {
    let wm_class = Command::new("xprop")
        .arg("-id")
        .arg(window.to_string())
        .arg("WM_CLASS")
        .stderr(Stdio::null())
        .output()
        .expect("Failed to get WM_CLASS of the window");

    let result = match String::from_utf8(wm_class.stdout) {
        Ok(v) => v,
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };

    result
        .split(' ')
        .last()
        .expect("WM_CLASS is empty")
        .to_string()
        .trim()
        .replace('"', "")
}

pub fn get_icon_name(window: i32) -> String {
    get_wm_class(window)
}

// Returns all childs of the node, that themselves do not contain any windows,
// mostly it's just windows
pub fn get_all_childs(node: Node) -> Vec<Node> {
    if node.nodes.is_empty() {
        return vec![node];
    }

    node.nodes.into_iter().flat_map(get_all_childs).collect()
}

fn get_desktop_windows(conn: &mut I3Connection, desktop: i32) -> Vec<Node> {
    let desktops = get_all_desktops(conn);

    for desk in desktops {
        let desk_name = desk.name.unwrap();

        if desk_name == desktop.to_string() {
            return desk.nodes;
        }
    }

    vec![]
}

fn get_desk_num(desktop: Node) -> Option<i32> {
    if desktop.nodetype != NodeType::Workspace {
        panic!("This is not a desktop");
    }

    todo!();
}

pub fn get_focused_monitor(conn: &mut I3Connection) -> String {
    todo!();
}

pub fn get_focused_desktop_id(
    conn: &mut I3Connection,
    monitor: &str,
) -> Option<i32> {
    let desktops = get_desks_on_mon(conn, monitor);

    for desktop in desktops {
        if desktop.focused {
            return get_desk_num(desktop);
        }
    }

    // Zero monitors on given monitor
    // TODO: check if it is possible
    None
}

pub fn convert_desk_id_to_node(
    conn: &mut I3Connection,
    desktop_id: i32,
) -> Node {
    let desktops = get_all_desktops(conn);

    for desk in desktops {
        if desk.name == Some(desktop_id.to_string()) {
            return desk;
        }
    }

    panic!("Something went wrong, when converting desktop to node");
}

pub fn is_desk_empty(conn: &mut I3Connection, desktop_id: i32) -> bool {
    let node = convert_desk_id_to_node(conn, desktop_id);

    node.nodes.is_empty()
}

pub fn get_all_nodes(conn: &mut I3Connection) -> Vec<Node> {
    let tree = conn
        .get_tree()
        .expect("Couldn't read information about tree");

    get_all_childs(tree)
}

pub fn get_focused_window_id(conn: &mut I3Connection) -> Option<i32> {
    let nodes = get_all_nodes(conn);

    for node in nodes {
        if node.focused {
            return node.window;
        }
    }

    // If no window is focused
    None
}

pub fn get_desks_on_mon(conn: &mut I3Connection, monitor: &str) -> Vec<Node> {
    todo!();
}

// Returns subnudes, that are desktops
fn get_desktop_subnodes(node: Node) -> Vec<Node> {
    if let NodeType::Workspace = node.nodetype {
        return vec![node];
    }

    node.nodes
        .into_iter()
        .flat_map(get_desktop_subnodes)
        .collect()
}

fn get_desktops_as_nodes(conn: &mut I3Connection, monitor: &str) -> Vec<Node> {
    let tree = conn
        .get_tree()
        .expect("Couldn't read information about tree");

    todo!();
}

fn get_all_desktops(conn: &mut I3Connection) -> Vec<Node> {
    let tree = conn
        .get_tree()
        .expect("Couldn't read information about tree");

    get_desktop_subnodes(tree)
}

pub fn get_fullscreen_window(
    conn: &mut I3Connection,
    desktop: i32,
) -> Option<i32> {
    let nodes = get_desktop_windows(conn, desktop);

    for node in nodes {
        if let Some(id) = node.window {
            if is_window_fullscreen(id) {
                return Some(id);
            }
        }
    }

    // If no fullscreen window is found in this desktop
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Config;
    use i3ipc::I3Connection;

    #[test]
    fn test_detecting_monitors() {
        let mut connection = I3Connection::connect().unwrap();
        let tree = connection
            .get_tree()
            .expect("Couldn't read information about tree");

        println!("Tree:\n{:?}", tree);
    }

    #[test]
    fn get_all_childs_works() {
        let mut connection = I3Connection::connect().unwrap();
        let tree = connection
            .get_tree()
            .expect("Couldn't read information about tree");

        println!("All windows:\n{:?}", get_all_childs(tree));
    }

    #[test]
    fn get_focused_window_works() {
        let mut core = Core::init();
        let window = core.get_focused_window_id();

        println!("{:?}", window);
    }

    #[test]
    fn get_desktop_windows_works() {
        let mut core = Core::init();
        let desktop = core.get_focused_desktop_id();
        let result = core.get_desktop_windows(desktop);

        println!("{:?}", result);
    }
}
