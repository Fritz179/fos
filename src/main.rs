use std::{cell::RefCell, fmt, rc::Rc};

mod platforms;
use platforms::{Event, SDLPlatform, Tekenen};

mod proc;
use proc::*;

mod fs;
pub use fs::*;

mod terminal;
use terminal::Terminal;

mod shell;
use shell::Shell;

mod table;
pub use table::Table;

use crate::future::Executor;

mod channel;

pub struct Root {
    platform: RefCell<Box<SDLPlatform>>,
    tekenen: Tekenen,
    terminal: Terminal,
    proc: Proc,
}

impl Process for Root {
    fn new(proc: Proc) -> Root {
        Root {
            platform: RefCell::new(platforms::SDLPlatform::new(800, 600)),
            tekenen: Tekenen::new(800, 600),
            terminal: Terminal::new(),
            proc,
        }
    }
}

impl Root {
    fn main(self: &Rc<Self>) {
        let (shell, shell_pid) = self.proc.spawn::<Shell>();
        let spawner = self.proc.spawner.upgrade().unwrap();

        // pipe stdin to shell stdin
        let self_clone = Rc::clone(self);
        // self.proc.read(
        //     STDIN,
        //     Box::new(move |char| {
        //         let fs = self_clone.proc.fs.upgrade().expect("No Fs");
        //         fs.write(shell_pid, 0, char);
        //     }),
        // );
        let self_clone = Rc::clone(self);

        let shell_clone = Rc::clone(&shell);
        spawner.executor.add_task(async move {
            loop {
                let char = self_clone.proc.read(STDIN).await;
                shell_clone.proc.write(STDIN, char.expect("Option sening to shell"));
            }
        });

        // pipe shell stdout to terminal
        let self_clone = Rc::clone(self);
        let fs = self_clone.proc.fs.upgrade().expect("No Fs");
        

        let shell_clone = Rc::clone(&shell);
        spawner.executor.add_task(async move {
            loop {
                let char = shell_clone.proc.read(STDOUT).await;

                self_clone.terminal.write(char.expect("Option sening to terminal"));
            }
        });

        shell.main();
    }

    fn update(&self) -> bool {
        let mut platform = self.platform.borrow_mut();

        while let Some(event) = platform.read_events() {
            match event {
                Event::Quit => {
                    // true indicates to interrupt the loop
                    return true;
                }
                Event::KeyDown { char, keycode, .. } => {
                    if let Some(c) = char {
                        self.proc.write(STDIN, c);
                    } else {
                        println!("a {}", keycode)
                    }
                } // _ => {
                  //     println!("Unhandled event: {:?}", event);
                  // }
            }
        }

        self.terminal.render(&self.tekenen, 0);

        platform.display_pixels(&self.tekenen);

        // should not stop
        return false;
    }
}

impl fmt::Debug for Root {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Root")
            //  .field("x", &self.x)
            //  .field("y", &self.y)
            .finish()
    }
}

pub mod future;

fn main() {
    let fs = Rc::new(Fs::new());
    let executor = Rc::new(Executor::new());

    let spawner = Rc::new(Spawner::new(fs, Rc::clone(&executor)));

    let (root, _pid) = spawner.spawn::<Root>();
    root.main();

    println!("{:?}", spawner);

    SDLPlatform::set_interval(Box::new(move || {
        executor.execute();
        return root.update();
    }), 60);
}