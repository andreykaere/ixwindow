use i3ipc::event::{
    inner::WindowChange, Event, ModeEventInfo, WindowEventInfo,
    WorkspaceEventInfo,
};
use i3ipc::Subscription;
use i3ipc::{self, I3Connection, I3EventListener};

use std::error::Error;
use std::path::Path;

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
    println!("{:?}", event);

    match event {
        Event::WindowEvent(e) => handle_window_event(e),
        Event::WorkspaceEvent(e) => handle_workspace_event(e),
        Event::ModeEvent(e) => handle_mode_event(e),
        _ => {}
    }
}

fn handle_window_event(event: WindowEventInfo) {
    let node = event.container;

    match event.change {
        WindowChange::New => {}
        WindowChange::Close => {}
        WindowChange::Focus => {}
        WindowChange::FullscreenMode => {}
        _ => {}
    }
}

fn handle_workspace_event(event: WorkspaceEventInfo) {}

fn handle_mode_event(event: ModeEventInfo) {}

fn generate_icon() {}

fn display_icon(
    path: &Path,
    cache_dir: &Path,
    size: u8,
    color: &str,
    name: &str,
) {
}

fn print_info(gap: &str, info: &str) {}

fn cleanup_icon() {}

fn parse_config() {}

fn cache_curr_icon() {}
