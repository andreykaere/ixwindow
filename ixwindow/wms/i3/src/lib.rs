#![allow(dead_code)]
#![allow(unused_variables)]

use i3ipc::event::{
    inner::{WindowChange, WorkspaceChange},
    Event, ModeEventInfo, WindowEventInfo, WorkspaceEventInfo,
};

use i3ipc::I3Connection;

pub mod config;
pub mod utils;

pub use config::Config;
pub use utils::*;

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
        // Event::ModeEvent(e) => handle_mode_event(e, core),
        _ => {}
    }
}

fn handle_window_event(event: WindowEventInfo, core: &mut Core) {
    let node = event.container;
    let id = match node.window {
        Some(x) => x,
        None => panic!("No focused window"),
    };

    match event.change {
        WindowChange::Focus => {
            if !is_window_fullscreen(id) {
                core.process_focused_window(id);
            }
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

                    core.process_fullscreen_window();
                }

                None => {
                    println!("Exit fullscreen ");

                    let window = core.get_focused_window();
                    if let Some(id) = window {
                        core.process_focused_window(id);
                    }
                }
            }
        }

        WindowChange::Move => {
            // Handle fullscreen on new desktop
        }

        _ => {}
    }
}

fn handle_workspace_event(event: WorkspaceEventInfo, core: &mut Core) {
    // if let Some(current_desktop) = event.current.

    match event.change {
        WorkspaceChange::Focus => {
            let current_desktop = core.get_focused_desktop();

            if core.is_empty(current_desktop) {
                core.process_empty_desktop();
            }

            if let Some(window) = core.get_fullscreen_window(current_desktop) {
                core.process_fullscreen_window();
            }
        }

        _ => {}
    }
}

// fn handle_mode_event(event: ModeEventInfo, core: &mut Core) {}
