use std::rc::Rc;

use crate::{
    root::{Proc, Process},
    STDOUT,
};

pub struct CatProgram {
    pub proc: Proc,
}

impl Process for CatProgram {
    fn new(proc: Proc) -> Self
    where
        Self: Sized,
    {
        CatProgram { proc }
    }

    fn get_process_name(&self) -> &str {
        "Cat"
    }

    fn get_proc(&self) -> &Proc {
        &self.proc
    }
}

impl CatProgram {
    pub fn main(self: &Rc<Self>, argv: Vec<&str>) {
        assert!(argv.len() == 1);

        let file = argv[0];

        let read = self.proc.open(file.to_string());

        if let Ok(desc) = read {
            let self_clone = Rc::clone(self);

            self.proc.root.executor.add_task(async move {
                loop {
                    let char = self_clone.proc.read(desc.clone()).await.unwrap();
                    self_clone.proc.write(STDOUT, char);
                }
            });
        }

        self.proc.exit();
    }
}
