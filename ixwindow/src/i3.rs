use i3ipc::event::{
    inner::{WindowChange, WorkspaceChange},
    Event, WindowEventInfo, WorkspaceEventInfo,
};

use i3ipc::{self, I3Connection, I3EventListener, Subscription};

use crate::config::I3Config;
use crate::core::{ConfigFeatures as _, Core};
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

    for raw_event in listener.listen() {
        match raw_event {
            Ok(event) => {
                core.handle_event(event);
            }

            Err(e) => {
                println!("While listening to events, encounter the following error: {e}");
            }
        }
    }
}

impl Core<I3Connection, I3Config> {
    fn handle_event(&mut self, event: Event) {
        match event {
            Event::WindowEvent(e) => self.handle_window_event(e),
            Event::WorkspaceEvent(e) => self.handle_workspace_event(e),
            _ => {}
        }
    }

    fn handle_window_event(&mut self, event: WindowEventInfo) {
        let node = event.container;
        let id = match node.window {
            Some(x) => x,

            // It means, the window was sent to scratchpad desktop
            None => {
                let window = self.get_focused_window_id();

                if let Some(x) = window {
                    x
                } else {
                    self.process_empty_desktop();
                    return;
                }
            }
        };

        match event.change {
            WindowChange::New => thread::sleep(Duration::from_millis(100)),

            WindowChange::Focus => {
                self.process_focused_window(id);
            }

            WindowChange::Close => {
                if self.is_curr_desk_empty() {
                    self.process_empty_desktop();
                }
            }

            WindowChange::FullscreenMode => {
                // We can use unwrap, because some desktop should be focused
                let current_desktop = self.get_focused_desktop_id().unwrap();

                match self.get_fullscreen_window_id(current_desktop) {
                    Some(_) => {
                        self.process_fullscreen_window();
                    }

                    None => {
                        let window = self.get_focused_window_id();
                        if let Some(id) = window {
                            self.process_focused_window(id);
                        }
                    }
                }
            }

            _ => {}
        }
    }

    fn handle_workspace_event(&mut self, event: WorkspaceEventInfo) {
        match event.change {
            WorkspaceChange::Focus => {
                let current_desktop = match self.get_focused_desktop_id() {
                    Some(x) => x,

                    // No desktop is focused on the monitor
                    None => {
                        return;
                    }
                };

                if self.is_desk_empty(current_desktop) {
                    self.process_empty_desktop();
                }

                if self.get_fullscreen_window_id(current_desktop).is_some() {
                    self.process_fullscreen_window();
                }
            }

            WorkspaceChange::Init => {
                self.process_empty_desktop();
            }

            _ => {}
        }

        self.update_x();
    }
}
