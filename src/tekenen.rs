mod sdl_renderer;
mod font;

pub use font::{FONT, FIRST_CHAR, LAST_CHAR};

pub use sdl2::{event::Event, keyboard::{Keycode, Mod}};
use sdl_renderer::SDLRenderer;

use std::{cell::{RefCell, Ref}, rc::{Weak, Rc}};


pub type Pixels = Vec<u8>;

pub mod colors;
use colors::Pixel;

pub struct Tekenen {
    app: RefCell<Option<Weak<RefCell<dyn AppTrait>>>>,
    renderer: RefCell<Box<dyn RendererTrait>>,
    pixels: RefCell<Pixels>,
    width: usize,
    height: usize
}

// To be implemented by the running App
pub trait AppTrait {
    fn update(&mut self, time :u64);
    fn event_handler(&mut self, event: Event) -> bool;
    fn new(renderer: Rc<Tekenen>) -> Rc<RefCell<Self>> where Self: Sized;
}

// Available to app
impl Tekenen {
    pub fn new(width: usize, height: usize) -> Rc<Tekenen> {
        let renderer = RefCell::new(SDLRenderer::new(width, height, "Salve"));

        let tekenen = Rc::new(Tekenen {
            app: RefCell::new(None),
            renderer,
            pixels: RefCell::new(vec![0; width * height * 4]),
            
            width,
            height
        });

        tekenen.renderer.borrow_mut().set_tekenen(Rc::clone(&tekenen));

        return tekenen;
    }

    pub fn new_app<AppStruct: AppTrait + 'static>(tekenen: &Rc<Tekenen>) -> Rc<RefCell<AppStruct>> {
        let appref = AppStruct::new(Rc::clone(&tekenen));

        let traitclone: Rc<RefCell<dyn AppTrait>> = Rc::clone(&appref) as Rc<RefCell<dyn AppTrait>>;
        *tekenen.app.borrow_mut() = Some(Rc::downgrade(&traitclone));

        return appref
    }

    pub fn start(&self, fps: u32) {
        self.renderer.borrow_mut().start(fps)
    }
}

// To be implemented by the renderer
pub trait RendererTrait {
    fn new(width: usize, height: usize, name: &str) -> Box<Self> where Self: Sized;
    fn set_tekenen(&mut self, tekenen: Rc<Tekenen>);
    fn start(&mut self, fps: u32);
}

// Available for the renderer
impl Tekenen {
    fn get_app(&self) -> Rc<RefCell<dyn AppTrait>> {
        let mut op = self.app.borrow_mut();
        if let Some(a) = op.as_mut() {
            let a = a.upgrade().expect("No app");
            return a;
        } else {
            panic!("No app")
        }
    }

    pub fn update(&self, time: u64) {
        let app = self.get_app();

        let now = std::time::SystemTime::now();

        app.borrow_mut().update(time);

        match now.elapsed() {
            Ok(elapsed) => {
                let mut text = "Update time: ".to_owned();
                text.push_str(&elapsed.as_micros().to_string());

                self.draw_text(&text, 450, 600 - 24);
            }
            Err(e) => {
                println!("Error: {e:?}");
            }
        }
    }

    pub fn event_handler(&self, event: Event) -> bool {
        let app = self.get_app();

        return app.borrow_mut().event_handler(event);
    }

    pub fn get_pixels(&self) -> Ref<Pixels> {
        return self.pixels.borrow()
    }
}

// Drawing implementation
impl Tekenen {
    pub fn pixel_index(&self, x: i32, y: i32) -> Option<usize> {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return None
        }

        return Some((y * self.width as i32 + x) as usize)
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

        for x in x .. x + w {
            for y in y .. y + h {
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
                    continue
                }
            }

            // skip whitespace
            if char == ' ' {
                curr_x += FONT_SIZE;
                continue
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
                        continue
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

        return (curr_x, curr_y)
    }

    pub fn draw_terminal(&self, buffer: &String, time: u64) {
        let (x, y) = self.draw_text(buffer, 0, 0);

        const BLINKING_TIME: u64 = 500;

        if time % BLINKING_TIME > BLINKING_TIME / 2 {
            self.rect(x, y, 16, 16, colors::WHITE)
        }   
    }
}

pub trait AppTraits {
    fn update(&mut self, tekenen: &mut Tekenen);
    fn event_handler(&mut self, event: Event) -> bool;
}