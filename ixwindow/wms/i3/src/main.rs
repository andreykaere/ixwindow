use ixwindow_i3::i3;
use std::env;

fn main() {
    let monitor_name = env::args().nth(1);

    i3::exec(monitor_name);
}
