use super::{colors, colors::Pixel, font};
pub use font::{FIRST_CHAR, FONT, LAST_CHAR};

use std::cell::RefCell;

pub type Pixels = Vec<u8>;

pub struct Tekenen {
    pub pixels: RefCell<Pixels>,
    width: usize,
    height: usize,
}

impl Tekenen {
    pub fn new(width: usize, height: usize) -> Tekenen {
        Tekenen {
            pixels: RefCell::new(vec![0; width * height * 4]),
            width,
            height,
        }
    }
}

// Drawing implementation
impl Tekenen {
    pub fn pixel_index(&self, x: i32, y: i32) -> Option<usize> {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return None;
        }

        return Some((y * self.width as i32 + x) as usize);
    }

    pub fn set_pixel(&self, pixels: &mut Pixels, x: i32, y: i32, color: Pixel) {
        if let Some(index) = self.pixel_index(x, y) {
            // self.pixels.borrow_mut()[index] = color;
            pixels[index * 4 + 0] = color[0];
            pixels[index * 4 + 1] = color[1];
            pixels[index * 4 + 2] = color[2];
            pixels[index * 4 + 3] = color[3];
        }
    }

    #[allow(dead_code)]
    pub fn rect(&self, x: i32, y: i32, w: i32, h: i32, color: Pixel) {
        let mut pixels = self.pixels.borrow_mut();

        for x in x..x + w {
            for y in y..y + h {
                self.set_pixel(&mut pixels, x, y, color);
            }
        }
    }

    #[allow(dead_code)]
    pub fn background(&self, color: Pixel) {
        let mut pixels = self.pixels.borrow_mut();

        for x in 0..self.width {
            for y in 0..self.height {
                self.set_pixel(&mut pixels, x as i32, y as i32, color);
            }
        }
    }

    #[allow(dead_code)]
    pub fn draw_text(&self, text: &str, x: i32, y: i32) -> (i32, i32) {
        const FONT_SCALE: i32 = 2;
        const FONT_SIZE: i32 = 8 * FONT_SCALE;

        let mut pixels = self.pixels.borrow_mut();
        let mut curr_x = 0;
        let mut curr_y = 0;

        for char in text.chars() {
            if curr_x >= 800 || char == '\n' {
                curr_x = 0;
                curr_y += FONT_SIZE;

                if char == '\n' {
                    continue;
                }
            }

            // skip whitespace
            if char == ' ' {
                curr_x += FONT_SIZE;
                continue;
            }

            // get data by finding offset in charset
            let data = FONT[char as usize - FIRST_CHAR as usize];

            // println!("{}, {}", char, char as usize - FIRST_CHAR as usize);
            // panic!();

            for (yd, line) in data.iter().enumerate() {
                let y = y + yd as i32 * FONT_SCALE + curr_y;

                for (xd, symbol) in line.iter().enumerate() {
                    let x = x + xd as i32 * FONT_SCALE + curr_x;

                    if *symbol == ' ' {
                        continue;
                    }

                    for xf in 0..FONT_SCALE {
                        for yf in 0..FONT_SCALE {
                            self.set_pixel(&mut pixels, x + xf, y + yf, colors::WHITE);
                        }
                    }
                }
            }

            // increment for next character
            curr_x += FONT_SIZE;
        }

        return (curr_x, curr_y);
    }

    pub fn draw_terminal(&self, buffer: &String, time: u64) {
        let (x, y) = self.draw_text(buffer, 0, 0);

        const BLINKING_TIME: u64 = 500;

        if time % BLINKING_TIME > BLINKING_TIME / 2 {
            self.rect(x, y, 16, 16, colors::WHITE)
        }
    }
}
