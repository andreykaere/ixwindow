use std::error::Error;
use std::string::String;

use image::imageops::FilterType;
use image::io::Reader;
use image::GenericImageView;

use x11rb::atom_manager;
use x11rb::connection::Connection;
use x11rb::protocol::randr::{self, ConnectionExt as _, GetCrtcInfoReply};
use x11rb::protocol::xproto::*;

atom_manager! {
    pub AtomCollection: AtomCollectionCookie {
        WM_PROTOCOLS,
        WM_DELETE_WINDOW,
        _NET_WM_NAME,
        UTF8_STRING,
        _NET_SUPPORTING_WM_CHECK,
    }
}

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
pub fn display_icon<Conn: Connection>(
    conn: &Conn,
    image_path: &str,
    x: i16,
    y: i16,
    size: u16,
    monitor_name: &str,
) -> Result<Window, Box<dyn Error>> {
    let image = Reader::open(image_path)?.decode()?;
    let image = image.resize(size as u32, size as u32, FilterType::CatmullRom);
    let (width, height) = image.dimensions();

    // Converting to u16, because it is required later by x11rb
    let (width, height) = (width as u16, height as u16);

    let screen = &conn.setup().roots[0];
    let monitor_crtc = get_monitor_crtc(conn, monitor_name)?;

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
    create_gc(conn, gc, win, &gc_aux)?;

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

    conn.copy_area(pixmap, win, gc, 0, 0, 0, 0, width, height)?;
    conn.flush()?;

    Ok(win)
}

// https://stackoverflow.com/questions/758648/find-the-name-of-the-x-window-manager
pub fn get_current_wm() -> Result<String, Box<dyn Error>> {
    let (conn, screen_num) = x11rb::connect(None)?;
    let screen = &conn.setup().roots[screen_num];

    let atoms = AtomCollection::new(&conn)?;
    let atoms = atoms.reply()?;

    let property = conn
        .get_property(
            false,
            screen.root,
            atoms._NET_SUPPORTING_WM_CHECK,
            AtomEnum::WINDOW,
            0,
            1024,
        )?
        .reply()?;

    let wm_window_id =
        u32::from_le_bytes(property.value[..].try_into().unwrap());

    let property = conn
        .get_property(
            false,
            wm_window_id,
            atoms._NET_WM_NAME,
            atoms.UTF8_STRING,
            0,
            1024,
        )?
        .reply()?;

    let wm_name = String::from_utf8(property.value)?;

    Ok(wm_name)
}

pub fn get_wm_class(id: i32) -> Result<String, Box<dyn Error>> {
    let (conn, screen_num) = x11rb::connect(None)?;
    let screen = &conn.setup().roots[screen_num];

    let property = conn
        .get_property(
            false,
            id.try_into().unwrap(),
            AtomEnum::WM_CLASS,
            AtomEnum::STRING,
            0,
            1024,
        )?
        .reply()?;

    // println!("{:#?}", property.value);

    // let (wm_class, wm_instance) = property.value.split(|x| x == 0).collect();
    let mut iter = property.value.split(|x| *x == 0);

    let wm_class = iter.next();
    let wm_instance = iter.next();

    if let Some(name) = wm_instance {
        return Ok(String::from_utf8(name.to_vec())?);
    }

    if let Some(name) = wm_class {
        return Ok(String::from_utf8(name.to_vec())?);
    }

    Ok(String::new())
}

pub fn is_window_fullscreen(id: i32) -> Result<String, Box<dyn Error>> {
    let (conn, screen_num) = x11rb::connect(None)?;
    let screen = &conn.setup().roots[screen_num];

    let atoms = AtomCollection::new(&conn)?;
    let atoms = atoms.reply()?;

    // let net_wm_state = conn.intern_atom(false, b"_NET_WM_STATE")?.reply()?.atom;

    // let wm_class = conn.intern_atom(false, b"WM_CLASS")?.reply()?.atom;

    // println!("{} ? {:?}", wm_class, AtomEnum::WM_CLASS);

    let property = conn
        .get_property(
            false,
            id.try_into().unwrap(),
            atoms._NET_WM_NAME,
            atoms.UTF8_STRING,
            0,
            1024,
        )?
        .reply()?;

    Ok(String::from_utf8(property.value)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_get_wm_class() {
        // let id = 69206018;
        let id = 123731974;
        let wm_class = get_wm_class(id).unwrap();

        println!("{wm_class}");
    }

    #[test]
    fn test_get_current_wm() {
        // let id = 69206018;
        let wm = get_current_wm().unwrap();

        println!("wm: {wm}");
    }

    #[test]
    fn test_is_window_fullscreen() {
        // let id = 69206018;
        let id = 37748738;
        let flag = is_window_fullscreen(id).unwrap();

        println!("flag: {flag}");
    }
    // fn get_icon_path() -> String {
    //     env::current_dir().unwrap().to_str().unwrap().to_owned()
    //         + "/tests/alacritty.png"
    // }

    // fn display(monitor_name: &str) {
    //     display_icon(
    //         &get_icon_path(),
    //         270,
    //         6,
    //         24,
    //         monitor_name,
    //         Arc::new(AtomicBool::new(true)),
    //     );
    // }

    // #[test]
    // fn display_icon_test() {
    //     let monitor_name = "eDP-1";
    //     display(monitor_name);
    // }
}
