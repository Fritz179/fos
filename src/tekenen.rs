mod sdl_renderer;
mod font;

use font::{FONT, FIRST_CHAR};

pub use sdl2::{event::Event, keyboard::Keycode};
use sdl_renderer::SDLRenderer;

use std::{cell::{RefCell, Ref}, rc::{Weak, Rc}};

pub type Pixel = [u8; 4];
// pub type Pixels = Vec<Pixel>;
pub type Pixels = Vec<u8>;

pub struct Tekenen {
    app: RefCell<Option<Weak<RefCell<dyn AppTrait>>>>,
    renderer: RefCell<Box<dyn RendererTrait>>,
    pixels: RefCell<Pixels>,
    width: usize,
    height: usize
}

// To be implemented by the running App
pub trait AppTrait {
    fn update(&mut self);
    fn event_handler(&mut self, event: Event) -> bool;
    fn new(renderer: Rc<Tekenen>) -> Self where Self: Sized;
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

        let app = AppStruct::new(Rc::clone(&tekenen));
        let appref = Rc::new(RefCell::new(app));

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

    pub fn update(&self) {
        let app = self.get_app();

        let now = std::time::SystemTime::now();

        app.borrow_mut().update();

        match now.elapsed() {
            Ok(elapsed) => {
                let mut text = "Update time: ".to_owned();
                text.push_str(&elapsed.as_micros().to_string());

                self.draw_text(&text, 600, 600 - 16)
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
    pub fn draw_text(&self, text: &str, x: i32, y: i32) {
        let mut pixels = self.pixels.borrow_mut();
        let mut pos = 0;

        for char in text.chars() {

            // skip whitespace
            if char == ' ' {
                pos += 8;
                continue
            }

            // get data by finding offset in charset
            let data = FONT[char as usize - FIRST_CHAR as usize];

            // println!("{}, {}", char, char as usize - FIRST_CHAR as usize);
            // panic!();

            for (yd, line) in data.iter().enumerate() {
                for (xd, symbol) in line.iter().enumerate() {
                    if *symbol == ' ' {
                        continue
                    }

                    self.set_pixel(&mut pixels, x + pos + xd as i32, y + yd as i32, [255, 255, 255, 255]);
                }
            }

            // increment for next character
            pos += 8;
        }
    }
}

pub trait AppTraits {
    fn update(&mut self, tekenen: &mut Tekenen);
    fn event_handler(&mut self, event: Event) -> bool;
}