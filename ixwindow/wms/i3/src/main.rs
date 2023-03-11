use std::env;

mod config;
mod core;
mod i3;
mod i3_utils;
mod wm_connection;
mod x11_utils;

fn main() {
    let monitor_name = env::args().nth(1);

    i3::exec(monitor_name);
}
