use std::{rc::Rc, cell::RefCell};
use crate::{Tekenen};

pub struct Terminal {
    buffer: Rc<RefCell<String>>,
}

impl Terminal {
    pub fn new() -> Terminal {
        Terminal {
            buffer: Rc::new(RefCell::new(String::new())),
        }
    }

    pub fn write(&self, c: char) {
        self.buffer.borrow_mut().push(c)
    }
}

impl Terminal {
    pub fn render(&self, renderer: &Tekenen, time: u64) {
        renderer.draw_terminal(&self.buffer.borrow(), time);
    }
}
