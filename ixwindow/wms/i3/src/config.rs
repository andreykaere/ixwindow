use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;
// use toml::Value;

#[derive(Serialize, Deserialize, Debug)]
pub struct I3Config {
    pub prefix: String,
    pub gap: String,
    pub x: i16,
    pub y: i16,
    pub size: u16,
    pub cache_dir: String,
    pub color: String,
    pub gap_per_desk: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BspwmConfig {
    pub prefix: String,
    pub gap: String,
    pub x: u16,
    pub y: u16,
    pub size: u16,
    pub cache_dir: String,
    pub color: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Config {
    I3Config(I3Config),
    BspwmConfig(BspwmConfig),
}

impl Config {
    pub fn init() -> toml::Table {
        let config_filename = match Self::locate_config_file() {
            Some(config_file) => config_file,
            None => {
                if let Some(config_opt) = Self::process_config_as_option() {
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

        config_str.parse().unwrap()
    }

    fn load(name: &str) -> Config {
        let mut table = Self::init();

        // We use remove here, because we need ownership for try_into
        let config_table = table.remove(name).unwrap();

        match name {
            "i3" => {
                let mut i3_config: I3Config = config_table.try_into().unwrap();
                i3_config.cache_dir = expand_filename(&i3_config.cache_dir);
                i3_config.prefix = expand_filename(&i3_config.prefix);

                Config::I3Config(i3_config)
            }

            "bspwm" => {
                let mut bspwm_config: BspwmConfig =
                    config_table.try_into().unwrap();
                bspwm_config.cache_dir =
                    expand_filename(&bspwm_config.cache_dir);
                bspwm_config.prefix = expand_filename(&bspwm_config.prefix);

                Config::BspwmConfig(bspwm_config)
            }

            _ => {
                unimplemented!();
            }
        }
    }

    pub fn load_i3() -> Config {
        Self::load("i3")
    }

    pub fn load_bspwm() -> Config {
        Self::load("bspwm")
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

pub fn expand_filename(filename: &str) -> String {
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
        let config = Config::load("i3");

        if let Config::I3Config(conf) = config {
            assert_eq!(conf.size, 24);
            assert_eq!(
                conf.cache_dir,
                "/home/andrey/.config/polybar/scripts/ixwindow/polybar-icons"
            );
        }
    }

    #[test]
    fn expand_filename_works() {
        let config = Config::load_i3();

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
