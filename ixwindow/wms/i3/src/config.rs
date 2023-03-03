use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;

pub const CONFIG_FILE: &str = "~/.config/ixwindow/i3/config.toml";

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub gap: String,
    pub x: i16,
    pub y: i16,
    pub size: u16,
    pub prefix: String,
    pub config_dir: String,
    pub cache_dir: String,
    pub color: String,
    pub gap_per_desk: u16,
}

impl Config {
    pub fn load() -> Config {
        let mut config_file = File::open(format_filename(CONFIG_FILE))
            .expect("Failed to open config file");
        let mut config_str = String::new();
        config_file.read_to_string(&mut config_str).unwrap();

        let mut config: Config = toml::from_str(&config_str).unwrap();

        config.prefix = format_filename(&config.prefix);
        config.config_dir = format_filename(&config.config_dir);
        config.cache_dir = format_filename(&config.cache_dir);

        config
    }
}

// Returns full path of the filename, extending $HOME and ~ to real home
// directory path
pub fn format_filename(filename: &str) -> String {
    let filename = &shellexpand::env(filename).unwrap();
    let filename = shellexpand::tilde(filename).to_string();

    filename
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_config_works() {
        let config = Config::load();

        assert_eq!(config.size, 24);
        assert_eq!(
            config.cache_dir,
            "/home/andrey/.config/polybar/scripts/ixwindow/polybar-icons"
        );
    }

    #[test]
    fn format_filename_works() {
        let config = Config::load();

        assert_eq!(
            format_filename(&config.cache_dir),
            "/home/andrey/.config/polybar/scripts/ixwindow/polybar-icons"
        );
    }
}
