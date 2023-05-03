// Every parent has a Rc to it's children
// Root has an Rc to every process
// Every child has a Weak to it's parent

// Every process has a root which has the usefull all functions

use std::{
    cell::RefCell,
    rc::Rc,
};

use crate::{fc::{channel_handler::{ChannelHandler, new_channel_handler, Writable, Readable, Closed, RawHandler}, table::Table}};

pub type Pid = u32;

pub trait Process {
    fn new(proc: Proc) -> Self
    where
        Self: Sized;
    
    fn get_process_name(&self) -> &str;

    fn get_proc(&self) -> &Proc;

    fn main(self: Rc<Self>, args: Vec<&str>);
}

pub struct Proc {
    pub pid: Pid,
    pub children: RefCell<Vec<Rc<dyn Process>>>,
    pub descriptor_table: Table<RawHandler>,
    pub stdin: ChannelHandler<Readable, Closed>,
    pub stdout: ChannelHandler<Closed, Writable>,
}

impl Proc {
    pub fn new(pid: Pid) -> Self {
        let stdin = new_channel_handler().close_write(); 
        let stdout = new_channel_handler().close_read();
        
        let descriptor_table = Table::new();

        Proc {
            pid,
            children: RefCell::new(vec![]),
            descriptor_table,
            stdin,
            stdout,
        }
    }

    pub fn exit(&self) {
        // Exit?
    }
}