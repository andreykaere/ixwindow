use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;

pub const CONFIG_FILE: &str = "~/.config/ixwindow/bspwm/config.toml";
// pub const CONFIG: Config = Config::load();

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub gap: String,
    pub x: u16,
    pub y: u16,
    pub size: u16,
    pub prefix: String,
    pub config_dir: String,
    pub cache_dir: String,
    pub color: String,
}

impl Config {
    pub fn init() -> Config {
        let mut config_file = File::open(format_filename(CONFIG_FILE))
            .expect("Failed to open config file");
        let mut config_str = String::new();
        config_file.read_to_string(&mut config_str).unwrap();

        let config: Config = toml::from_str(&config_str).unwrap();

        config
    }
}

pub fn format_filename(filename: &str) -> String {
    let home = std::env::var("HOME").unwrap();
    let filename = &shellexpand::env(filename).unwrap();
    let filename = shellexpand::tilde(filename).to_string();

    filename
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn parse_config_works() {
        let config = Config::init();

        assert_eq!(config.size, 24);
        assert_eq!(config.x, 270);
        assert_eq!(
            config.cache_dir,
            "$HOME/.config/polybar/scripts/ixwindow/polybar-icons"
        );
    }
}
