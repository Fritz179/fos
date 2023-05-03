use std::rc::Rc;

use crate::{
    root::{Proc, Process}
};

pub struct EchoProgram {
    pub proc: Proc,
}

impl Process for EchoProgram {
    fn new(proc: Proc) -> Self
    where
        Self: Sized,
    {
        EchoProgram { proc }
    }

    fn get_process_name(&self) -> &str {
        "Echo"
    }

    fn get_proc(&self) -> &Proc {
        &self.proc
    }

    fn main(self: Rc<Self>, args: Vec<&str>) {
        for arg in args {
            self.proc.stdout.write(arg);
        }

        self.proc.stdout.write_char('\n');

        self.proc.exit();
    }
}