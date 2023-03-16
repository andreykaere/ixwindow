use std::error::Error;
use std::string::String;

use image::imageops::FilterType;
use image::io::Reader as ImageReader;
use image::DynamicImage;
use image::GenericImageView;
use image::RgbImage;
use image::RgbaImage;

use x11rb::atom_manager;
use x11rb::connection::Connection;
use x11rb::protocol::randr::{self, ConnectionExt as _, GetCrtcInfoReply};
use x11rb::protocol::xproto::*;

atom_manager! {
    pub AtomCollection: AtomCollectionCookie {
        WM_PROTOCOLS,
        WM_DELETE_WINDOW,
        _NET_WM_NAME,
        _NET_WM_STATE,
        _NET_WM_STATE_FULLSCREEN,
        _NET_WM_ICON,
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
    let image = ImageReader::open(image_path)?.decode()?;
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

    // Swapping blue and red colors so that icon will be displayed with normal
    // colors
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

    let wm_window_id = u32::from_le_bytes(property.value[..].try_into()?);

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

pub fn get_wm_class(wid: i32) -> Result<String, Box<dyn Error>> {
    let (conn, screen_num) = x11rb::connect(None)?;
    let screen = &conn.setup().roots[screen_num];

    let property = conn
        .get_property(
            false,
            wid.try_into()?,
            AtomEnum::WM_CLASS,
            AtomEnum::STRING,
            0,
            1024,
        )?
        .reply()?;

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

pub fn is_window_fullscreen(window_id: i32) -> Result<bool, Box<dyn Error>> {
    let (conn, screen_num) = x11rb::connect(None)?;
    let screen = &conn.setup().roots[screen_num];

    let atoms = AtomCollection::new(&conn)?;
    let atoms = atoms.reply()?;

    let property = conn
        .get_property(
            false,
            window_id.try_into()?,
            atoms._NET_WM_STATE,
            AtomEnum::ATOM,
            0,
            1024,
        )?
        .reply()?;

    let data = property.value;

    if data.is_empty() {
        return Ok(false);
    }

    // Read property value by chunks of 4 bytes, because there might be more,
    // than one Atom specified in _NET_WM_STATE
    Ok(data.chunks_exact(4).fold(false, |acc, chunk| {
        let net_wm_state_atom = u32::from_le_bytes(chunk.try_into().unwrap());

        acc && (net_wm_state_atom == atoms._NET_WM_STATE_FULLSCREEN)
    }))
}

fn save_image(
    image_data: &ImageData,
    icon_path: &str,
) -> Result<(), Box<dyn Error>> {
    // let img = ImageReader::new(Cursor::new(bytes))
    //     .with_guessed_format()?
    //     .decode()?;
    // let img = image::load_from_memory_with_format(
    //     image_data.buf,
    //     image::ImageFormat::Pnm,
    // )?;

    let img = RgbaImage::from_raw(
        image_data.width,
        image_data.height,
        image_data.buf.to_vec(),
    )
    .unwrap();

    // let img2 = DynamicImage::ImageRgba8(img.clone()).to_rgba8();
    // img2.save_with_format(icon_path, image::ImageFormat::Png)?;
    // img2.save("/home/andrey/foogooo.png");
    img.save_with_format(icon_path, image::ImageFormat::Png)?;

    Ok(())
}

struct ImageData {
    width: u32,
    height: u32,
    buf: Vec<u8>,
}

pub fn generate_icon(
    icon_name: &str,
    cache_dir: &str,
    color: &str,
    window_id: i32,
) -> Result<(), Box<dyn Error>> {
    let (conn, screen_num) = x11rb::connect(None)?;
    let screen = &conn.setup().roots[screen_num];

    let atoms = AtomCollection::new(&conn)?;
    let atoms = atoms.reply()?;

    let property = conn
        .get_property(
            false,
            window_id.try_into()?,
            atoms._NET_WM_ICON,
            AtomEnum::CARDINAL,
            0,
            u32::MAX,
        )?
        .reply()?;

    let mut icons = Vec::new();
    let mut data = property.value;

    let mut i = 0;

    // println!("{}", data.len());

    while i < data.len() {
        let mut chunks = (&data[i..]).chunks_exact(4);
        // Ok(.fold(false, |acc, chunk| {
        //     let net_wm_state_atom = u32::from_le_bytes(chunk.try_into().unwrap());

        //     acc && (net_wm_state_atom == atoms._NET_WM_STATE_FULLSCREEN)
        // }))
        let width =
            u32::from_le_bytes(chunks.next().unwrap().try_into().unwrap());
        let height =
            u32::from_le_bytes(chunks.next().unwrap().try_into().unwrap());

        // println!("{}, {}", width, height);
        // data = data[8..].to_vec();
        i += 8;

        let size = usize::try_from(width * height * 4).unwrap();

        // println!("{}", i + size);

        if i + size > data.len() {
            break;
        } else {
            let mut buf = (&data[i..i + size]).to_vec();

            buf.chunks_exact_mut(4).for_each(|chunk| {
                let (c0, c2) = (chunk[0], chunk[2]);
                chunk[2] = c0;
                chunk[0] = c2;
            });

            i += size;
            icons.push(ImageData { width, height, buf });
        }
    }

    let icon_path = format!("{cache_dir}/{icon_name}.png");

    // println!("{}", icons.len());

    save_image(icons.last().unwrap(), &icon_path);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    #[ignore]
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
    #[ignore]
    fn test_is_window_fullscreen() {
        // let id = 69206018;
        let id = 20971522;
        let flag = is_window_fullscreen(id).unwrap();

        println!("flag: {flag}");
    }

    #[test]
    fn test_generate_icon() {
        let id = 71303170;
        generate_icon("foo.png", "/home/andrey", "#252737", id);
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
