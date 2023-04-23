use crate::{
    root::{Proc, Process},
    STDOUT,
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
}

impl EchoProgram {
    pub fn main(&self, argv: Vec<&str>) {
        for arg in argv {
            for char in arg.bytes() {
                self.proc.write(STDOUT, char as char);
            }
        }

        self.proc.write(STDOUT, '\n');

        self.proc.exit();
    }
}