#![allow(clippy::enum_variant_names)]
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct CommonConfig {
    pub gap: String,
    pub x: i16,
    pub y: i16,
    pub size: u16,
    pub cache_dir: String,
    pub color: String,

    #[serde(default)]
    pub print_info: PrintInfo,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PrintInfo {
    #[default]
    WmInstance,

    WmClass,
    WmName,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct I3Config {
    #[serde(flatten)]
    pub common_config: CommonConfig,

    pub gap_per_desk: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BspwmConfig {
    #[serde(flatten)]
    pub common_config: CommonConfig,
}

pub trait Config {
    fn common_config(&self) -> &CommonConfig;

    fn gap(&self) -> &str {
        &self.common_config().gap
    }

    fn color(&self) -> &str {
        &self.common_config().color
    }

    fn cache_dir(&self) -> &str {
        &self.common_config().cache_dir
    }

    fn x(&self) -> i16 {
        self.common_config().x
    }

    fn y(&self) -> i16 {
        self.common_config().y
    }

    fn size(&self) -> u16 {
        self.common_config().size
    }

    fn print_info(&self) -> PrintInfo {
        self.common_config().print_info
    }
}

impl Config for I3Config {
    fn common_config(&self) -> &CommonConfig {
        &self.common_config
    }
}

impl Config for BspwmConfig {
    fn common_config(&self) -> &CommonConfig {
        &self.common_config
    }
}

pub fn read_to_table(config_option: Option<&str>) -> toml::Table {
    let config_filename = if let Some(name) = config_option {
        name.to_string()
    } else {
        locate_config_file().expect("Couldn't find config file")
    };

    let config_filename = expand_filename(&config_filename);

    let mut config_file =
        File::open(config_filename).expect("Failed to open config file");
    let mut config_str = String::new();
    config_file.read_to_string(&mut config_str).unwrap();

    config_str.parse().unwrap()
}

pub fn load_i3(config_option: Option<&str>) -> I3Config {
    let mut table = read_to_table(config_option);

    // We use remove here, because we need ownership for try_into
    let config_table = table.remove("i3").unwrap();

    let mut i3_config: I3Config = config_table.try_into().unwrap();
    i3_config.common_config.cache_dir =
        expand_filename(&i3_config.common_config.cache_dir);

    i3_config
}

pub fn load_bspwm(config_option: Option<&str>) -> BspwmConfig {
    let mut table = read_to_table(config_option);

    // We use remove here, because we need ownership for try_into
    let config_table = table.remove("bspwm").unwrap();

    let mut bspwm_config: BspwmConfig = config_table.try_into().unwrap();
    bspwm_config.common_config.cache_dir =
        expand_filename(&bspwm_config.common_config.cache_dir);

    bspwm_config
}

fn locate_config_file() -> Option<String> {
    if let Ok(default_dir) = env::var("XDG_CONFIG_HOME") {
        let default_config = format!("{default_dir}/ixwindow/ixwindow.toml");

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
        locate_config_file();
    }

    #[test]
    fn parse_config_works() {
        let config = load_i3(None);

        assert_eq!(config.size(), 24);
        assert_eq!(config.print_info(), PrintInfo::WmInstance);
        assert_eq!(
            config.cache_dir(),
            "/home/andrey/.config/polybar/scripts/ixwindow/polybar-icons"
        );
    }

    #[test]
    fn expand_filename_works() {
        let config = load_i3(None);

        assert_eq!(
            expand_filename(config.cache_dir()),
            "/home/andrey/.config/polybar/scripts/ixwindow/polybar-icons"
        );
    }
}
