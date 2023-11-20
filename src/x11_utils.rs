use anyhow::{bail, Context};
use std::path::Path;
use std::string::String;

use image::imageops::FilterType;
use image::io::Reader as ImageReader;
use image::{GenericImageView, RgbaImage};

use crate::config::WindowInfoType;
use crate::core::WindowInfo;

use x11rb::atom_manager;
use x11rb::connection::Connection;
use x11rb::protocol::randr::{self, ConnectionExt as _, GetCrtcInfoReply};
use x11rb::protocol::xproto::*;

atom_manager! {
    pub AtomCollection: AtomCollectionCookie {
        WM_PROTOCOLS,
        WM_DELETE_WINDOW,
        _NET_WM_NAME,
        _NET_WM_VISIBLE_NAME,
        WM_NAME,
        _NET_WM_STATE,
        _NET_WM_STATE_FULLSCREEN,
        _NET_WM_ICON,
        UTF8_STRING,
        _NET_SUPPORTING_WM_CHECK,
    }
}

struct ImageData {
    width: u32,
    height: u32,
    buf: Vec<u8>,
}

pub fn get_primary_monitor_name() -> anyhow::Result<String> {
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
) -> anyhow::Result<GetCrtcInfoReply> {
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

    bail!("Couldn't get given monitor CRTC")
}

