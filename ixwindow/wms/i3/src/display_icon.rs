use std::error::Error;

use image::imageops::FilterType;
use image::io::Reader;
use image::GenericImageView;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;
use x11rb::protocol::Event;

pub fn display_icon(
    image_path: &str,
    x: u16,
    y: u16,
    size: u16,
    monitor: &str,
) -> Result<(), Box<dyn Error>> {
    let image = Reader::open(image_path)?.decode()?;
    let image = image.resize(size as u32, size as u32, FilterType::CatmullRom);
    let (width, height) = image.dimensions();

    let (conn, screen_num) = x11rb::connect(Some(monitor))?;
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
    use super::*;

    #[test]
    fn display_icon_works() {
        display_icon("/home/andrey/alacritty.png", 270, 6, 24);
    }
}
