use i3ipc::event::{
    inner::{WindowChange, WorkspaceChange},
    Event, WindowEventInfo, WorkspaceEventInfo,
};

use i3ipc::{self, I3Connection, I3EventListener, Subscription};

use super::config::I3Config;
use super::core::{ConfigFeatures as _, Core};
use std::thread;
use std::time::Duration;

pub fn exec(monitor_name: Option<String>) {
    let mut listener =
        I3EventListener::connect().expect("Couldn't connect to event listener");

    let mut core: Core<I3Connection, I3Config> = Core::init(monitor_name);
    // let mut core = Core::init(monitor_name);
    core.process_start();

    let subscriptions = [
        Subscription::Workspace,
        Subscription::Mode,
        Subscription::Window,
    ];

    listener
        .subscribe(&subscriptions)
        .expect("Couldn't subscribe to events");

    for event in listener.listen() {
        match event {
            Ok(res) => {
                handle_event(res, &mut core);
            }

            Err(e) => {
                println!("While listening to events, encounter the following error: {e}");
            }
        }
    }
}

fn handle_event(event: Event, core: &mut Core<I3Connection, I3Config>) {
    match event {
        Event::WindowEvent(e) => handle_window_event(e, core),
        Event::WorkspaceEvent(e) => handle_workspace_event(e, core),
        _ => {}
    }
}

fn handle_window_event(
    event: WindowEventInfo,
    core: &mut Core<I3Connection, I3Config>,
) {
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
        WindowChange::New => thread::sleep(Duration::from_millis(100)),

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

            match core.get_fullscreen_window_id(current_desktop) {
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

fn handle_workspace_event(
    event: WorkspaceEventInfo,
    core: &mut Core<I3Connection, I3Config>,
) {
    match event.change {
        WorkspaceChange::Focus => {
            let current_desktop = match core.get_focused_desktop_id() {
                Some(x) => x,

                // No desktop is focused on the monitor
                None => {
                    return;
                }
            };

            if core.is_desk_empty(current_desktop) {
                core.process_empty_desktop();
            }

            if core.get_fullscreen_window_id(current_desktop).is_some() {
                core.process_fullscreen_window();
            }
        }

        WorkspaceChange::Init => {
            core.process_empty_desktop();
        }

        _ => {}
    }

    core.update_x();
}
