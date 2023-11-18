use i3ipc::I3Connection;

use std::error::Error;
use std::fs;
use std::path::Path;
use std::str;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use x11rb::connection::Connection;
use x11rb::protocol::xproto::ConnectionExt;
use x11rb::rust_connection::RustConnection;

use crate::bspwm::BspwmConnection;
use crate::config::{
    self, BspwmConfig, Config, EmptyInfo, I3Config, WindowInfo,
};
use crate::i3_utils;
use crate::wm_connection::WMConnection;
use crate::x11_utils;
