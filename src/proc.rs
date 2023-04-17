use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use debug_cell::Ref;

use crate::{FileDescriptor, Fs, Table, future::Executor};

pub type Pid = u32;

pub trait Process {
    fn new(proc: Proc) -> Self
    where
        Self: Sized;
}

use crate::channel::{Tx, Rx, new_channel};

type Fd = (Rc<Tx<char>>, Option<Rx<char>>);

pub struct Proc {
    pub fs: Weak<Fs>,
    pid: Pid,
    children: RefCell<Vec<Rc<dyn Process>>>,
    pub spawner: Weak<Spawner>,
    descriptor_table: Table<Fd>
    // descriptorMap
}

impl Proc {
    fn new(pid: Pid, fs: Weak<Fs>, spawner: Weak<Spawner>) -> Self {
        Proc {
            pid,
            fs,
            spawner,
            children: RefCell::new(vec![]),
            descriptor_table: Table::new()
        }
    }

    pub fn spawn<Child: Process + 'static>(&self) -> (Rc<Child>, Pid) {
        let spawner = self.spawner.upgrade().expect("No Spawner");
        let mut children = self.children.borrow_mut();

        let (child, pid) = spawner.spawn::<Child>();
        let child_clone = Rc::clone(&child);
        children.push(child_clone);

        return (child, pid);
    }

    pub fn open(&self) -> FileDescriptor {
        let (tx, rx) = new_channel();
        let channel = (tx, Some(rx));

        let id = self.descriptor_table.add(channel);

        return id as FileDescriptor;
    }

    pub async fn read(&self, descriptor: FileDescriptor) -> Option<char> {
        let fd = self.descriptor_table.get(descriptor);

        let rx = std::cell::Ref::map(fd, |f| &f.1);

        if let Some(ref rx) = *rx {
            println!("Reading: {descriptor}");

            return rx.read().await
        } else {
            None
        }
    }

    pub fn write(&self, descriptor: FileDescriptor, char: char) -> Option<()> {
        let fd = self.descriptor_table.get(descriptor);

        println!("Writng: {descriptor}, {char}");

        let tx = std::cell::Ref::map(fd, |f| &f.0);
        tx.send(char)
    }
}

#[derive(Debug)]
pub struct Spawner {
    processes: Table<Weak<dyn Process>>,
    fs: Rc<Fs>,
    pub executor: Rc<Executor>,
}

impl Spawner {
    pub fn new(fs: Rc<Fs>, executor: Rc<Executor>) -> Self {
        Spawner {
            processes: Table::new(),
            fs,
            executor
        }
    }

    pub fn spawn<Child: Process + 'static>(self: &Rc<Self>) -> (Rc<Child>, Pid) {
        let processes = &self.processes;

        let child_pid = processes.next_free() as Pid;
        let child_proc = Proc::new(child_pid, Rc::downgrade(&self.fs), Rc::downgrade(self));

        self.fs.add_pid(child_pid);

        child_proc.open(); // stdin
        child_proc.open(); // stdout
        child_proc.open(); // stderr

        let child = Rc::new(Child::new(child_proc));
        let id = processes.add(Rc::downgrade(&child) as Weak<dyn Process>);

        assert_eq!(child_pid, id as u32);

        drop(processes);

        return (child, child_pid);
    }
}
