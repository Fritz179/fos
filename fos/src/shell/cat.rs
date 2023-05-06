use std::rc::Rc;

use crate::{
    root::{Proc, Process, descriptor::{ReadableDescriptor, WritableDescriptor}},
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

        match read {
            Ok(desc) => {
                let self_clone = Rc::clone(&self);
    
                ROOT.executor.add_task(async move {
                    loop {
                        let content = desc.read(50).await.unwrap();
                        self_clone.proc.stdout.write(&content);
                    }
                });
            },
            Err(err) => {
                let err = format!("Error: {:?}", err);
                self.proc.stdout.write(&err);
            }
        }

        self.proc.exit();
    }
}

impl CatProgram {}
