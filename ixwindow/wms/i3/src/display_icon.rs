use std::error::Error;
use std::string::String;

use image::imageops::FilterType;
use image::io::Reader;
use image::GenericImageView;
use x11rb::connection::Connection;
use x11rb::protocol::randr::{
    self, get_output_info, get_screen_resources, GetScreenResourcesReply,
};
use x11rb::protocol::xproto::*;
use x11rb::protocol::Event;

// use x11rb::xcb_ffi::XCBConnection;

fn get_screen_name<C: Connection>(
    conn: &C,
    screen_num: usize,
) -> Option<String> {
    let setup = conn.setup();
    let screen = &setup.roots[screen_num];
    let get_screen_resources_cookie =
        randr::get_screen_resources(conn, screen.root);
    let get_screen_resources_reply =
        get_screen_resources_cookie.unwrap().reply().unwrap();

    // Find the output that corresponds to the screen
    let mut screen_output = None;
    for output in get_screen_resources_reply.outputs {
        let get_output_info_cookie = randr::get_output_info(conn, output, 0);
        let get_output_info_reply =
            get_output_info_cookie.unwrap().reply().unwrap();

        if get_output_info_reply.crtc == screen.root {
            screen_output = Some(output);
            break;
        }
    }

    if let Some(output) = screen_output {
        let get_output_info_cookie = get_output_info(conn, output, 0);
        let get_output_info_reply =
            get_output_info_cookie.unwrap().reply().unwrap();
        let output_name =
            String::from_utf8_lossy(&get_output_info_reply.name).to_string();
        return Some(output_name);
    }

    None
}

// fn get_screen_number_by_name(
//     conn: &XCBConnection,
//     screen_name: &str,
// ) -> Option<usize> {
// Loop over the screens and find the one with the specified name
// let setup = conn.setup();
// for (screen_num, screen) in setup.roots().enumerate() {
//     let get_screen_resources_cookie =
//         randr::get_screen_resources(conn, screen.root());
//     let get_screen_resources_reply =
//         get_screen_resources_cookie.unwrap().reply().unwrap();

//     // Loop over the outputs of the screen and find the one with the specified name
//     for output in get_screen_resources_reply.outputs() {
//         let get_output_info_cookie =
//             randr::get_output_info(conn, *output, 0);
//         let get_output_info_reply =
//             get_output_info_cookie.unwrap().reply().unwrap();

//         let output_name =
//             String::from_utf8_lossy(get_output_info_reply.name())
//                 .to_string();

//         if output_name == screen_name {
//             return Some(screen_num);
//         }
//     }
// }

// None
// }

// fn get_screen_name<Conn: Connection>(conn: &Conn, screen: Screen) -> String {
//     let screen_res = get_screen_resources(conn, screen.root)?.reply()?;

//     for (i, output) in screen_res.outputs.into_iter().enumerate() {
//         let output_info = get_output_info(conn, output, 0)?.reply()?;
//         let name = String::from_utf8(output_info.name)?;

//         if i == screen_num {
//             return name;
//         }
//     }
// }

// fn convert_screen_name_to_num<Conn: Connection>(
//     conn: &Conn,
//     monitor_name: &str,
// ) -> Result<usize, Box<dyn Error>> {
//     for screen_num in &conn.setup().roots {
//         // let screen_res = get_screen_resources(conn, screen.root)?.reply()?;
//         // let result = screen_res.outputs;
//         let name = get_screen_name(conn, screen_num);
//         println!("{name:?}");
//     }

//     Err("No monitor with this name was found".into())
// }

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
    println!("foo: {screen_num}");

    println!("{:?}", get_screen_name(&conn, 0));

    // let (conn1, _) = x11rb::xcb_ffi::XCBConnection::connect(None)?;

    // let screen_num1 = get_screen_number_by_name(conn1, monitor_name).unwrap();
    // println!("bar: {screen_num1}");

    let screen = &conn.setup().roots[screen_num];

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

#[cfg(test)]
mod tests {
    use super::super::i3_utils as i3;
    use super::*;
    use i3ipc::I3Connection;

    #[test]
    fn display_icon_works() {
        let mut conn = I3Connection::connect().unwrap();
        let monitor_name = i3::get_focused_monitor(&mut conn);

        // println!("{monitor_name}");

        display_icon("/home/andrey/alacritty.png", 270, 6, 24, &monitor_name);
    }
}
