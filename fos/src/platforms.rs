pub mod font;
pub mod tekenen;
pub use tekenen::Tekenen;

pub use sdl2::keyboard::Keycode;

#[derive(Debug)]
pub struct Keymod {
    pub shift: bool,
    pub ctrl: bool,
    pub caps: bool,
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

pub trait PlatformTrait {
    fn new(width: u32, height: u32) -> Box<Self> where Self: Sized;
    fn display_pixels(&mut self, pixels: &tekenen::Pixels);
    fn read_events(&mut self) -> Option<Event>;
    fn set_interval(callback: &mut dyn FnMut() -> bool, fps: u32) where Self: Sized;
}