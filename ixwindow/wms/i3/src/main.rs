use i3ipc::{self, I3EventListener, Subscription};

use ixwindow_i3::{handle_event, Core};

fn main() {
    let mut listener =
        I3EventListener::connect().expect("Couldn't connect to event listener");

    let core = Core::init();

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
                handle_event(res, &core);
            }

            Err(e) => {
                println!("While listening to events, encounter the following error: {e}");
            }
        }
    }
}
