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
    // let desktop = core.get_focused_desktop();
    let node = event.container;
    let id = match node.window {
        Some(x) => x,
        None => panic!("No focused window"),
    };

    match event.change {
        WindowChange::Focus => {
            core.process_focused_window(id);
        }

        WindowChange::Close => {
            match core.get_focused_window() {
                Some(id) => {
                    core.process_focused_window(id);
                }

                None => {
                    core.state.reset_icons();
                    core.print_info(None);
                    core.destroy_prev_icons();
                }
            }

            // Some(0);
        }

        WindowChange::FullscreenMode => {
            let current_desktop = core.get_focused_desktop();

            match core.get_fullscreen_window(current_desktop) {
                Some(_) => {
                    println!("Get fullscreen ");
                    core.destroy_prev_icons();

                    // Reset icons, so that we can use process_focused_window
                    // below. Otherwise it will not display icon, since app
                    // name didn't change during fullscreen toggling
                    core.state.reset_icons();
                }

                None => {
                    println!("Exit fullscreen ");

                    let window = core.get_focused_window();
                    if let Some(id) = window {
                        core.process_focused_window(id);
                    }
                }
            }

            // Some(0)
        }

        WindowChange::Move => {
            // Handle fullscreen on new desktop
        }

        _ => {}
    }
}

fn handle_workspace_event(event: WorkspaceEventInfo, core: &mut Core) {}

fn handle_mode_event(event: ModeEventInfo, core: &mut Core) {
    println!("Something happened");
}
