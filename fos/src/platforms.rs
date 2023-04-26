pub mod font;
pub mod tekenen;
pub use tekenen::Tekenen;

#[derive(Debug)]
pub enum Keycode {
    Temp
}

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
    fn set_interval(callback: Box<dyn FnMut() -> bool>, fps: u32) where Self: Sized;
}