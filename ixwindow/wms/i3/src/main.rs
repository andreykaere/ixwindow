use i3ipc::{self, I3EventListener, Subscription};

use std::env;

use ixwindow_i3::{handle_event, Core};

fn main() {
    let monitor_name = env::args().nth(1);

    let mut listener =
        I3EventListener::connect().expect("Couldn't connect to event listener");

    let mut core = Core::init(monitor_name);
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
