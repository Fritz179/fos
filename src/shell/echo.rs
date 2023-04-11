use crate::{
    proc::{Proc, Process},
    STDOUT,
};

pub struct EchoProgram {
    proc: Proc,
}

impl Process for EchoProgram {
    fn new(proc: Proc) -> Self
    where
        Self: Sized,
    {
        EchoProgram { proc }
    }
}

impl EchoProgram {
    pub fn main(&self, argv: Vec<&str>) {
        for arg in argv {
            for char in arg.bytes() {
                self.proc.write(STDOUT, char as char)
            }
        }

        self.proc.write(STDOUT, '\n')
    }
}
