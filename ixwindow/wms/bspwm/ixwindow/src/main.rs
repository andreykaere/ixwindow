use std::process::{Command, Stdio};

mod config;

use config::Config;

fn main() {
    let config = config::load_bspwm();

    let mut child =
        Command::new(format!("{}/bspwm/ixwindow.sh", config.prefix()))
            .arg(config.prefix())
            .arg(config.cache_dir())
            .arg(config.gap())
            .arg(config.x().to_string())
            .arg(config.y().to_string())
            .arg(config.size().to_string())
            .arg(config.color())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();

    child.wait().unwrap();
}
