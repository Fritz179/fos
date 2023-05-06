// Every parent has a Rc to it's children
// Root has an Rc to every process
// Every child has a Weak to it's parent

// Every process has a root which has the usefull all functions

use std::{cell::RefCell, rc::Rc};

use crate::descriptor::ReadableWritablePipe;
use crate::fc::table::Table;
use crate::root::pipe::{new_pipe, PipeReader, PipeWriter};

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
    // pub descriptor_table: Table<RawHandler>,
    pub stdin: PipeReader,
    pub stdout: PipeWriter,
    pub handler: ReadableWritablePipe
}

impl Proc {
    pub fn new(pid: Pid) -> Self {
        let (stdin_reader, stdin_writer) = new_pipe();
        let (stdout_reader, stdout_writer) = new_pipe();


        // let descriptor_table = Table::new();

        let handler = ReadableWritablePipe::new(stdout_reader, stdin_writer);

        Proc {
            pid,
            children: RefCell::new(vec![]),
            // descriptor_table,
            stdin: stdin_reader,
            stdout: stdout_writer,
            handler
        }
    }

    pub fn exit(&self) {
        // Exit?
    }
}
