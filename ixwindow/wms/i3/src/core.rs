use i3ipc::reply::Node;
use i3ipc::I3Connection;

use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::str;
use std::thread;

use super::config::{CommonConfig, I3Config};
use super::display_icon::display_icon;
use super::utils::*;

pub struct State {
    pub curr_icon: Option<String>,
    pub prev_icon: Option<String>,
    pub curr_window: Option<i32>,
    pub curr_desktop: i32,
    pub dyn_x: u16,
}

impl State {
    pub fn update_icon(&mut self, icon_name: &str) {
        self.prev_icon = self.curr_icon.as_ref().map(|x| x.to_string());
        self.curr_icon = Some(icon_name.to_string());
    }

    pub fn reset_icons(&mut self) {
        self.prev_icon = None;
        self.curr_icon = None;
    }
}

pub struct Core {
    pub config: I3Config,
    pub state: State,
    pub connection: I3Connection,
}

impl Core {
    pub fn init() -> Self {
        let connection =
            I3Connection::connect().expect("Failed to connect to i3");
        let config = CommonConfig::load_i3();

        let state = State {
            curr_icon: None,
            prev_icon: None,
            curr_window: None,
            curr_desktop: 1,
            dyn_x: config.x,
        };

        Self {
            config,
            connection,
            state,
        }
    }

    pub fn generate_icon(&self, window: i32) {
        let config = &self.config;

        if !Path::new(&config.cache_dir).is_dir() {
            fs::create_dir(&config.cache_dir)
                .expect("No cache folder was detected and couldn't create it");
        }

        let mut generate_icon_child = Command::new("../generate-icon")
            .arg(&config.cache_dir)
            .arg(config.size.to_string())
            .arg(&config.color)
            .arg(window.to_string())
            .stderr(Stdio::null())
            .spawn()
            .expect("Couldn't generate icon");

        generate_icon_child.wait().expect("Failed to wait on child");
    }

    pub fn update_dyn_x(&mut self) {
        // -1 because of scratchpad desktop
        let desks_num = self.get_desktops_as_nodes().len() - 1;
        let config = &self.config;
        let new_x = config.x + config.gap_per_desk * (desks_num as u16);

        self.state.dyn_x = new_x;
    }

    pub fn show_icon(&self, icon_path: String) {
        let config = &self.config;

        let (icon, dyn_x, y, size) =
            (icon_path, self.state.dyn_x, config.y, config.size);

        thread::spawn(move || {
            display_icon(&icon, dyn_x, y, size);
        });
    }

    pub fn process_icon(&mut self, window: i32) {
        let icon_name = get_icon_name(window);

        // If icon is the same, don't do anything
        if let Some(prev_icon) = &self.state.prev_icon {
            if &icon_name == prev_icon {
                return;
            }
        }

        let config = &self.config;
        let icon_path = format!("{}/{}.jpg", &config.cache_dir, icon_name);

        if !Path::new(&icon_path).exists() {
            self.generate_icon(window);
        }

        self.destroy_prev_icons();
        self.show_icon(icon_path);
    }

    pub fn print_info(&self, maybe_window: Option<i32>) {
        // Don't add '\n' at the end, so that it will appear in front of icon
        // name, printed after it
        print!("{}", self.config.gap);
        io::stdout().flush().unwrap();

        match maybe_window {
            None => println!("Empty"),

            Some(window) => {
                let icon_name = &get_icon_name(window);

                match icon_name.as_ref() {
                    "Brave-browser" => println!("Brave"),
                    "TelegramDesktop" => println!("Telegram"),
                    _ => println!("{}", capitalize_first(icon_name)),
                }
            }
        }
    }

    pub fn destroy_prev_icons(&mut self) {
        let icons_ids_raw = Command::new("xdo")
            .arg("id")
            .arg("-n")
            .arg("polybar-ixwindow-icon")
            .stderr(Stdio::null())
            .output()
            .expect("Couldn't detect any 'polybar-xwindow-icon' windows");

        let output = match String::from_utf8(icons_ids_raw.stdout) {
            Ok(v) => v,
            Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
        };

        let icons_ids = output.trim().split('\n');

        for id in icons_ids {
            let mut xdo_kill_child = Command::new("xdo")
                .arg("kill")
                .arg(id)
                .stderr(Stdio::null())
                .spawn()
                .expect("xdo couldn't kill icon window");

            xdo_kill_child.wait().expect("Failed to wait on child");
        }
    }

