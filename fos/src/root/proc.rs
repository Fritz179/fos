// Every parent has a Rc to it's children
// Root has an Rc to every process
// Every child has a Weak to it's parent

// Every process has a root which has the usefull all functions

use std::{
    cell::RefCell,
    rc::Rc,
};

use crate::{fc::table::Table, FileDirectoryPipe};

pub type Pid = u32;

pub trait Process {
    fn new(proc: Proc) -> Self
    where
        Self: Sized;
    
    fn get_process_name(&self) -> &str;

    fn get_proc(&self) -> &Proc;
}

pub struct Proc {
    pub pid: Pid,
    pub children: RefCell<Vec<Rc<dyn Process>>>,
    pub descriptor_table: Table<FileDirectoryPipe>
}

impl std::fmt::Debug for Proc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Proc")
            // .field("root", self.root)
            .finish()
    }
}

impl Proc {
    pub const fn new(pid: Pid) -> Self {
        Proc {
            pid,
            children: RefCell::new(vec![]),
            descriptor_table: Table::new()
        }
    }

    pub fn exit(&self) {
        // Exit?
    }
}