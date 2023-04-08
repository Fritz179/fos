use std::{rc::{Rc, Weak}, cell::RefCell};

use crate::{Fs, FileDescriptor};

pub type Pid = u32;

pub trait Process {
    fn new(proc: Proc) -> Self where Self: Sized;
}

pub struct Proc {
    pid: Pid,
    pub fs: Weak<Fs>,
    children: RefCell<Vec<Rc<dyn Process>>>,
    spawner: Weak<Spawner>,
}

impl Proc {
    fn new(pid: Pid, fs: Weak<Fs>, spawner: Weak<Spawner>) -> Self {
        Proc {
            pid, 
            fs,
            spawner,
            children: RefCell::new(vec![])
        }
    }

    pub fn spawn<Child: Process + 'static>(&self) -> (Rc<Child>, Pid) {
        let spawner = self.spawner.upgrade().expect("No Spawner");
        let mut children = self.children.borrow_mut();

        let (child, pid) = spawner.spawn::<Child>();
        let child_clone = Rc::clone(&child);
        children.push(child_clone);

        return (child, pid)
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

    pub fn spawn<Child: Process + 'static>(self: &Rc<Self>) -> (Rc<Child>, Pid) {
        let mut processes = self.processes.borrow_mut();

        let child_pid = processes.len() as Pid;
        let child_proc = Proc::new(child_pid, Rc::downgrade(&self.fs), Rc::downgrade(self));

        self.fs.add_pid(child_pid);

        child_proc.open(); // stdin
        child_proc.open(); // stdout
        child_proc.open(); // stderr

        let child = Rc::new(Child::new(child_proc));
        processes.push(Rc::downgrade(&child) as Weak<dyn Process>);

        drop(processes);

        return (child, child_pid)
    }
}