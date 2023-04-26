use std::{cell::RefCell, rc::Rc};

use crate::{
    root::{Proc, Process},
    STDIN, STDOUT,
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
}

impl Shell {
    pub fn main(self: &Rc<Self>) {
        let self_clone = Rc::clone(&self);
        let message = "fritz@tekenen:~$ ".to_string();

        self_clone.proc.write(STDOUT, &message);

        self.proc.root.executor.add_task(async move {
            loop {
                let char = self_clone.proc.read_char(STDIN).await.unwrap();

                let mut buffer = self_clone.buffer.borrow_mut();

                if char == '\n' {
                    self_clone.proc.write(STDOUT, &char.to_string());

                    // process command

                    let mut strings: Vec<&str> = vec![];
                    for string in buffer.split_whitespace() {
                        strings.push(string);
                    }

                    // println!("{}", strings.len());

                    if strings.len() > 0 {
                        let command = strings.remove(0);
                        println!("{}", command);

                        match command {
                            "echo" => {
                                let (echo, _) = self_clone.proc.spawn::<EchoProgram>();


                                // pipe shell stdout to terminal
                                let self_clone_clone = Rc::clone(&self_clone);
                                let echo_clone = Rc::clone(&echo);
                                self_clone.proc.root.executor.add_task(async move {
                                    loop {
                                        let char = echo_clone.proc.read(STDOUT).await.unwrap();
                                        self_clone_clone.proc.write(STDOUT, &char);
                                    }
                                });

                                echo.main(strings);
                            },
                            "pstree" => {
                                let (ps_tree, _) = self_clone.proc.spawn::<PsTreeProgram>();

                                // pipe shell stdout to terminal
                                let self_clone_clone = Rc::clone(&self_clone);
                                let ps_tree_clone = Rc::clone(&ps_tree);
                                self_clone.proc.root.executor.add_task(async move {
                                    loop {
                                        let char = ps_tree_clone.proc.read(STDOUT).await.unwrap();
                                        self_clone_clone.proc.write(STDOUT, &char);
                                    }
                                });

                                ps_tree.main();
                            },
                            "cat" => {
                                let (cat, _) = self_clone.proc.spawn::<CatProgram>();

                                // pipe shell stdout to terminal
                                let self_clone_clone = Rc::clone(&self_clone);
                                let ps_tree_clone = Rc::clone(&cat);
                                self_clone.proc.root.executor.add_task(async move {
                                    loop {
                                        let char = ps_tree_clone.proc.read(STDOUT).await.unwrap();
                                        self_clone_clone.proc.write(STDOUT, &char);
                                    }
                                });

                                cat.main(strings);
                            },
                            _ => {
                                self_clone.proc.write(STDOUT, "Invalid command!\n");
                            }
                        }
                    }

                    // prepare new line
                    self_clone.proc.write(STDOUT, &message);

                    buffer.clear();
                } else {
                    buffer.push(char);
                    self_clone.proc.write(STDOUT, &char.to_string());
                }
            }
        })
    }
}