// Add icon-handler to MonitorState to be able to kill it later
pub fn display_icon<Conn: Connection>(
    conn: &Conn,
    image_path: &Path,
    x: i16,
    y: i16,
    size: u16,
    monitor_name: &str,
) -> anyhow::Result<Window> {
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
pub fn get_current_wm() -> anyhow::Result<String> {
    let (conn, screen_num) = x11rb::connect(None)?;
    let screen = &conn.setup().roots[screen_num];
    let atoms = AtomCollection::new(&conn)?.reply()?;

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

    let wm_window_id = property.value32().unwrap().next().unwrap();

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

pub fn get_wm_class(wid: u32) -> anyhow::Result<String> {
    let (conn, _) = x11rb::connect(None)?;

    let property = conn
        .get_property(
            false,
            wid,
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

//fn get_window_info<Conn: Connection>(
//    conn: &Conn,
//    event_info: PropertyNotifyEvent,
//    info_type: WindowInfoType,
//) -> Result<String, Box<dyn Error>> {
//    match info_type {
//        WindowInfoType::WmClass => {
//            if event_info.atom == Into::<u32>::into(AtomEnum::WM_CLASS) {
//                //
//            }
//        }

//        WindowInfoType::WmInstance => {
//            if event_info.atom == Into::<u32>::into(AtomEnum::WM_CLASS) {
//                //
//            }
//        }

//        WindowInfoType::WmName => {
//            if event_info.atom == Into::<u32>::into(AtomEnum::WM_NAME) {
//                //
//            }
//        }
//    }
//}

pub fn get_window_info(
    window_id: u32,
    info_types: &[WindowInfoType],
) -> anyhow::Result<WindowInfo> {
    let (conn, _) = x11rb::connect(None)?;
    let atoms = AtomCollection::new(&conn)?.reply()?;

    for info_type in info_types {
        let info_bytes = match info_type {
            WindowInfoType::WmClass => {
                let property = conn
                    .get_property(
                        false,
                        window_id,
                        AtomEnum::WM_CLASS,
                        AtomEnum::STRING,
                        0,
                        1024,
                    )?
                    .reply()?;

                let mut iter = property.value.split(|x| *x == 0);
                let wm_class = iter.next();

                wm_class.map(|x| x.to_vec())
            }

            WindowInfoType::WmInstance => {
                let property = conn
                    .get_property(
                        false,
                        window_id,
                        AtomEnum::WM_CLASS,
                        AtomEnum::STRING,
                        0,
                        1024,
                    )?
                    .reply()?;

                let mut iter = property.value.split(|x| *x == 0);
                let wm_instance = iter.nth(1);

                wm_instance.map(|x| x.to_vec())
            }

            WindowInfoType::NetWmName => {
                let property = conn
                    .get_property(
                        false,
                        window_id,
                        atoms._NET_WM_NAME,
                        atoms.UTF8_STRING,
                        0,
                        1024,
                    )?
                    .reply()?;

                let wm_name = property.value;

                Some(wm_name)
            }

            WindowInfoType::NetWmVisibleName => {
                let property = conn
                    .get_property(
                        false,
                        window_id,
                        atoms._NET_WM_VISIBLE_NAME,
                        atoms.UTF8_STRING,
                        0,
                        1024,
                    )?
                    .reply()?;

                let wm_name = property.value;

                Some(wm_name)
            }

            WindowInfoType::WmName => {
                let property = conn
                    .get_property(
                        false,
                        window_id,
                        AtomEnum::WM_NAME,
                        AtomEnum::STRING,
                        0,
                        1024,
                    )?
                    .reply()?;

                let wm_name = property.value;

                Some(wm_name)
            }
        };

        if let Some(bytes) = info_bytes {
            if !bytes.is_empty() {
                return Ok(WindowInfo {
                    info: String::from_utf8_lossy(&bytes).to_string(),
                    info_type: info_type.to_owned(),
                });
            }
        }
    }

    Ok(WindowInfo {
        info: String::new(),
        info_type: info_types.last().unwrap().to_owned(),
    })
}

pub fn is_window_fullscreen(window_id: u32) -> anyhow::Result<bool> {
    let (conn, _) = x11rb::connect(None)?;
    let atoms = AtomCollection::new(&conn)?.reply()?;

    let property = conn
        .get_property(
            false,
            window_id,
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

        acc || (net_wm_state_atom == atoms._NET_WM_STATE_FULLSCREEN)
    }))
}

fn save_transparent_image(
    image_data: &ImageData,
    icon_path: &str,
) -> anyhow::Result<()> {
    let img = RgbaImage::from_raw(
        image_data.width,
        image_data.height,
        image_data.buf.to_vec(),
    )
    .unwrap();

    img.save(icon_path)?;

    Ok(())
}

fn save_filled_image(
    image_data: &ImageData,
    icon_path: &str,
    color: &str,
) -> anyhow::Result<()> {
    let bg_color = color[1..]
        .chars()
        .collect::<Vec<_>>()
        .chunks_exact(2)
        .map(|x| u8::from_str_radix(&format!("{}{}", x[0], x[1]), 16).unwrap())
        .collect::<Vec<u8>>();

    let (bg_r, bg_g, bg_b) = (bg_color[0], bg_color[1], bg_color[2]);

    let mut new_img = vec![
        0u8;
        (image_data.width * image_data.height * 3)
            .try_into()
            .unwrap()
    ];
    let buf = &image_data.buf;

    let mut i = 0;
    let mut j = 0;

    // Fill background color, for more information, see here:
    // https://stackoverflow.com/questions/2049230/convert-rgba-color-to-rgb
    for _chunk in buf.chunks(4) {
        let (r, g, b, a) =
            (buf[j], buf[j + 1], buf[j + 2], (buf[j + 3] as f64) / 255.0);

        new_img[i] = (((1.0 - a) * (bg_r as f64)) + a * (r as f64)) as u8;
        new_img[i + 1] = (((1.0 - a) * (bg_g as f64)) + a * (g as f64)) as u8;
        new_img[i + 2] = (((1.0 - a) * (bg_b as f64)) + a * (b as f64)) as u8;

        i += 3;
        j += 4;
    }

    image::save_buffer(
        Path::new(icon_path),
        &new_img,
        image_data.width,
        image_data.height,
        image::ColorType::Rgb8,
    )?;

    Ok(())
}

pub fn generate_icon(
    icon_name: &str,
    cache_dir: &Path,
    color: &str,
    window_id: u32,
) -> anyhow::Result<()> {
    let (conn, _) = x11rb::connect(None)?;
    let atoms = AtomCollection::new(&conn)?.reply()?;

    let property = conn
        .get_property(
            false,
            window_id,
            atoms._NET_WM_ICON,
            AtomEnum::CARDINAL,
            0,
            u32::MAX,
        )?
        .reply()?;

    let data = property.value;
    let mut icons = Vec::new();
    let mut i = 0;

    while i < data.len() {
        let mut chunks = data[i..].chunks_exact(4);

        let width =
            u32::from_le_bytes(chunks.next().unwrap().try_into().unwrap());
        let height =
            u32::from_le_bytes(chunks.next().unwrap().try_into().unwrap());

        i += 8;

        let size = usize::try_from(width * height * 4).unwrap();

        if i + size > data.len() {
            break;
        } else {
            let mut buf = data[i..i + size].to_vec();

            buf.chunks_exact_mut(4).for_each(|chunk| {
                let (c0, c2) = (chunk[0], chunk[2]);
                chunk[2] = c0;
                chunk[0] = c2;
            });

            i += size;
            icons.push(ImageData { width, height, buf });
        }
    }

    let icon_path = format!(
        "{}/{}.jpg",
        cache_dir.to_string_lossy().to_string(),
        icon_name,
    );

    let mut max_size = 0;
    let mut max_icon = None;

    for icon in icons.iter() {
        if icon.width > max_size {
            max_size = icon.width;
            max_icon = Some(icon);
        }
    }

    match max_icon {
        Some(icon) => save_filled_image(icon, &icon_path, color),
        None => bail!("No icon was found for this window"),
    }
}

fn composite_manager_running(
    conn: &impl Connection,
    screen_num: usize,
) -> anyhow::Result<bool> {
    let atom = format!("_NET_WM_CM_S{}", screen_num);
    let atom = conn.intern_atom(false, atom.as_bytes())?.reply()?.atom;
    let owner = conn.get_selection_owner(atom)?.reply()?;
    Ok(owner.owner != x11rb::NONE)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bspc::selectors::NodeSelector;
    use bspc_rs as bspc;
    use std::env;

    #[test]
    fn test_get_wm_class() {
        let id =
            bspc::query_nodes(None, None, None, Some(NodeSelector("focused")))
                .unwrap()[0];
        let wm_class = get_wm_class(id).unwrap();

        println!("{wm_class}");
    }

    #[test]
    fn test_get_current_wm() {
        let wm = get_current_wm().unwrap();

        println!("wm: {wm}");
    }

    #[test]
    fn test_is_window_fullscreen() {
        let id =
            bspc::query_nodes(None, None, None, Some(NodeSelector("focused")))
                .unwrap()[0];
        let flag = is_window_fullscreen(id).unwrap();

        println!("flag: {flag}");
    }

    #[test]
    #[ignore]
    fn test_generate_icon() {
        let id =
            bspc::query_nodes(None, None, None, Some(NodeSelector("focused")))
                .unwrap()[0];

        generate_icon("foo.jpg", "/home/andrey", "#252737", id).unwrap();
    }

    fn get_icon_path() -> String {
        env::current_dir().unwrap().to_str().unwrap().to_owned()
            + "/tests_files/alacritty.png"
    }

    #[test]
    fn display_icon_test() {
        let (conn, _) = x11rb::connect(None).unwrap();
        let monitor_name = "eDP-1";
        display_icon(&conn, &get_icon_path(), 270, 6, 24, monitor_name)
            .unwrap();
    }
}
