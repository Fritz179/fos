use std::{rc::{Rc, Weak}, cell::RefCell};

use crate::{Fs, FileDescriptor};

pub type Pid = u32;

pub trait Process {
    fn new(proc: Proc) -> Self where Self: Sized;
    fn main(self: Rc<Self>);
}

pub struct Proc {
    pid: Pid,
    fs: Weak<Fs>,
    spawner: Weak<Spawner>,
}

impl Proc {
    fn new(pid: Pid, fs: Weak<Fs>, spawner: Weak<Spawner>) -> Self {
        Proc {
            pid, 
            fs,
            spawner,
        }
    }

    pub fn open(&self) -> FileDescriptor {
        let fs = self.fs.upgrade().expect("No FS found");

        return fs.open(self.pid);
    }

    pub fn read(&self, descriptor: FileDescriptor, callback: Box<dyn Fn(char)>) {
        let fs = self.fs.upgrade().expect("No FS found");

        fs.read(self.pid, descriptor, callback);
    }

    pub fn write(&self, descriptor: FileDescriptor, char: char) {
        let fs = self.fs.upgrade().expect("No FS found");

        fs.write(self.pid, descriptor, char);
    }
}

#[derive(Debug)]
pub struct Spawner {
    processes: RefCell<Vec<Weak<dyn Process>>>,
    fs: Rc<Fs>
}

impl Spawner {
    pub fn new(fs: Rc<Fs>) -> Self {
        Spawner {
            processes: RefCell::new(vec![]),
            fs
        }
    }

    pub fn spawn<Child: Process + 'static>(self: &Rc<Self>) -> Rc<Child> {
        let mut processes = self.processes.borrow_mut();

        let child_pid = processes.len() as Pid;
        let child_proc = Proc::new(child_pid, Rc::downgrade(&self.fs), Rc::downgrade(self));

        self.fs.add_pid(child_pid);

        child_proc.open(); // stdin
        child_proc.open(); // stdout
        child_proc.open(); // stderr

        let child = Rc::new(Child::new(child_proc));
        processes.push(Rc::downgrade(&child) as Weak<dyn Process>);

        Rc::clone(&child).main();

        return child
    }
}