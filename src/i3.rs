use i3ipc::event::{
    inner::{WindowChange, WorkspaceChange},
    Event, WindowEventInfo, WorkspaceEventInfo,
};

use i3ipc::{self, I3Connection, I3EventListener, Subscription};

use std::path::Path;

use crate::config::I3Config;
use crate::core::{WmCore, WmCoreFeatures as _};

pub fn exec(monitor_name: Option<&str>, config: Option<&Path>) {
    let mut listener =
        I3EventListener::connect().expect("Couldn't connect to event listener");
    let mut core = WmCore::init(monitor_name, config);
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
                eprintln!("While listening to events, encounter the following error: {e}");
            }
        }
    }
}

impl WmCore<I3Connection, I3Config> {
    fn handle_event(&mut self, event: Event) {
        match event {
            Event::WindowEvent(e) => self.handle_window_event(e),
            Event::WorkspaceEvent(e) => self.handle_workspace_event(e),

            // Prevent panic when switching binding mode or hotplugging outputs
            Event::ModeEvent(_) | Event::OutputEvent(_) => {
                self.handle_general_event();
            }

            err => unreachable!("{:?}", err),
        }
    }

    // In a general case, we want to just update the icon name and location
    fn handle_general_event(&mut self) {
        let window_id = self.get_focused_window_id();
        match window_id {
            Some(id) => self.process_focused_window(id),
            None => self.process_empty_desktop(),
        };
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
