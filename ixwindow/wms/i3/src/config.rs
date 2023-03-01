use serde::{Deserialize, Deserializer, Serialize};
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use toml::Value;

#[derive(Serialize, Deserialize, Debug)]
pub struct I3Config {
    pub gap: String,
    pub x: u16,
    pub y: u16,
    pub size: u16,
    pub cache_dir: String,
    pub color: String,
    pub gap_per_desk: u16,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BspwmConfig {
    pub gap: String,
    pub x: u16,
    pub y: u16,
    pub size: u16,
    pub cache_dir: String,
    pub color: String,
    // pub gap_per_desk: u16,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CommonConfig {
    #[serde(rename = "i3wm")]
    #[serde(deserialize_with = "ok_or_default")]
    i3: Option<I3Config>,

    #[serde(deserialize_with = "ok_or_default")]
    bspwm: Option<BspwmConfig>,
}

// We don't want to panic in case one of the config is wrong, because it
// should not be related to another one, since they do not depend on each
// other
fn ok_or_default<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: Deserialize<'de> + Default,
    D: Deserializer<'de>,
{
    let v: Value = Deserialize::deserialize(deserializer).unwrap();
    Ok(T::deserialize(v).unwrap_or_default())
}

impl CommonConfig {
    pub fn init() -> Self {
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
        let common_config: CommonConfig = toml::from_str(&config_str).unwrap();

        common_config
    }

    pub fn load_i3() -> I3Config {
        let common_config = Self::init();
        let mut i3_config = common_config
            .i3
            .expect("While parsing config error occured");
        i3_config.cache_dir = expand_filename(&i3_config.cache_dir);

        i3_config
    }

    pub fn load_bspwm() -> BspwmConfig {
        let common_config = Self::init();
        let mut bspwm_config = common_config
            .bspwm
            .expect("While parsing config error occured");
        bspwm_config.cache_dir = expand_filename(&bspwm_config.cache_dir);

        bspwm_config
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
