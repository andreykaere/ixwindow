use i3ipc::reply::{Node, NodeType};
use i3ipc::I3Connection;

use std::process::{Command, Stdio};
use std::str;

use super::config::I3Config;

pub fn is_window_fullscreen(window_id: i32) -> bool {
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

pub fn get_wm_class(window_id: i32) -> String {
    let wm_class = Command::new("xprop")
        .arg("-id")
        .arg(window_id.to_string())
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

pub fn get_icon_name(window_id: i32) -> String {
    get_wm_class(window_id)
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

pub fn get_focused_monitor(conn: &mut I3Connection) -> String {
    let monitors = conn
        .get_outputs()
        .expect("Couldn't read information about tree")
        .outputs;

    for monitor in monitors {
        if monitor.active {
            return monitor.name;
        }
    }

    panic!("No focused monitor was found!");
}

pub fn get_focused_desktop_id(
    conn: &mut I3Connection,
    monitor_name: &str,
) -> Option<i32> {
    let desktops = conn
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

fn get_all_nodes_on_mon(
    conn: &mut I3Connection,
    monitor_name: &str,
) -> Vec<Node> {
    let desktops = get_desks_on_mon(conn, monitor_name);
    let nodes: Vec<_> = desktops.into_iter().flat_map(|x| x.nodes).collect();

    nodes
}

pub fn get_focused_window_id(
    conn: &mut I3Connection,
    monitor_name: &str,
) -> Option<i32> {
    let nodes = get_all_nodes_on_mon(conn, monitor_name);

    for node in nodes {
        if node.focused {
            return node.window;
        }
    }

    // If no window is focused
    None
}

pub fn get_desks_on_mon(
    conn: &mut I3Connection,
    monitor_name: &str,
) -> Vec<Node> {
    let tree = conn
        .get_tree()
        .expect("Couldn't read information about tree");

    let monitors = tree.nodes;

    for monitor in monitors {
        if let Some(x) = monitor.name.as_ref() {
            if x == monitor_name {
                return get_subnodes_type_desk(monitor);
            }
        }
    }

    vec![]
}

// Returns subnodes of the given node, which type is desktop (workspace)
fn get_subnodes_type_desk(node: Node) -> Vec<Node> {
    if let NodeType::Workspace = node.nodetype {
        return vec![node];
    }

    node.nodes
        .into_iter()
        .flat_map(get_subnodes_type_desk)
        .collect()
}

// The output also includes scratchpad desktop
fn get_all_desktops(conn: &mut I3Connection) -> Vec<Node> {
    let tree = conn
        .get_tree()
        .expect("Couldn't read information about tree");

    get_subnodes_type_desk(tree)
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

pub fn calculate_dyn_x(
    conn: &mut I3Connection,
    config: &I3Config,
    monitor_name: &str,
) -> i16 {
    let desks_num = get_desks_on_mon(conn, monitor_name).len();

    ((config.x as f32) + config.gap_per_desk * (desks_num as f32)) as i16
}

#[cfg(test)]
mod tests {
    use super::*;

    use i3ipc::I3Connection;

    fn get_desk_num(desktop: Node) -> Option<i32> {
        if desktop.nodetype != NodeType::Workspace {
            panic!("This is not a desktop");
        }

        if let Some(name) = desktop.name {
            return name.parse::<i32>().ok();
        }

        None
    }

    fn get_focused_desktop(
        conn: &mut I3Connection,
        monitor_name: &str,
    ) -> Node {
        let curr_desk_id = get_focused_desktop_id(conn, monitor_name).unwrap();

        convert_desk_id_to_node(conn, curr_desk_id)
    }

    #[test]
    fn test_tree() {
        let mut connection = I3Connection::connect().unwrap();
        let tree = connection
            .get_tree()
            .expect("Couldn't read information about tree");

        println!("Tree:\n{:#?}", tree);
    }

    #[test]
    fn get_focused_monitor_works() {
        let mut connection = I3Connection::connect().unwrap();

        println!(
            "Focused monitor:\n{:?}",
            get_focused_monitor(&mut connection)
        );
    }

    #[test]
    fn get_desks_on_mon_works() {
        let mut conn = I3Connection::connect().unwrap();

        println!(
            "Focused monitor desktops:\n{:#?}",
            get_desks_on_mon(&mut conn, "eDP-1")
        );
    }

    #[test]
    fn get_all_childs_works() {
        let mut conn = I3Connection::connect().unwrap();
        let tree = conn
            .get_tree()
            .expect("Couldn't read information about tree");

        println!("All windows:\n{:?}", get_all_childs(tree));
    }

    #[test]
    fn get_focused_window_works() {
        let mut conn = I3Connection::connect().unwrap();
        let monitor_name = get_focused_monitor(&mut conn);
        let window = get_focused_window_id(&mut conn, &monitor_name);

        println!("{:?}", window);
    }

    #[test]
    fn get_desk_num_works() {
        let mut conn = I3Connection::connect().unwrap();
        let curr_mon = get_focused_monitor(&mut conn);
        let curr_desk = get_focused_desktop(&mut conn, &curr_mon);
        let desk_num = get_desk_num(curr_desk);

        println!("{desk_num:?}");
    }

    #[test]
    fn get_focused_desktop_id_works() {
        let mut conn = I3Connection::connect().unwrap();
        let curr_mon = get_focused_monitor(&mut conn);
        let curr_desk_id = get_focused_desktop_id(&mut conn, &curr_mon);

        println!("{curr_desk_id:?}");
    }

    #[test]
    fn get_desktop_windows_works() {
        let mut conn = I3Connection::connect().unwrap();
        let curr_mon = get_focused_monitor(&mut conn);
        let desktop = get_focused_desktop_id(&mut conn, &curr_mon).unwrap();
        let result = get_desktop_windows(&mut conn, desktop);

        println!("{:?}", result);
    }

    #[test]
    fn get_all_nodes_on_mon_works() {
        let mut conn = I3Connection::connect().unwrap();
        let curr_mon = get_focused_monitor(&mut conn);
        let nodes = get_all_nodes_on_mon(&mut conn, &curr_mon);

        println!("{:#?}", nodes);
    }
}
