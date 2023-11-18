use i3ipc::event::{
    inner::{WindowChange, WorkspaceChange},
    Event, WindowEventInfo, WorkspaceEventInfo,
};

use i3ipc::{self, I3Connection, I3EventListener, Subscription};

use crate::config::I3Config;
use crate::core::{WmCore, WmCoreFeatures as _};

pub fn exec(monitor_name: Option<&str>, config_option: Option<&str>) {
    let mut listener =
        I3EventListener::connect().expect("Couldn't connect to event listener");
    let mut core = WmCore::init(monitor_name, config_option);
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

impl WmCore<I3Connection, I3Config> {
    fn handle_event(&mut self, event: Event) {
        match event {
            Event::WindowEvent(e) => self.handle_window_event(e),
            Event::WorkspaceEvent(e) => self.handle_workspace_event(e),
            _ => {
                unreachable!();
            }
        }
    }

    fn handle_window_event(&mut self, event_info: WindowEventInfo) {
        let node = event_info.container;
        let id = match node.window {
            Some(x) => x as u32,

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

        match event_info.change {
            WindowChange::Focus => {
                self.process_focused_window(id);
            }

            WindowChange::Close => {
                let window_id = self.get_focused_window_id();

                if let Some(id) = window_id {
                    self.process_focused_window(id);
                } else {
                    self.process_empty_desktop();
                }
            }

            WindowChange::FullscreenMode => {
                self.process_focused_window(id);
            }

            _ => {}
        }
    }

    fn handle_workspace_event(&mut self, event_info: WorkspaceEventInfo) {
        match event_info.change {
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

            // TODO: test if this is needed
            WorkspaceChange::Init => {
                self.process_empty_desktop();
            }

            _ => {}
        }
    }
}
