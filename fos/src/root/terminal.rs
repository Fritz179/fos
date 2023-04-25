use crate::Tekenen;
use std::cell::RefCell;

pub struct Terminal {
    buffer: RefCell<String>,
}

impl Terminal {
    pub fn new() -> Terminal {
        Terminal {
            buffer: RefCell::new(String::new()),
        }
    }

    pub fn write(&self, c: char) {
        self.buffer.borrow_mut().push(c)
    }
}

impl Terminal {
    pub fn render(&self, renderer: &mut Tekenen, time: u64) {
        renderer.draw_terminal(&self.buffer.borrow(), time);
    }
}
