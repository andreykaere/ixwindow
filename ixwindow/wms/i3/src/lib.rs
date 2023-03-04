use i3ipc::event::{
    inner::{WindowChange, WorkspaceChange},
    Event, WindowEventInfo, WorkspaceEventInfo,
};

pub mod config;
pub mod core;
pub mod i3_utils;
pub mod x11_utils;

pub use self::core::Core;
use self::i3_utils as i3;

use std::thread;
use std::time::Duration;

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
            let window = core.get_focused_window_id();

            if let Some(x) = window {
                x
            } else {
                core.process_empty_desktop();
                return;
            }
        }
    };

    match event.change {
        WindowChange::New => {
            thread::sleep(Duration::from_millis(100));
        }
        WindowChange::Focus => {
            core.process_focused_window(id);
        }

        WindowChange::Close => {
            if core.is_curr_desk_empty() {
                core.process_empty_desktop();
            }
        }

        WindowChange::FullscreenMode => {
            // We can use unwrap, because some desktop should be focused
            let current_desktop = core.get_focused_desktop_id().unwrap();

            match i3::get_fullscreen_window(
                &mut core.connection,
                current_desktop,
            ) {
                Some(_) => {
                    core.process_fullscreen_window();
                }

                None => {
                    let window = core.get_focused_window_id();
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
            let current_desktop = match core.get_focused_desktop_id() {
                Some(x) => x,

                // No desktop is focused on the monitor
                None => {
                    return;
                }
            };

            if i3::is_desk_empty(&mut core.connection, current_desktop) {
                core.process_empty_desktop();
            }

            if i3::get_fullscreen_window(&mut core.connection, current_desktop)
                .is_some()
            {
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
