use crossbeam::channel::{Receiver};
use minifb::*;
use crate::config_file::ConfigFile;
use crate::constants::{DEFAULT_MAGNIFICATION, HIRES_HEIGHT, HIRES_WIDTH};
use crate::messages::DrawCommand::Rectangle;
use crate::messages::ToMiniFb;

pub fn main_minifb(receiver: Receiver<ToMiniFb>, config: &ConfigFile) {
    let mag = config.magnification();
    let width: usize = (HIRES_WIDTH * mag) as usize;
    let height: usize = (HIRES_HEIGHT * mag) as usize;

    let mut buffer: Vec<u32> = vec![0; width * height];

    let mut window = Window::new(
        "Apple ][ emulator on minifb",
        width,
        height,
        WindowOptions::default(),
    )
        .unwrap_or_else(|e| {
            panic!("{}", e);
        });

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // We unwrap here as we want this code to exit if it fails. Real applications may want
        // to handle this in a different way
        window
            .update_with_buffer(&buffer, width, height)
            .unwrap();

        while ! receiver.is_empty() {
            match receiver.recv() {
                Ok(message) => {
                    for it in buffer.iter_mut() {
                        *it = 0;
                    }
                    match message {
                        ToMiniFb::Buffer(draw_commands) => {
                            for dc in draw_commands {
                                match dc {
                                    Rectangle(x0, y0, x1, y1, color) => {
                                        let x0 = x0 as u16;
                                        let y0 = y0 as u16;
                                        let x1 = x1 as u16;
                                        let y1 = y1 as u16;
                                        for x in x0..x1 {
                                            for y in y0..y1 {
                                                let address = y as usize * width + x as usize;
                                                let (r, g, b) = color.to_rgb();
                                                buffer[address] = ((r as u32) << 16)
                                                    | ((g as u32) << 8)
                                                    | (b as u32);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => { println!("Received error: {}", e); }
            }
        }
    }
}