    pub fn process_focused_window(&mut self, window: i32) {
        if is_window_fullscreen(window) {
            self.process_fullscreen_window();
            return;
        }

        let icon_name = get_icon_name(window);

        self.print_info(Some(window));
        self.state.update_icon(&icon_name);
        self.process_icon(window);
    }

    pub fn get_fullscreen_window(&mut self, desktop: i32) -> Option<i32> {
        let nodes = self.get_desktop_windows(desktop);

        for node in nodes {
            if let Some(id) = node.window {
                if is_window_fullscreen(id) {
                    return Some(id);
                }
            }
        }

        // If no fullscreen window is found in this desktop
        None
    }

    // Come up with a better name
    pub fn process_fullscreen_window(&mut self) {
        self.destroy_prev_icons();

        // Reset icons, so that we can use process_focused_window
        // after. Otherwise it will not display icon, since app
        // name didn't change during fullscreen toggling
        self.state.reset_icons();
    }

    pub fn get_all_nodes(&mut self) -> Vec<Node> {
        let connection = &mut self.connection;
        let tree = connection
            .get_tree()
            .expect("Couldn't read information about tree");

        get_all_childs(tree)
    }

    pub fn get_focused_window(&mut self) -> Option<i32> {
        let nodes = self.get_all_nodes();

        for node in nodes {
            if node.focused {
                return node.window;
            }
        }

        // If no window is focused
        None
    }

    fn get_desktops_as_nodes(&mut self) -> Vec<Node> {
        let connection = &mut self.connection;
        let tree = connection
            .get_tree()
            .expect("Couldn't read information about tree");

        get_desktop_subnodes(tree)
    }

    fn get_desktop_windows(&mut self, desktop: i32) -> Vec<Node> {
        let desktops = self.get_desktops_as_nodes();

        for desk in desktops {
            let desk_name = desk.name.unwrap();

            if desk_name == desktop.to_string() {
                return desk.nodes;
            }
        }

        vec![]
    }

    pub fn get_focused_desktop(&mut self) -> i32 {
        let connection = &mut self.connection;
        let desktops = connection
            .get_workspaces()
            .expect("Couldn't read information about desktops")
            .workspaces;

        for desktop in desktops {
            if desktop.focused {
                return desktop.num;
            }
        }

        panic!("Zero desktops!");
    }

    pub fn convert_desktop(&mut self, desktop: i32) -> Node {
        let desktops = self.get_desktops_as_nodes();

        for desk in desktops {
            if desk.name == Some(desktop.to_string()) {
                return desk;
            }
        }

        panic!("Something went wrong, when converting desktop to node");
    }

    pub fn is_empty(&mut self, desktop: i32) -> bool {
        let node = self.convert_desktop(desktop);

        node.nodes.is_empty()
    }

    pub fn process_empty_desktop(&mut self) {
        self.destroy_prev_icons();
        self.state.reset_icons();
        self.print_info(None);
    }
}

#[cfg(test)]
mod tests {
    use super::super::I3Config;
    use super::*;
    use i3ipc::I3Connection;

    #[test]
    fn get_all_childs_works() {
        let mut connection = I3Connection::connect().unwrap();
        let tree = connection
            .get_tree()
            .expect("Couldn't read information about tree");

        println!("All windows:\n{:?}", get_all_childs(tree));
    }

    #[test]
    fn get_focused_window_works() {
        let mut core = Core::init();
        let window = core.get_focused_window();

        println!("{:?}", window);
    }

    #[test]
    fn get_desktop_windows_works() {
        let mut core = Core::init();
        let desktop = core.get_focused_desktop();
        let result = core.get_desktop_windows(desktop);

        println!("{:?}", result);
    }
}
