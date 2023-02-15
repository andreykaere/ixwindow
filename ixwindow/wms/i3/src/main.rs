use i3ipc::event::{
    inner::WindowChange, Event, ModeEventInfo, WindowEventInfo,
    WorkspaceEventInfo,
};
use i3ipc::reply::{Node, WindowProperty};
use i3ipc::Subscription;
use i3ipc::{self, I3Connection, I3EventListener};

use std::error::Error;
use std::io::{self, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::str;

fn main() {
    let mut listener = I3EventListener::connect().unwrap();

    let subscriptions = [
        Subscription::Workspace,
        Subscription::Mode,
        Subscription::Window,
    ];

    listener.subscribe(&subscriptions);

    for event in listener.listen() {
        match event {
            Ok(res) => {
                handle_event(res);
            }

            Err(e) => {
                println!("While listening to events, encounter the following error: {e}");
            }
        }
    }
}

fn handle_event(event: Event) {
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
    size: u8,
    color: &str,
    name: &str,
) {
}

fn display_icon(
    path: &Path,
    cache_dir: &Path,
    size: u8,
    x: u8,
    y: u8,
    color: &str,
    name: &str,
) {
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    // println!("{:?}", chars);
    chars
        .next()
        .map(|first_letter| first_letter.to_uppercase())
        .into_iter()
        .flatten()
        .chain(chars)
        .collect()
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

fn parse_config(config: &Path, var_name: &str) -> String {
    todo!();
}

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
