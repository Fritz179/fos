use std::{rc::{Rc, Weak}, cell::RefCell};
use crate::{STD_IN, Tekenen, AppManager};

pub struct Terminal {
    buffer: RefCell<String>,
    parent: Weak<RefCell<AppManager>>,
}

impl Terminal {
    pub fn new(parent: Weak<RefCell<AppManager>>) -> Terminal {
        Terminal { 
            buffer: RefCell::new(String::new()),
            parent,
        }
    }

    fn exec(&self) {
        println!("Hello there!")
    }

    pub fn main(&self, weak_self: Weak<Self>) {

        self.parent.upgrade().expect("No parent").borrow_mut().read(STD_IN, Box::new(move | c | {

            let this = weak_self.upgrade().expect("msg");
            this.buffer.borrow_mut().push(c);

            if c == '\n' {
                this.exec();
            }
        }))
    }

    pub fn render(&self, renderer: &Rc<Tekenen>, time: u64) {
        renderer.draw_terminal(&self.buffer.borrow(), time);
    }
}
