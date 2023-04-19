use crate::{
    root::{Proc, Process},
    STDOUT,
};

pub struct TopProgram {
    pub proc: Proc,
}

impl Process for TopProgram {
    fn new(proc: Proc) -> Self
    where
        Self: Sized,
    {
        TopProgram { proc }
    }
}

impl TopProgram {
    pub fn main(&self) {
        self.proc.write(STDOUT, 'h');


        self.proc.write(STDOUT, '\n');

        self.proc.exit();
    }
}
