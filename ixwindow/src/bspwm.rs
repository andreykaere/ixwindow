use bspc_rs::events::{
    DesktopEvent, DesktopFocusInfo, Event, NodeAddInfo, NodeEvent,
    NodeFocusInfo, NodeRemoveInfo, NodeTransferInfo, Subscription,
};

use bspc_rs::properties::State;
use bspc_rs::Bspc;
use std::thread;
use std::time::Duration;

use crate::config::BspwmConfig;
use crate::core::{ConfigFeatures as _, Core};

pub struct BspwmConnection;

impl BspwmConnection {
    pub fn new() -> Self {
        Self
    }
}

pub fn exec(monitor_name: Option<String>) {
    let mut conn = BspwmConnection::new();
    let mut core = Core::init(monitor_name);
    core.process_start();

    let subscriptions = [
        Subscription::NodeAdd,
        Subscription::NodeFocus,
        Subscription::NodeRemove,
        Subscription::NodeFlag,
        Subscription::NodeState,
        Subscription::DesktopFocus,
    ];

    let subscriber = Bspc::subscribe(&subscriptions, false, None)
        .expect("Couldn't subscribe to events");

    for raw_event in subscriber {
        // println!("{raw_event:#?}");

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

impl Core<BspwmConnection, BspwmConfig> {
    fn handle_event(&mut self, event: Event) {
        match event {
            Event::NodeEvent(e) => self.handle_node_event(e),
            Event::DesktopEvent(e) => self.handle_desktop_event(e),
            _ => {
                unreachable!();
            }
        }
    }

    fn handle_node_event(&mut self, event: NodeEvent) {
        match event {
            NodeEvent::NodeAdd(node_info) => {
                thread::sleep(Duration::from_millis(100));
            }
            NodeEvent::NodeFocus(node_info) => {
                self.process_focused_window(
                    node_info.node_id.try_into().unwrap(),
                );
            }

            NodeEvent::NodeRemove(node_info) => {
                if self.is_desk_empty(node_info.desktop_id.try_into().unwrap())
                {
                    self.process_empty_desktop();
                }
            }

            NodeEvent::NodeFlag(node_info) => {
                // NodeFlag event can in particular mean, that node can become
                // hidden and we need to check if that was the only visible
                // node on that desktop
                if self.is_desk_empty(node_info.desktop_id.try_into().unwrap())
                {
                    self.process_empty_desktop();
                }
            }

            NodeEvent::NodeState(node_info) => {
                self.process_focused_window(
                    node_info.node_id.try_into().unwrap(),
                );
            }
            _ => {
                unreachable!();
            }
        }
    }

    fn handle_desktop_event(&mut self, event: DesktopEvent) {
        match event {
            DesktopEvent::DesktopFocus(event_info) => {
                let current_desktop = event_info.desktop_id.try_into().unwrap();

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
