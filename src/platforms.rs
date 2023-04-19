pub mod font;
pub mod tekenen;

use sdl2::keyboard::Keycode;

#[derive(Debug)]
pub struct Keymod {
    shift: bool,
    ctrl: bool,
    caps: bool,
}

#[derive(Debug)]
pub enum Event {
    KeyDown {
        repeat: bool,
        char: Option<char>,
        keycode: Keycode,
        keymod: Keymod,
    },
    Quit,
}

pub trait Platform {
    fn new(width: u32, height: u32) -> Box<Self>;
    fn display_pixels(&mut self, pixels: &tekenen::Pixels);
    fn read_events(&mut self) -> Option<Event>;
    fn set_interval(callback: &mut dyn FnMut() -> bool, fps: u32);
}


// sdl2
mod sdl_platform;
pub use self::sdl_platform::*;