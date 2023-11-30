use clap::Parser;
use std::path::PathBuf;

mod bspwm;
mod config;
mod core;
mod i3;
mod i3_utils;
mod wm_connection;
mod x11_utils;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Opts {
    #[arg(long, short)]
    monitor_name: Option<String>,

    #[arg(long = "config", short)]
    config_path: Option<PathBuf>,
}

fn main() {
    let options = Opts::parse();
    let config_path = options.config_path.as_deref();
    let monitor_name = options.monitor_name.as_deref();

    let wm_name = x11_utils::get_current_wm()
        .expect("Couldn't get current window manager name");

    match wm_name.as_str() {
        "i3" => i3::exec(monitor_name, config_path),
        "bspwm" => bspwm::exec(monitor_name, config_path),
        _ => {}
    }
}
