use i3ipc::event::Event;
use i3ipc::Subscription;
use i3ipc::{self, I3Connection, I3EventListener};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // establish a connection to i3 over a unix socket
    // let mut connection = I3Connection::connect().unwrap();

    let mut listener = I3EventListener::connect()?;

    let subscriptions = [
        Subscription::Workspace,
        Subscription::Mode,
        Subscription::Window,
    ];

    listener.subscribe(&subscriptions);

    for event in listener.listen() {
        // let y: i32 = event;

        match event {
            Ok(res) => {
                handle_event(res);
            }

            Err(e) => {
                println!("Encounter the following error: {e}");
            }
        }
    }

    Ok(())
}

fn handle_event(event: Event) {
    println!("{:?}", event);

    match event {
        Event::WindowEvent(e) => handle_window_event(e),
        Event::WorkspaceEvent(e) => handle_workspace_event(e),
        Event::ModeEvent(e) => handle_mode_event(e),
    }
}

fn handle_window_event(event: WindowEventInfo) {
    let node = event.node;

    match event.change {
        WindowChange::New => {}
        WindowChange::Close => {}
        WindowChange::Focus => {}
        WindowChange::FullscreenMode => {}
    }
}

fn handle_workspace_event(event: WorkspaceEventInfo) {}

fn handle_mode_event(event: ModeEventInfo) {}



fn generate_icon() {
}

fn display_icon(path: , cache_dir: , size: , color: , name: ) {
}

fn print_info(gap: , info: &str) {
}

fn cleanup_icon() {
}

fn parse_config() {
}


fn cache_curr_icon() {
}
