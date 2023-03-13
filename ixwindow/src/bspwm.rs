use crate::config::BspwmConfig;
use crate::core::{ConfigFeatures as _, Core};
use bspc_rs::{BspwmConnection, Event, Subscription};

pub fn exec(monitor_name: Option<String>) {
    let mut listener =
        BspwmConnection::connect().expect("Couldn't connect to event listener");

    let mut core: Core<BspwmConnection, BspwmConfig> = Core::init(monitor_name);
    core.process_start();

    let subscriptions = [Subscription::NodeAdd, Subscription::NodeFocus];

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
        todo!();
    }
}
