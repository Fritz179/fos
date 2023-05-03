use std::rc::Rc;

use crate::{
    root::{Proc, Process},
    ROOT,
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

    fn main(self: Rc<Self>, args: Vec<&str>) {
        assert!(args.len() == 1);

        let file = args[0];

        let read = self.proc.open(file.to_string());

        if let Ok(desc) = read {
            let self_clone = Rc::clone(&self);

            ROOT.executor.add_task(async move {
                loop {
                    let content = desc.read().await.unwrap();
                    self_clone.proc.stdout.write(&content);
                }
            });
        }

        self.proc.exit();
    }
}

impl CatProgram {}
