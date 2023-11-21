use std::path::Path;

use crate::config::BspwmConfig;
use crate::core::{WmCore, WmCoreFeatures as _};
use bspc_rs::events::{self, DesktopEvent, Event, NodeEvent, Subscription};

pub struct BspwmConnection;

impl BspwmConnection {
    pub fn new() -> Self {
        Self
    }
}

pub fn exec(monitor_name: Option<&str>, config_file: Option<&Path>) {
    let mut core = WmCore::init(monitor_name, config_file);
    core.process_start();

    let subscriptions = [
        Subscription::NodeFocus,
        Subscription::NodeRemove,
        Subscription::NodeFlag,
        Subscription::NodeState,
        Subscription::DesktopFocus,
    ];

    let mut subscriber = events::subscribe(false, None, &subscriptions)
        .expect("Couldn't subscribe to events");

    for raw_event in subscriber.events() {
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

impl WmCore<BspwmConnection, BspwmConfig> {
    fn handle_event(&mut self, event: Event) {
        match event {
            Event::NodeEvent(e) => self.handle_node_event(e),
            Event::DesktopEvent(e) => self.handle_desktop_event(e),
            _ => unreachable!(),
        }
    }

    fn handle_node_event(&mut self, event: NodeEvent) {
        match event {
            NodeEvent::NodeFocus(node_info) => {
                self.process_focused_window(node_info.node_id);
            }

            NodeEvent::NodeRemove(_) => {
                let window_id = self.get_focused_window_id();

                if let Some(id) = window_id {
                    self.process_focused_window(id);
                } else {
                    self.process_empty_desktop();
                }
            }

            NodeEvent::NodeFlag(node_info) => {
                // NodeFlag event can in particular mean, that node can become
                // hidden and we need to check if that was the only visible
                // node on that desktop
                if self.is_desk_empty(node_info.desktop_id) {
                    self.process_empty_desktop();
                }
            }

            NodeEvent::NodeState(node_info) => {
                // println!("{:#?}", node_info);
                self.process_focused_window(node_info.node_id);
            }
            _ => {
                unreachable!();
            }
        }
    }

    fn handle_desktop_event(&mut self, event: DesktopEvent) {
        match event {
            DesktopEvent::DesktopFocus(event_info) => {
                let current_desktop = event_info.desktop_id;

                if self.is_desk_empty(current_desktop) {
                    self.process_empty_desktop();
                }

                if self.get_fullscreen_window_id(current_desktop).is_some() {
                    self.process_fullscreen_window();
                }
            }

            _ => {
                unreachable!();
            }
        }
    }
}
