use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;

use super::utils::format_filename;

pub const CONFIG_FILE: &str = $$CONFIG;

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
    pub gap_per_desk: u16,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_config_works() {
        let config = Config::init();

        assert_eq!(config.size, 24);
        assert_eq!(
            config.cache_dir,
            "$HOME/.config/polybar/scripts/ixwindow/polybar-icons"
        );
    }
}
