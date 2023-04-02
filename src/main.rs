use std::env;

mod bspwm;
mod config;
mod core;
mod i3;
mod i3_utils;
mod wm_connection;
mod x11_utils;

struct IxwindowOptions {
    monitor_name: Option<String>,
    config_option: Option<String>,
}

impl IxwindowOptions {
    fn init() -> Self {
        let mut args = env::args();
        args.next();

        let mut monitor_name = None;
        let mut config_option = None;

        for arg in args {
            if arg.contains("--config=") {
                config_option = arg.split(' ').nth(1).map(|x| x.to_string());
            } else {
                monitor_name = Some(arg);
            }
        }

        Self {
            monitor_name,
            config_option,
        }
    }
}

fn main() {
    let options = IxwindowOptions::init();
    let config_option = options.config_option.as_deref();
    let monitor_name = options.monitor_name.as_deref();

    let wm_name = x11_utils::get_current_wm()
        .expect("Couldn't get current window manager name");

    match wm_name.as_str() {
        "i3" => i3::exec(monitor_name, config_option),
        "bspwm" => bspwm::exec(monitor_name, config_option),
        _ => {}
    }
}
