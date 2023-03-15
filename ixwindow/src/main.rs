use std::env;

mod bspwm;
mod config;
mod core;
mod i3;
mod i3_utils;
mod wm_connection;
mod x11_utils;

fn main() {
    let monitor_name = env::args().nth(1);
    let wm_name = x11_utils::get_current_wm()
        .expect("Couldn't get current window manager name");

    match wm_name.as_ref() {
        "i3wm" => i3::exec(monitor_name),
        "bspwm" => bspwm::exec(monitor_name),
        _ => {}
    }
}
