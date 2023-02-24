use i3ipc::event::{
    inner::{WindowChange, WorkspaceChange},
    Event, WindowEventInfo, WorkspaceEventInfo,
};

pub mod config;
pub mod core;
pub mod display_icon;
pub mod utils;

pub use self::core::Core;

pub fn handle_event(event: Event, core: &mut Core) {
    match event {
        Event::WindowEvent(e) => handle_window_event(e, core),
        Event::WorkspaceEvent(e) => handle_workspace_event(e, core),
        _ => {}
    }
}

fn handle_window_event(event: WindowEventInfo, core: &mut Core) {
    let node = event.container;
    let id = match node.window {
        Some(x) => x,

        // It means, the window was sent to scratchpad desktop
        None => {
            let window = core.get_focused_window();

            if let Some(x) = window {
                x
            } else {
                core.process_empty_desktop();
                return;
            }
        }
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
                    core.process_empty_desktop();
                }
            }

            // Some(0);
        }

        WindowChange::FullscreenMode => {
            let current_desktop = core.get_focused_desktop();

            match core.get_fullscreen_window(current_desktop) {
                Some(_) => {
                    // println!("Get fullscreen ");

                    core.process_fullscreen_window();
                }

                None => {
                    // println!("Exit fullscreen ");

                    let window = core.get_focused_window();
                    if let Some(id) = window {
                        core.process_focused_window(id);
                    }
                }
            }
        }

        _ => {}
    }
}

fn handle_workspace_event(event: WorkspaceEventInfo, core: &mut Core) {
    match event.change {
        WorkspaceChange::Focus => {
            let current_desktop = core.get_focused_desktop();

            if core.is_empty(current_desktop) {
                core.process_empty_desktop();
            }

            if core.get_fullscreen_window(current_desktop).is_some() {
                core.process_fullscreen_window();
            }
        }

        WorkspaceChange::Init => {
            core.process_empty_desktop();
        }

        _ => {}
    }

    core.update_dyn_x();
}
