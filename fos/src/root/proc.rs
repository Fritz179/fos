// Every parent has a Rc to it's children
// Root has an Rc to every process
// Every child has a Weak to it's parent

// Every process has a root which has the usefull all functions

use std::{
    cell::RefCell,
    rc::Rc,
};

use crate::{FileDescriptor, fc::table::Table, Root};

pub type Pid = u32;

pub trait Process {
    fn new(proc: Proc) -> Self
    where
        Self: Sized;
    
    fn get_process_name(&self) -> &str;

    fn get_proc(&self) -> &Proc;
}

use crate::fc::channel::{Tx, Rx, new_channel};

type Fd = (Rc<Tx<char>>, Option<Rx<char>>);

pub struct Proc {
    pub pid: Pid,
    pub root: Rc<Root>,
    pub children: RefCell<Vec<Rc<dyn Process>>>,
    pub descriptor_table: Table<Fd>
}

impl std::fmt::Debug for Proc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Proc")
            // .field("root", self.root)
            .finish()
    }
}

impl Proc {
    pub fn new(pid: Pid, root: Rc<Root>) -> Self {
        Proc {
            pid,
            root,
            children: RefCell::new(vec![]),
            descriptor_table: Table::new()
        }
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
            // println!("Reading: {descriptor}");

            return rx.read().await
        } else {
            None
        }
    }

    pub fn write(&self, descriptor: FileDescriptor, char: char) -> Option<()> {
        let fd = self.descriptor_table.get(descriptor);

        // println!("Writng: {descriptor}, {char}");

        let tx = std::cell::Ref::map(fd, |f| &f.0);
        tx.send(char)
    }

    pub fn exit(&self) {
        // Exit?
    }
}