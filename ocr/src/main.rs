#![allow(clippy::single_match)]

use minifb::{Key, MouseButton, MouseMode, Window, WindowOptions};
use screenshots::Screen;

const WIDTH: usize = 640;
const HEIGHT: usize = 360;

fn main() -> anyhow::Result<()> {
    println!("{:#?}", Screen::all());
    // let current_screen = 

    let screen = Screen::from_point(0, 0)?;
    let img = screen.capture_area(0, 0, WIDTH as u32, HEIGHT as u32)?;

    let (width, height) = img.dimensions();
    let buffer: Vec<u32> = img
        .into_raw()
        .chunks_exact(4)
        .map(|p| {
            ((p[3] as u32) << 24) | ((p[0] as u32) << 16) | ((p[1] as u32) << 8) | (p[2] as u32)
        })
        .collect();

    let mut window = Window::new(
        "Screenshot",
        WIDTH,
        HEIGHT,
        WindowOptions {
            title: false,
            borderless: false,
            topmost: true,
            ..WindowOptions::default()
        },
    )?;

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    while window.is_open() && !window.is_key_down(Key::Escape) {
        if let Some((x, y)) = window.get_mouse_pos(MouseMode::Discard) {
            // println!("Mouse Position: x: {}, y: {}", x, y);
        }

        // Get mouse button state
        if window.get_mouse_down(MouseButton::Left) {
            println!("Mouse Button Left Down");
        }

        window.update_with_buffer(&buffer, width as usize, height as usize)?;
    }

    Ok(())
}
