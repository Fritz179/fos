use std::{cell::RefCell, rc::Rc};

use crate::{
    root::{Proc, Process},
    STDIN, STDOUT,
};

mod echo;
use echo::EchoProgram;

mod top;
use top::TopProgram;

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
}

impl Shell {
    pub fn main(self: &Rc<Self>) {
        let self_clone = Rc::clone(&self);
        let message = "fritz@tekenen:~$ ".to_string();

        for char in message.chars() {
            self_clone.proc.write(STDOUT, char);
        }

        self.proc.root.executor.add_task(async move {
            loop {
                let char = self_clone.proc.read(STDIN).await.unwrap();

                let mut buffer = self_clone.buffer.borrow_mut();

                if char == '\n' {
                    self_clone.proc.write(STDOUT, char);

                    // process command

                    let mut strings: Vec<&str> = vec![];
                    for string in buffer.split_whitespace() {
                        strings.push(string);
                    }

                    println!("{}", strings.len());

                    if strings.len() > 0 {
                        let command = strings.remove(0);
                        println!("{}", command);

                        match command {
                            "echo" => {
                                let (echo, _) = self_clone.proc.spawn::<EchoProgram>();

                                // pipe shell stdin to echo stdin
                                // let self_clone_clone = Rc::clone(&self_clone);
                                // spawner.executor.add_task(async move {
                                //     loop {
                                //         self_clone_clone.proc.read(descriptor)
                                //     }
                                // });

                                // pipe shell stdout to terminal
                                let self_clone_clone = Rc::clone(&self_clone);
                                let echo_clone = Rc::clone(&echo);
                                self_clone.proc.root.executor.add_task(async move {
                                    loop {
                                        let char = echo_clone.proc.read(STDOUT).await.unwrap();
                                        self_clone_clone.proc.write(STDOUT, char);
                                    }
                                });

                                echo.main(strings);
                            },
                            "top" => {
                                let (top, _) = self_clone.proc.spawn::<TopProgram>();

                                // pipe shell stdin to echo stdin
                                // let self_clone_clone = Rc::clone(&self_clone);
                                // spawner.executor.add_task(async move {
                                //     loop {
                                //         self_clone_clone.proc.read(descriptor)
                                //     }
                                // });

                                // pipe shell stdout to terminal
                                let self_clone_clone = Rc::clone(&self_clone);
                                let echo_clone = Rc::clone(&top);
                                self_clone.proc.root.executor.add_task(async move {
                                    loop {
                                        let char = echo_clone.proc.read(STDOUT).await.unwrap();
                                        self_clone_clone.proc.write(STDOUT, char);
                                    }
                                });

                                top.main();
                            },
                            _ => {
                                for char in "Invalid command!\n".chars() {
                                    self_clone.proc.write(STDOUT, char);
                                }
                            }
                        }
                    }

                    // prepare new line
                    for char in message.chars() {
                        self_clone.proc.write(STDOUT, char);
                    }

                    buffer.clear();
                } else {
                    buffer.push(char);
                    self_clone.proc.write(STDOUT, char);
                }
            }
        })
    }
}
