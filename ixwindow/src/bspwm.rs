use bspc_rs::events::{
    DesktopEvent, DesktopFocusInfo, Event, NodeAddInfo, NodeEvent,
    NodeFocusInfo, NodeRemoveInfo, NodeTransferInfo, Subscription,
};
use bspc_rs::BspwmConnection;
use std::thread;
use std::time::Duration;

use crate::config::BspwmConfig;
use crate::core::{ConfigFeatures as _, Core};

pub fn exec(monitor_name: Option<String>) {
    let mut listener =
        BspwmConnection::connect().expect("Couldn't connect to event listener");
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

    listener
        .subscribe(&subscriptions, false, None)
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
            NodeEvent::NodeAdd(event_info) => {
                thread::sleep(Duration::from_millis(100));
            }

            NodeEvent::NodeFocus(event_info) => {}

            NodeEvent::NodeRemove(event_info) => {}

            NodeEvent::NodeFlag(event_info) => {}

            NodeEvent::NodeState(event_info) => {}

            _ => {
                unreachable!();
            }
        }
    }

    fn handle_desktop_event(&mut self, event: DesktopEvent) {
        todo!();
    }
}
