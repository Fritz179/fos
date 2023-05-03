use std::rc::Rc;

use crate::{
    root::{Proc, Process},
    Root, ROOT,
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

    fn main(self: Rc<Self>, _: Vec<&str>) {
        let root: Rc<Root> = Rc::clone(&*ROOT);
        self.print(&(root as Rc<dyn Process>), 0);

        self.proc.exit();
    }
}

impl PsTreeProgram {
    pub fn print(&self, node: &Rc<dyn Process>, indent: u32) {
        let proc = node.get_proc();

        let name = node.get_process_name();
        let pid = proc.pid;

        let string = format!("[{pid}]{name}");

        for _ in 0..indent {
            self.proc.stdout.write_char(' ');
        }

        self.proc.stdout.write(&string);

        self.proc.stdout.write_char('\n');

        for child in proc.children.borrow().iter() {
            self.print(child, indent + 2)
        }
    }
}
