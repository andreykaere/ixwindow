use std::error::Error;
use std::string::String;

use image::imageops::FilterType;
use image::io::Reader;
use image::GenericImageView;

use x11rb::connection::Connection;
use x11rb::protocol::randr::{
    self, get_output_info, get_screen_resources, ConnectionExt as _,
    GetScreenResourcesReply,
};
use x11rb::protocol::xproto::*;
use x11rb::protocol::Event;

fn get_screen_name_by_num<C: Connection>(
    conn: &C,
    screen_num: usize,
) -> Result<String, Box<dyn Error>> {
    let screen = &conn.setup().roots[screen_num];
    let randr_monitors = conn.randr_get_monitors(screen.root, true)?.reply()?;

    for (i, monitor) in randr_monitors.monitors.into_iter().enumerate() {
        let name = conn.get_atom_name(monitor.name)?.reply()?;
        let monitor_name = String::from_utf8(name.name)?;

        if i == screen_num {
            return Ok(monitor_name);
        }
    }

    Err("Couldn't get screen name by given number".into())
}

pub fn get_default_monitor() -> String {
    let (conn, _) = x11rb::connect(None).unwrap();
    get_screen_name_by_num(&conn, 0)
        .expect("Couldn't get a name of the default monitor")
}

fn get_screen_num_by_name<Conn: Connection>(
    conn: &Conn,
    screen: &Screen,
    monitor_name: &str,
) -> Result<usize, Box<dyn Error>> {
    let randr_monitors = conn.randr_get_monitors(screen.root, true)?.reply()?;

    for (num, mon) in randr_monitors.monitors.into_iter().enumerate() {
        let name = conn.get_atom_name(mon.name)?.reply()?;
        let mon_name = String::from_utf8(name.name)?;

        if monitor_name == &mon_name {
            return Ok(num);
        }
    }

    Err("Couldn't find a monitor with given name".into())
}

// Add icon-handler to MonitorState to be able to kill it later
pub fn display_icon(
    image_path: &str,
    x: u16,
    y: u16,
    size: u16,
    monitor_name: &str,
) -> Result<(), Box<dyn Error>> {
    let image = Reader::open(image_path)?.decode()?;
    let image = image.resize(size as u32, size as u32, FilterType::CatmullRom);
    let (width, height) = image.dimensions();

    let (conn, screen_num) = x11rb::connect(None)?;
    let screen = &conn.setup().roots[screen_num];
    // let screen_num = get_screen_num_by_name(&conn, screen, monitor_name);

    let win = conn.generate_id()?;
    let window_aux = CreateWindowAux::default()
        .override_redirect(1)
        .border_pixel(screen.black_pixel)
        .event_mask(EventMask::EXPOSURE);

    let wm_class = b"polybar-ixwindow-icon";

    conn.create_window(
        x11rb::COPY_FROM_PARENT as u8,
        win,
        screen.root,
        x.try_into()?,
        y.try_into()?,
        width.try_into()?,
        height.try_into()?,
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
    conn.create_pixmap(
        screen.root_depth,
        pixmap,
        win,
        width as u16,
        height as u16,
    )?;

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
        width as u16,
        height as u16,
        0,
        0,
        0,
        screen.root_depth,
        &data,
    )?;

    loop {
        let event = conn.wait_for_event()?;

        if let Event::Expose(_) = event {
            conn.copy_area(
                pixmap,
                win,
                gc,
                0,
                0,
                0,
                0,
                width.try_into()?,
                height.try_into()?,
            )?;
            conn.flush()?;
        }
    }
}

fn display_window_on_monitor(conn: &impl Connection, monitor_name: &str) {
    // Get the screen resources
    let resources = conn
        .randr_get_screen_resources_current(conn.setup().roots[0].root)
        .unwrap()
        .reply()
        .unwrap();

    // Iterate over the outputs and look for the one with the matching name
    let mut output_info: Option<randr::GetOutputInfoReply> = None;
    let mut crtc_info: Option<randr::GetCrtcInfoReply> = None;
    for output in resources.outputs {
        let oi = conn
            .randr_get_output_info(output, 0)
            .unwrap()
            .reply()
            .unwrap();
        if std::str::from_utf8(&oi.name).unwrap() == monitor_name {
            let ci = conn
                .randr_get_crtc_info(oi.crtc, 0)
                .unwrap()
                .reply()
                .unwrap();
            if oi.connection == randr::Connection::CONNECTED
            // && ci.mode() != randr::Mode::None
            {
                output_info = Some(oi);
                crtc_info = Some(ci);
                break;
            }
        }
    }

    // If we found a matching output, create and display a window on that monitor
    if let (Some(output_info), Some(crtc_info)) = (output_info, crtc_info) {
        let window_id = conn.generate_id().unwrap();
        let (width, height) = (800, 600);
        let aux = CreateWindowAux::new()
            .event_mask(EventMask::EXPOSURE)
            .background_pixel(0xffffff)
            .override_redirect(1);

        let screen = &conn.setup().roots[0];
        conn.create_window(
            x11rb::COPY_FROM_PARENT.try_into().unwrap(),
            window_id,
            screen.root,
            crtc_info.x.try_into().unwrap(),
            crtc_info.y,
            width,
            height,
            0,
            WindowClass::COPY_FROM_PARENT,
            screen.root_visual,
            &aux,
        )
        .unwrap();
        conn.map_window(window_id).unwrap();
        conn.flush().unwrap();
    }

    loop {
        let event = conn.wait_for_event().unwrap();

        conn.flush().unwrap()
    }
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
    fn display_window_on_monitor1_works() {
        // Open a connection to the X server
        let (conn, _) = x11rb::connect(None).unwrap();

        // Display a window on the monitor with the given name
        let monitor_name = "DisplayPort-1";
        display_window_on_monitor(&conn, monitor_name);
    }

    #[test]
    fn display_window_on_monitor2_works() {
        // Open a connection to the X server
        let (conn, _) = x11rb::connect(None).unwrap();

        // Display a window on the monitor with the given name
        let monitor_name = "DisplayPort-2";
        display_window_on_monitor(&conn, monitor_name);
    }
}
