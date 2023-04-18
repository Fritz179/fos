use crate::Tekenen;
use std::{cell::RefCell, rc::Rc};

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
    pub fn render(&self, renderer: &mut Tekenen, time: u64) {
        renderer.draw_terminal(&self.buffer.borrow(), time);
    }
}
