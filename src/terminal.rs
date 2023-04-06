use std::{rc::Rc, cell::RefCell};
use crate::{Tekenen, Pid, Fs, Process};

pub struct Terminal {
    buffer: Rc<RefCell<String>>,
    pid: Pid,
}

impl Process for Terminal {
    fn new(pid: Pid) -> Terminal {
        Terminal { 
            buffer: Rc::new(RefCell::new(String::new())),
            pid
        }
    }

    fn main(&self, fs: &Fs) {
        let buffer = Rc::clone(&self.buffer);
        fs.read(self.pid, 0, Box::new(move |c| {
            buffer.borrow_mut().push(c)
        }));
    }
}

impl Terminal {
    pub fn render(&self, renderer: &Tekenen, time: u64) {
        renderer.draw_terminal(&self.buffer.borrow(), time);
    }
}