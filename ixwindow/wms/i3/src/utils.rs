use i3ipc::reply::{Node, NodeType};

use std::process::{Command, Stdio};
use std::str;

// Capitalizes first letter of the string, i.e. converts foo to Foo
pub fn capitalize_first(s: &str) -> String {
    let mut c = s.chars();

    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().chain(c).collect(),
    }
}

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

// Returns visible desktops as nodes
pub fn get_desktop_subnodes(node: Node) -> Vec<Node> {
    if let NodeType::Workspace = node.nodetype {
        return vec![node];
    }

    node.nodes
        .into_iter()
        .flat_map(get_desktop_subnodes)
        .collect()
}

// Returns all childs of the node, that themselves do not contain any windows,
// mostly it's just windows
pub fn get_all_childs(node: Node) -> Vec<Node> {
    if node.nodes.is_empty() {
        return vec![node];
    }

    node.nodes.into_iter().flat_map(get_all_childs).collect()
}
