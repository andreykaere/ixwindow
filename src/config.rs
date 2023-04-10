use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct EssentialConfig {
    pub gap: String,
    pub x: i16,
    pub y: i16,
    pub size: u16,
    pub cache_dir: String,
    pub color: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct I3Config {
    #[serde(flatten)]
    pub essential_config: EssentialConfig,

    pub gap_per_desk: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BspwmConfig {
    #[serde(flatten)]
    pub essential_config: EssentialConfig,
}

pub trait Config {
    fn essential_config(&self) -> &EssentialConfig;

    fn gap(&self) -> &str {
        &self.essential_config().gap
    }

    fn color(&self) -> &str {
        &self.essential_config().color
    }

    fn cache_dir(&self) -> &str {
        &self.essential_config().cache_dir
    }

    fn x(&self) -> i16 {
        self.essential_config().x
    }

    fn y(&self) -> i16 {
        self.essential_config().y
    }

    fn size(&self) -> u16 {
        self.essential_config().size
    }
}

impl Config for I3Config {
    fn essential_config(&self) -> &EssentialConfig {
        &self.essential_config
    }
}

impl Config for BspwmConfig {
    fn essential_config(&self) -> &EssentialConfig {
        &self.essential_config
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
    i3_config.essential_config.cache_dir =
        expand_filename(&i3_config.essential_config.cache_dir);

    i3_config
}

pub fn load_bspwm(config_option: Option<&str>) -> BspwmConfig {
    let mut table = read_to_table(config_option);

    // We use remove here, because we need ownership for try_into
    let config_table = table.remove("bspwm").unwrap();

    let mut bspwm_config: BspwmConfig = config_table.try_into().unwrap();
    bspwm_config.essential_config.cache_dir =
        expand_filename(&bspwm_config.essential_config.cache_dir);

    bspwm_config
}

fn locate_config_file() -> Option<String> {
    if let Ok(specified_config) = env::var("IXWINDOW_CONFIG_PATH") {
        if Path::new(&specified_config).exists() {
            return Some(specified_config);
        }
    }

    // default_dir is $XDG_CONFIG_HOME if XDG_CONFIG_HOME is set and not empty,
    // otherwise it is "$HOME/.config"
    let default_dir = match env::var("XDG_CONFIG_HOME") {
        Err(_error) => format!(
            "{}/.config",
            env::var("HOME").expect(
                "$HOME is not set, but is needed \
             because $XDG_CONFIG_HOME is not set"
            )
        ),
        Ok(path) if path.is_empty() => format!(
            "{}/.config",
            env::var("HOME").expect(
                "$HOME is not set, but is needed \
                 because $XDG_CONFIG_HOME is empty"
            )
        ),
        Ok(path) => path,
    };

    let default_config = format!("{default_dir}/ixwindow/ixwindow.toml");

    if Path::new(&default_config).exists() {
        return Some(default_config);
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
        assert_eq!(
            config.cache_dir(),
            shellexpand::tilde(
                "~/.config/polybar/scripts/ixwindow/polybar-icons"
            )
        );
    }

    #[test]
    fn expand_filename_works() {
        let config = load_i3(None);

        assert_eq!(
            expand_filename(config.cache_dir()),
            shellexpand::tilde(
                "~/.config/polybar/scripts/ixwindow/polybar-icons"
            )
        );
    }
}
