#![allow(dead_code)]
#![allow(unused_variables)]

use i3ipc::event::{
    inner::WindowChange, Event, ModeEventInfo, WindowEventInfo,
    WorkspaceEventInfo,
};
use i3ipc::reply::Node;

use std::io::{self, Write};
use std::path::Path;
use std::process::Command;
use std::str;

pub mod config;

pub fn handle_event(event: Event) {
    // println!("{:?}", event);

    match event {
        Event::WindowEvent(e) => handle_window_event(e),
        Event::WorkspaceEvent(e) => handle_workspace_event(e),
        Event::ModeEvent(e) => handle_mode_event(e),
        _ => {}
    }
}

fn handle_window_event(event: WindowEventInfo) {
    let node = event.container;
    let id = node.id;

    match event.change {
        WindowChange::New => {}
        WindowChange::Close => {}
        WindowChange::Focus => {
            // println!("{}", node.name.unwrap());
            print_info("   ", Some(&node));
        }
        WindowChange::FullscreenMode => {}
        _ => {}
    }
}

fn handle_workspace_event(event: WorkspaceEventInfo) {}

fn handle_mode_event(event: ModeEventInfo) {}

fn generate_icon(
    path: &Path,
    cache_dir: &Path,
    size: u16,
    color: &str,
    name: &str,
) {
}

fn display_icon(
    path: &Path,
    cache_dir: &Path,
    size: u16,
    x: u16,
    y: u16,
    color: &str,
    name: &str,
) {
}

fn capitalize_first(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().chain(c).collect(),
    }
}

fn print_info(gap: &str, window: Option<&Node>) {
    print!("{gap}");

    match window {
        None => print!("Empty"),
        Some(node) => {
            let id = node.window.expect("Couldn't get window id");
            let wm_class = &get_wm_class(id);

            match wm_class.as_ref() {
                "Brave-browser" => print!("Brave"),
                "TelegramDesktop" => print!("Telegram"),
                _ => print!("{}", capitalize_first(wm_class)),
            }
        }
    }

    io::stdout().flush().unwrap();
}

fn cleanup_icon() {}

fn cache_curr_icon() {}

fn remove_prev_icon() {}

fn reset_prev_icon(prev_icon: &Path) {}

fn update_prev_icon(prev_icon: &Path, new_app: &str) {}

fn exists_fullscreen_node() {}

fn get_wm_class(id: i32) -> String {
    let wm_class = Command::new("xprop")
        .arg("-id")
        .arg(id.to_string())
        .arg("WM_CLASS")
        .output()
        .expect("Failed to get WM_CLASS of the window");

    let stdout = match String::from_utf8(wm_class.stdout) {
        Ok(v) => v,
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };

    stdout
        .split(' ')
        .last()
        .expect("WM_CLASS is empty")
        .to_string()
        .trim()
        .replace('"', "")
}
