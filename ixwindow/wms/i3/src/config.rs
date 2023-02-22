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
    pub config_file: String,
    pub cache_dir: String,
    pub color: String,
    pub gap_per_desk: u16,
    pub monitors: Vec<String>,
}

impl Config {
    pub fn load() -> Config {
        let config_filename = match Config::locate_config_file() {
            Some(config_file) => config_file,
            None => {
                if let Some(config_opt) = Config::process_config_as_option() {
                    config_opt
                } else {
                    panic!("Couldn't find config file");
                }
            }
        };

        let mut config_file =
            File::open(config_filename).expect("Failed to open config file");
        let mut config_str = String::new();
        config_file.read_to_string(&mut config_str).unwrap();
        let mut config: Config = toml::from_str(&config_str).unwrap();

        config.config_file = expand_filename(&config.config_file);
        config.cache_dir = expand_filename(&config.cache_dir);

        config
    }

    pub fn process_config_as_option() -> Option<String> {
        let args = env::args();

        for arg in args {
            let parse: Vec<_> = arg.split("--config=").collect();

            if parse.len() == 2 {
                return Some(parse[1].to_string());
            }
        }

        None
    }

    pub fn locate_config_file() -> Option<String> {
        if let Ok(default_dir) = env::var("XDG_CONFIG_HOME") {
            let default_config =
                format!("{default_dir}/ixwindow/ixwindow.toml");

            if Path::new(&default_config).exists() {
                return Some(default_config);
            }
        }

        if let Ok(specified_config) = env::var("IXWINDOW_CONFIG_PATH") {
            if Path::new(&specified_config).exists() {
                return Some(specified_config);
            }
        }

        None
    }
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

    #[test]
    fn process_config_as_option_works() {
        Config::process_config_as_option();
    }
}
