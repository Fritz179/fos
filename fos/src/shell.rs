use std::{cell::RefCell, rc::Rc};

use crate::{
    root::{Proc, Process},
    ROOT
};

mod echo;
use echo::EchoProgram;

mod ps_tree;
use ps_tree::PsTreeProgram;

mod cat;
use cat::CatProgram;

pub struct Shell {
    pub proc: Proc,
    buffer: RefCell<String>,
}

impl Process for Shell {
    fn new(proc: Proc) -> Self
    where
        Self: Sized,
    {
        Shell {
            proc,
            buffer: RefCell::new(String::new()),
        }
    }

    fn get_process_name(&self) -> &str {
        "Shell"
    }

    fn get_proc(&self) -> &Proc {
        &self.proc
    }

    fn main(self: Rc<Self>, _: Vec<&str>) {
        let self_clone = Rc::clone(&self);
        let message = "fritz@tekenen:~$ ".to_string();

        self_clone.proc.stdout.write(&message);

        ROOT.executor.add_task(async move {
            loop {
                let char = self_clone.proc.stdin.read_char().await.unwrap();

                let mut buffer = self_clone.buffer.borrow_mut();

                if char == '\n' {
                    self_clone.proc.stdout.write_char(char);

                    // process command

                    let mut strings: Vec<&str> = vec![];
                    for string in buffer.split_whitespace() {
                        strings.push(string);
                    }

                    if !strings.is_empty() {
                        let command = strings.remove(0);
                        println!("{}", command);

                        let program: Option<Rc<dyn Process>> = match command {
                            "echo" => Some(self_clone.proc.spawn::<EchoProgram>()) ,
                            "pstree" => Some(self_clone.proc.spawn::<PsTreeProgram>()),
                            "cat" => Some(self_clone.proc.spawn::<CatProgram>()),
                            _ => {
                                self_clone.proc.stdout.write("Invalid command!\n");
                                None
                            }
                        };

                        if let Some(program) = program {
                            // pipe shell stdout to terminal
                            let self_clone_clone = Rc::clone(&self_clone);
                            let program_clone = Rc::clone(&program);
                            ROOT.executor.add_task(async move {
                                loop {
                                    let str = program_clone.get_proc().stdout.raw.read().await.unwrap();
                                    self_clone_clone.proc.stdout.write(&str);
                                }
                            });

                            program.main(strings);
                        }
                    }

                    self_clone.proc.stdout.write(&message);

                    buffer.clear();
                } else {
                    buffer.push(char);
                    self_clone.proc.stdout.write_char(char);
                }
            }
        })
    }
}