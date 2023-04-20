use std::rc::Rc;

use crate::{
    root::{Proc, Process},
    STDOUT,
};

pub struct PsTreeProgram {
    pub proc: Proc,
}

impl Process for PsTreeProgram {
    fn new(proc: Proc) -> Self
    where
        Self: Sized,
    {
        PsTreeProgram { proc }
    }

    fn get_process_name(&self) -> &str {
        "PsTree"
    }

    fn get_proc(&self) -> &Proc {
        &self.proc
    }
}

impl PsTreeProgram {
    pub fn print(&self, node: &Rc<dyn Process>, indent: u32) {

        let proc = node.get_proc();

        let name = node.get_process_name();
        let pid = proc.pid;

        let string = format!("[{pid}]{name}");

        for _ in 0..indent {
            self.proc.write(STDOUT, ' ');
        }

        for c in string.chars() {
            self.proc.write(STDOUT, c);
        }

        self.proc.write(STDOUT, '\n');

        for child in proc.children.borrow().iter() {
            self.print(child, indent + 2)
        }
        
    } 

    pub fn main(&self) {
        let root = Rc::clone(&self.proc.root) as Rc<dyn Process>;

        self.print(&root, 0);

        self.proc.exit();
    }
}
