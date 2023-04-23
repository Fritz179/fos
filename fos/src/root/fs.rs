use std::{fmt, cell::RefCell};
// use std::cell::RefCell;

use crate::Pid;

pub type FileDescriptor = usize;
pub const STDIN: FileDescriptor = 0;
pub const STDOUT: FileDescriptor = 1;
pub const STDERR: FileDescriptor = 2;

pub struct Fs {
    pid_map: RefCell<Vec<Vec<FileDescriptor>>>,
}

impl Fs {
    pub fn new() -> Self {
        Fs {
            pid_map: RefCell::new(vec![]),
        }
    }

    pub fn add_pid(&self, pid: Pid) {
        let mut pid_map = self.pid_map.borrow_mut();
        pid_map.push(vec![]);

        assert!(pid_map.len() > pid as usize, "Invalid pid_map length")
    }
}

impl fmt::Debug for Fs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Fs")
            //  .field("x", &self.x)
            //  .field("y", &self.y)
            .finish()
    }
}

// pipes -> multiple sender, single reciver
// using Futures -> read one char per Future
// open also has await? Performance = more state

// -	Regular or ordinary file
// d	Directory file
// l	Link file
// b	Block special file => buffered access, chunks of data
// p	Named pipe file => interproces communication
// c	Character special file => direct access, byte by byte
// s	Socket file => ip:socket