use std::{fmt, collections::VecDeque, rc::Rc, cell::RefCell};
// use std::cell::RefCell;

use crate::Pid;

pub type FileDescriptor = u32;
pub const STDIN: FileDescriptor = 0;
pub const STDOUT: FileDescriptor = 1;
pub const STDERR: FileDescriptor = 2;

type Readers = Vec<Box<dyn Fn(char)>>;

pub struct Fs {
    readers_map: RefCell<Vec<Readers>>,
    pid_map: RefCell<Vec<Vec<FileDescriptor>>>,
}

impl Fs {
    pub fn new() -> Self {
        Fs {
            readers_map: RefCell::new(vec![]),
            pid_map: RefCell::new(vec![]),
        }
    }

    pub fn add_pid(&self, pid: Pid) {
        let mut pid_map = self.pid_map.borrow_mut();
        pid_map.push(vec![]);

        assert!(pid_map.len() > pid as usize, "Invalid pid_map length")
    }

    pub fn open(&self, pid: Pid) -> FileDescriptor {
        let mut readers_map = self.readers_map.borrow_mut();

        let raw_descriptor = readers_map.len() as u32;
        readers_map.push(vec![]);

        let mut pid_map = self.pid_map.borrow_mut();

        let pid_mapping = pid_map.get_mut(pid as usize).expect("No PID mapping");
        let file_id = pid_mapping.len() as FileDescriptor;
        pid_mapping.push(raw_descriptor);

        return file_id;
    }

    pub fn read(&self, pid: Pid, descriptor: FileDescriptor, callback: Box<dyn Fn(char)>) {
        let pid_map = self.pid_map.borrow();
        let pid_map = pid_map.get(pid as usize).expect("No PID mapping");
        let raw = *pid_map.get(descriptor as usize).expect("No descriptor");

        let mut readers = self.readers_map.borrow_mut();
        let readers = readers.get_mut(raw as usize).expect("No raders");

        readers.push(callback);
    }

    pub fn write(&self, pid: Pid, descriptor: FileDescriptor, char: char) {
        let mut raw = 0;

        {
            let pid_map = self.pid_map.borrow();
            let pid_map = pid_map.get(pid as usize).expect("No PID mapping");
            raw = *pid_map.get(descriptor as usize).expect("No descriptor");
        }

        let readers = self.readers_map.borrow();
        let readers = readers.get(raw as usize).expect("No raders");

        for reader in readers.iter() {
            reader(char);
        }
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

struct Shared<T> {
    buffer: RefCell<VecDeque<T>>
}

struct Tx<T> {
    shared: Rc<Shared<T>>
}

impl<T> Tx<T> {
    fn send(&self, data: T) {
        self.shared.buffer.borrow_mut().push_back(data);
    }
}

struct Rx<T> {
    shared: Rc<Shared<T>>
}

use crate::future::{Future, Poll, Context};
use std::pin::Pin;

impl<T> Future for Rx<T> {
    type Output = T;
    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<T> {
        let mut buffer = self.shared.buffer.borrow_mut();
        let data = buffer.pop_front();
        drop(buffer);

        if let Some(data) = data {
            Poll::Ready(data)
        } else {
            Poll::Pending
        }
    }
}

fn create_pipe<T>() -> (Tx<T>, Rx<T>) {
    let shared = Rc::new(
        Shared {
            buffer: RefCell::new(VecDeque::new()),
        }
    );

    let tx = Tx {
        shared: Rc::clone(&shared)
    };

    let rx = Rx {
        shared
    };

    return (
        tx,
        rx
    )
}