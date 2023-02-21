use std::error::Error;

use image::io::Reader;
use image::GenericImageView;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;
use x11rb::protocol::Event;

pub fn display_image(
    image_path: &str,
    x: u16,
    y: u16,
    size: u16,
) -> Result<(), Box<dyn Error>> {
    let image = Reader::open(image_path)?.decode()?;
    let (width, height) = image.dimensions();

    let (conn, screen_num) = x11rb::connect(None)?;
    let screen = &conn.setup().roots[screen_num];

    let win = conn.generate_id()?;
    let window_aux = CreateWindowAux::new()
        .border_pixel(screen.black_pixel)
        .event_mask(EventMask::EXPOSURE);

    create_window(
        &conn,
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

    let data = image.into_rgba8().into_raw();
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

    conn.map_window(win)?;
    conn.flush()?;

    loop {
        let event = conn.wait_for_event()?;

        if let Event::Expose(_) = event {
            conn.flush()?;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_image_works() {
        display_image("/home/andrey/alacritty.png", 200, 200, 200);
    }
}
