use i3ipc::reply::{Node, NodeType};
use i3ipc::I3Connection;

use std::str;

pub fn get_desktop_windows(
    conn: &mut I3Connection,
    desktop_id: i32,
) -> Vec<Node> {
    let desktops = get_all_desktops(conn);

    for desk in desktops {
        let desk_name = desk.name.unwrap();

        if desk_name == desktop_id.to_string() {
            return desk.nodes;
        }
    }

    vec![]
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

pub fn get_all_nodes_on_mon(
    conn: &mut I3Connection,
    monitor_name: &str,
) -> Vec<Node> {
    let desktops = get_desks_on_mon(conn, monitor_name);
    let nodes: Vec<_> = desktops.into_iter().flat_map(|x| x.nodes).collect();

    nodes
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
        if let Some(x) = &monitor.name {
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
