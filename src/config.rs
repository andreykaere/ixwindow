#![allow(clippy::enum_variant_names)]

use serde::{Deserialize, Serialize};

use std::cmp::min;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

mod utils {
    // Capitalizes first letter of the string, i.e. converts foo to Foo
    pub fn capitalize_first(string: &str) -> String {
        let mut chars = string.chars();

        match chars.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().chain(chars).collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommonConfig {
    gap: u16,
    x: i16,
    y: i16,
    size: u16,
    cache_dir: PathBuf,
    color: String,

    #[serde(rename = "print_info")]
    #[serde(default)]
    print_info_settings: PrintInfoSettings,
}

#[derive(
    Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Hash, Eq,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WindowInfoType {
    #[default]
    WmInstance,

    WmClass,
    WmName,

    #[serde(rename = "_NET_WM_NAME")]
    NetWmName,

    NetWmVisibleName,
}

fn default_info_types() -> Vec<WindowInfoType> {
    vec![WindowInfoType::WmInstance]
}

fn default_empty_info() -> Option<String> {
    Some("Empty".to_string())
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct PrintInfoSettings {
    #[serde(rename = "types", default = "default_info_types")]
    pub info_types: Vec<WindowInfoType>,

    #[serde(default)]
    pub max_len: Option<usize>,

    #[serde(default)]
    pub capitalize_first: Vec<WindowInfoType>, // for which types capitalize first letter

    #[serde(default)]
    pub substitute_rules: HashMap<WindowInfoType, HashMap<String, String>>,

    #[serde(rename = "label_empty", default = "default_empty_info")]
    pub empty_info: Option<String>,
}

impl PrintInfoSettings {
    pub fn format_info(
        &self,
        info: &str,
        info_type_format: Option<WindowInfoType>,
    ) -> String {
        let mut formatted_info = info.to_string();

        if let Some(info_type) = info_type_format {
            formatted_info = self.capitalize_first(&formatted_info, info_type);
            formatted_info =
                self.apply_substitute_rules(&formatted_info, info_type);
        }

        // If max_len is not specified, then we don't bound the length of the
        // output info
        let cut_len = if let Some(max_len) = self.max_len {
            min(max_len, formatted_info.len())
        } else {
            formatted_info.len()
        };

        formatted_info.chars().take(cut_len).collect()
    }

    fn apply_substitute_rules(
        &self,
        info: &str,
        info_type: WindowInfoType,
    ) -> String {
        if self.substitute_rules.contains_key(&info_type) {
            let rules = self.substitute_rules.get(&info_type).unwrap();

            for (old, new) in rules {
                if info == old {
                    return new.to_string();
                }
            }
        }

        info.to_string()
    }

    pub fn capitalize_first(
        &self,
        info: &str,
        info_type: WindowInfoType,
    ) -> String {
        if self.capitalize_first.contains(&info_type) {
            utils::capitalize_first(info)
        } else {
            info.to_string()
        }
    }

    pub fn get_empty_desk_info(&self) -> &str {
        match &self.empty_info {
            Some(x) => x,
            None => "Empty",
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct I3Config {
    #[serde(flatten)]
    common_config: CommonConfig,

    pub gap_per_desk: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BspwmConfig {
    #[serde(flatten)]
    common_config: CommonConfig,
}

pub trait Config {
    fn common_config(&self) -> &CommonConfig;

    fn gap(&self) -> String {
        let mut acc = String::new();
        let gap = self.common_config().gap;

        for _ in 1..=gap {
            acc.push(' ');
        }
        acc
    }

    fn color(&self) -> &str {
        &self.common_config().color
    }

    fn cache_dir(&self) -> &Path {
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

    fn print_info_settings(&self) -> &PrintInfoSettings {
        &self.common_config().print_info_settings
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

pub fn read_to_table(config_file: Option<&Path>) -> toml::Table {
    let config_filename = if let Some(name) = config_file {
        name.to_path_buf()
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

pub fn load_i3(config_file: Option<&Path>) -> I3Config {
    let mut table = read_to_table(config_file);

    // We use remove here, because we need ownership for try_into
    let config_table = table.remove("i3").unwrap();

    let mut i3_config: I3Config = config_table.try_into().unwrap();
    i3_config.common_config.cache_dir =
        expand_filename(&i3_config.common_config.cache_dir);

    i3_config
}

pub fn load_bspwm(config_file: Option<&Path>) -> BspwmConfig {
    let mut table = read_to_table(config_file);

    // We use remove here, because we need ownership for try_into
    let config_table = table.remove("bspwm").unwrap();

    let mut bspwm_config: BspwmConfig = config_table.try_into().unwrap();
    bspwm_config.common_config.cache_dir =
        expand_filename(&bspwm_config.common_config.cache_dir);

    bspwm_config
}

fn locate_config_file() -> Option<PathBuf> {
    if let Ok(specified_config) = env::var("IXWINDOW_CONFIG_PATH") {
        if Path::new(&specified_config).exists() {
            return Some(PathBuf::from(specified_config));
        }
    }

    let default_dir = match env::var("XDG_CONFIG_HOME") {
        Err(_) => format!(
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
        return Some(PathBuf::from(default_config));
    }

    None
}

fn expand_filename(file: &Path) -> PathBuf {
    let filename = file.to_string_lossy().to_string();
    let filename = &shellexpand::env(&filename).unwrap();
    let filename = shellexpand::tilde(&filename).to_string();

    PathBuf::from(filename)
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONFIG_PATH: &str = "./examples/ixwindow.toml";

    #[test]
    fn locate_config_file_works() {
        locate_config_file();
    }

    #[test]
    fn parse_config_works() {
        let config_path = PathBuf::from(CONFIG_PATH);
        let config = load_i3(Some(&config_path));

        assert_eq!(config.size(), 24);
        assert_eq!(
            config.cache_dir(),
            PathBuf::from(
                shellexpand::tilde(
                    "~/.config/polybar/scripts/ixwindow/polybar-icons"
                )
                .to_string()
            )
        );
    }

    #[test]
    fn expand_filename_works() {
        let config_path = PathBuf::from(CONFIG_PATH);
        let config = load_i3(Some(&config_path));

        assert_eq!(
            expand_filename(config.cache_dir()),
            PathBuf::from(
                shellexpand::tilde(
                    "~/.config/polybar/scripts/ixwindow/polybar-icons"
                )
                .to_string()
            )
        );
    }
}
