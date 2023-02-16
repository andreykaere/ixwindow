#![allow(dead_code)]
#![allow(unused_variables)]

use i3ipc::event::{
    inner::WindowChange, Event, ModeEventInfo, WindowEventInfo,
    WorkspaceEventInfo,
};

use i3ipc::I3Connection;




pub mod config;
pub mod utils;

pub use config::Config;
pub use utils::*;
// use config::CONFIG;

pub struct Core {
    pub config: Config,
    pub state: State,
    pub connection: I3Connection,
}

impl Core {
    pub fn init() -> Self {
        let connection =
            I3Connection::connect().expect("Failed to connect to i3");

        Self {
            config: Config::init(),
            state: State::default(),
            connection,
        }
    }
}

pub fn handle_event(event: Event, core: &Core) {
    match event {
        Event::WindowEvent(e) => handle_window_event(e, core),
        Event::WorkspaceEvent(e) => handle_workspace_event(e, core),
        Event::ModeEvent(e) => handle_mode_event(e, core),
        _ => {}
    }
}

fn handle_window_event(event: WindowEventInfo, core: &Core) {
    let node = event.container;
    let id = node.id;

    match event.change {
        WindowChange::New => {}
        WindowChange::Close => {}
        WindowChange::Focus => {
            print_info("   ", Some(&node));
        }
        WindowChange::FullscreenMode => {}
        _ => {}
    }
}

fn handle_workspace_event(event: WorkspaceEventInfo, core: &Core) {}

fn handle_mode_event(event: ModeEventInfo, core: &Core) {}
