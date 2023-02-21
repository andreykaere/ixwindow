use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;

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
    pub monitors: Vec<String>,
}

impl Config {
    pub fn load() -> Config {
        let mut config_file = File::open(Config::locate_config_file())
            .expect("Failed to open config file");

        let mut config_str = String::new();
        config_file.read_to_string(&mut config_str).unwrap();
        let mut config: Config = toml::from_str(&config_str).unwrap();

        config.prefix = expand_filename(&config.prefix);
        config.config_dir = expand_filename(&config.config_dir);
        config.cache_dir = expand_filename(&config.cache_dir);

        config
    }

    fn locate_config_file() -> String {
        todo!();
    }

    // // Generates config from installation profile
    // fn generate_config() -> String {
    //     todo!();
    // }
}

fn expand_filename(filename: &str) -> String {
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
            "$HOME/.config/polybar/scripts/ixwindow/polybar-icons"
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
