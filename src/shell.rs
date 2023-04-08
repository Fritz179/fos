use std::{cell::RefCell, rc::Rc};

use crate::{proc::{Proc, Process}, STDOUT, STDIN};

pub struct Shell {
    proc: Proc,
    buffer: RefCell<String>
}

impl Process for Shell {
    fn new(proc: Proc) -> Self where Self: Sized {
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
            self_clone.proc.write(STDOUT, char)
        }

        self.proc.read(STDIN, Box::new(move |char| {
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
                            println!("Echo!")
                        }
                        _ => {
                            for char in "Invalid command!\n".chars() {
                                self_clone.proc.write(STDOUT, char)
                            }
                        }
                    }
                }

                // prepare new line
                for char in message.chars() {
                    self_clone.proc.write(STDOUT, char)
                }

                buffer.clear();
            } else {
                buffer.push(char);
                self_clone.proc.write(STDOUT, char)
            }
        }))
    }
}