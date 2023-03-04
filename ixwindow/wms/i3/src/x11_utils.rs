use std::error::Error;
use std::string::String;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use image::imageops::FilterType;
use image::io::Reader;
use image::GenericImageView;

use x11rb::connection::Connection;
use x11rb::protocol::randr::{
    self, get_output_info, get_screen_resources, ConnectionExt as _,
    GetCrtcInfoReply, GetScreenResourcesReply,
};
use x11rb::protocol::xproto::*;
use x11rb::protocol::Event;

pub fn get_primary_monitor_name() -> Result<String, Box<dyn Error>> {
    let (conn, screen_num) = x11rb::connect(None)?;
    let screen = &conn.setup().roots[screen_num];

    let output_primary =
        conn.randr_get_output_primary(screen.root)?.reply()?.output;

    let output_primary_info =
        conn.randr_get_output_info(output_primary, 0)?.reply()?;

    Ok(String::from_utf8(output_primary_info.name)?)
}

fn get_monitor_crtc<Conn: Connection>(
    conn: &Conn,
    monitor_name: &str,
) -> Result<GetCrtcInfoReply, Box<dyn Error>> {
    let screen = &conn.setup().roots[0];
    let resources = conn
        .randr_get_screen_resources_current(screen.root)?
        .reply()?;

    for output in resources.outputs {
        let output_info = conn.randr_get_output_info(output, 0)?.reply()?;

        if std::str::from_utf8(&output_info.name)? == monitor_name {
            let crtc_info =
                conn.randr_get_crtc_info(output_info.crtc, 0)?.reply()?;

            if output_info.connection == randr::Connection::CONNECTED {
                return Ok(crtc_info);
            }
        }
    }

    Err("Couldn't get given monitor CRTC".into())
}

// Add icon-handler to MonitorState to be able to kill it later
pub fn display_icon(
    image_path: &str,
    x: i16,
    y: i16,
    size: u16,
    monitor_name: &str,
    flag: &AtomicBool,
) -> Result<(), Box<dyn Error>> {
    let image = Reader::open(image_path)?.decode()?;
    let image = image.resize(size as u32, size as u32, FilterType::CatmullRom);
    let (width, height) = image.dimensions();

    // Converting to u16, because it is required later by x11rb
    let (width, height) = (width as u16, height as u16);

    let (conn, screen_num) = x11rb::connect(None)?;
    let screen = &conn.setup().roots[screen_num];
    let monitor_crtc = get_monitor_crtc(&conn, &monitor_name)?;

    let wm_class = b"polybar-ixwindow-icon";
    let win = conn.generate_id()?;
    let window_aux = CreateWindowAux::default()
        .override_redirect(1)
        .event_mask(EventMask::EXPOSURE);

    conn.create_window(
        x11rb::COPY_FROM_PARENT as u8,
        win,
        screen.root,
        monitor_crtc.x + x,
        monitor_crtc.y + y,
        width,
        height,
        0,
        WindowClass::COPY_FROM_PARENT,
        screen.root_visual,
        &window_aux,
    )?;
    conn.map_window(win)?;
    conn.flush()?;

    conn.change_property(
        PropMode::REPLACE,
        win,
        AtomEnum::WM_CLASS,
        AtomEnum::STRING,
        8,
        wm_class.len() as u32,
        wm_class,
    )?;

    let gc_aux = CreateGCAux::new();
    let gc = conn.generate_id()?;
    create_gc(&conn, gc, win, &gc_aux)?;

    let pixmap = conn.generate_id()?;
    conn.create_pixmap(screen.root_depth, pixmap, win, width, height)?;

    // This is needed for making icon colorful
    let mut data = image.into_rgba8().into_raw();
    data.chunks_exact_mut(4).for_each(|chunk| {
        let (c0, c2) = (chunk[0], chunk[2]);
        chunk[2] = c0;
        chunk[0] = c2;
    });

    conn.put_image(
        ImageFormat::Z_PIXMAP,
        pixmap,
        gc,
        width,
        height,
        0,
        0,
        0,
        screen.root_depth,
        &data,
    )?;

    conn.flush()?;

    let timeout = Duration::from_millis(10);

    while !flag.load(Ordering::SeqCst) {
        // let event = conn.poll_for_event()?;
        conn.copy_area(pixmap, win, gc, 0, 0, 0, 0, width, height)?;
        conn.flush()?;

        thread::sleep(timeout);
    }

    // loop {
    // let event = conn.poll_for_event()?;

    // last_event_time = event.time;

    // let current_time = conn.conn_time()?;
    // let elapsed_time = current_time - last_event_time;
    // if elapsed_time >= timeout {
    //     continue;
    // }

    // println!("1:{event:#?} {}", flag.load(Ordering::SeqCst));
    // println!("bar");

    // if let Event::Expose(_) = event {
    //     conn.copy_area(pixmap, win, gc, 0, 0, 0, 0, width, height)?;
    //     conn.flush()?;
    // }
    // conn.flush()?;
    // }

    // println!("foo");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::i3_utils as i3;
    use super::*;
    use crate::config::format_filename;
    use i3ipc::I3Connection;
    use std::env;

    fn get_icon_path() -> String {
        env::current_dir().unwrap().to_str().unwrap().to_owned()
            + "/tests/alacritty.png"
    }

    fn display(monitor_name: &str) {
        display_icon(&get_icon_path(), 270, 6, 24, monitor_name);
    }

    #[test]
    fn display_icon_test() {
        let monitor_name = "eDP-1";
        display(monitor_name);
    }
}
