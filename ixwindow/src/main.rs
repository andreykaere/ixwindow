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

    match get_current_wm().as_ref() {
        "i3wm" => i3::exec(monitor_name),
        "bspwm" => bspwm::exec(monitor_name),
        _ => {}
    }
}

fn get_current_wm() -> String {
    todo!();
}
