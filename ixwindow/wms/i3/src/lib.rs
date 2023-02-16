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

pub fn handle_event(event: Event, core: &mut Core) {
    match event {
        Event::WindowEvent(e) => handle_window_event(e, core),
        Event::WorkspaceEvent(e) => handle_workspace_event(e, core),
        Event::ModeEvent(e) => handle_mode_event(e, core),
        _ => {}
    }
}

fn handle_window_event(event: WindowEventInfo, core: &mut Core) {
    let desktop = core.get_current_desktop();
    let node = event.container;
    let id = match node.window {
        Some(x) => x,
        None => panic!("No focused window"),
    };

    match event.change {
        // WindowChange::New => {
        // println!("new");
        // core.process_window(id);
        // }
        WindowChange::Focus => {
            // println!("Focused");
            core.process_window(id);
        }

        WindowChange::Close => {
            let icon_name = get_icon_name(id);
            core.state.update(&icon_name);
        }

        WindowChange::FullscreenMode => {
            // println!("fullscreen");
        }

        _ => {}
    }
}

fn handle_workspace_event(event: WorkspaceEventInfo, core: &mut Core) {}

fn handle_mode_event(event: ModeEventInfo, core: &mut Core) {}
