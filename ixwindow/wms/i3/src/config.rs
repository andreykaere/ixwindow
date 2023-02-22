use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;

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

    pub fn locate_config_file() -> String {
        if let Ok(default_dir) = env::var("XDG_CONFIG_HOME") {
            let default_config =
                format!("{}/ixwindow/ixwindow.toml", default_dir);

            if Path::new(&default_config).exists() {
                return default_config;
            }
        } else {
            println!("Environmental variable $XDG_CONFIG_HOME is not set");
        }

        if let Ok(specified_config) = env::var("IXWINDOW_CONFIG_PATH") {
            if Path::new(&specified_config).exists() {
                return specified_config;
            }
        }

        panic!("Couldn't find config file");
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
    fn locate_config_file_works() {
        Config::locate_config_file();
    }

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
    fn expand_filename_works() {
        let config = Config::load();

        assert_eq!(
            expand_filename(&config.cache_dir),
            "/home/andrey/.config/polybar/scripts/ixwindow/polybar-icons"
        );
    }
}
