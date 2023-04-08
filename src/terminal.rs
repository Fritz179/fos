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

// pub struct EchoProgram {
//     pid: Pid,
// }

// impl Process for EchoProgram {
//     fn new(pid: Pid) -> EchoProgram {
//         EchoProgram { 
//             pid
//         }
//     }

//     fn main(self: Rc<Self>, fs: &Fs) {
//         fs.write(self.pid, descriptor, c)
//     }
// }


// -	Regular or ordinary file
// d	Directory file
// l	Link file
// b	Block special file => buffered access, chunks of data
// p	Named pipe file => interproces communication
// c	Character special file => direct access, byte by byte
// s	Socket file => ip:socket