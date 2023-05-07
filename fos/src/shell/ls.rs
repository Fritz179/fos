use std::rc::Rc;

use crate::{
    root::{Proc, Process, descriptor::{ReadableDescriptor, WritableDescriptor}},
    ROOT, Inode,
};

pub struct LsProgram {
    pub proc: Proc,
}

impl Process for LsProgram {
    fn new(proc: Proc) -> Self
    where
        Self: Sized,
    {
        LsProgram { proc }
    }

    fn get_process_name(&self) -> &str {
        "ls"
    }

    fn get_proc(&self) -> &Proc {
        &self.proc
    }

    fn main(self: Rc<Self>, args: Vec<&str>) {
        assert!(args.len() == 1);

        let dir_name = args[0];

        let dir = self.proc.open_dir(dir_name.to_string());

        match dir {
            Ok(desc) => {    
                desc.0.iter().for_each(|node| {
                    match node {
                        (name, Inode::Directory(_)) => {
                            self.proc.stdout.write(&format!("Directory: {:?} \n", name.0));
                        },
                        (name, Inode::File(_)) => {
                            self.proc.stdout.write(&format!("File: {:?}\n", name.0));
                        }
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
