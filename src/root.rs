use std::{cell::RefCell, rc::Rc, fmt};

mod proc;
pub use proc::*;

mod fs;
pub use fs::*;

mod spawner;
pub use spawner::*;

mod terminal;
use terminal::Terminal;

use crate::{platforms::{SDLPlatform, tekenen::Tekenen, Event, Platform}, shell::Shell, fc::future::Executor};

pub struct Root {
    platform: RefCell<Box<SDLPlatform>>,
    terminal: Terminal,
    proc: Proc,
    pub fs: Rc<Fs>,
    pub executor: Rc<Executor>,
    pub spawner: Rc<Spawner>
}

impl Process for Root {
    fn new(proc: Proc) -> Root {
        let platform = RefCell::new(SDLPlatform::new(800, 600));
        let terminal = Terminal::new();
        let fs = Fs::new();
        let executor = Executor::new();

        Root {
            platform,
            terminal,
            proc,
            fs: Rc::new(fs),
            executor: Rc::new(executor),
            spawner: Rc::new(Spawner::new()),
        }
    }
}

impl Root {
    pub fn main(self: &Rc<Self>) {
        let (shell, _) = self.proc.spawn::<Shell>();

        // pipe stdin to shell stdin
        let self_clone = Rc::clone(self);

        let shell_clone = Rc::clone(&shell);
        self.executor.add_task(async move {
            loop {
                let char = self_clone.proc.read(STDIN).await;
                shell_clone.proc.write(STDIN, char.expect("Option sening to shell"));
            }
        });

        // pipe shell stdout to terminal
        let self_clone = Rc::clone(self);

        let shell_clone = Rc::clone(&shell);
        self.executor.add_task(async move {
            loop {
                let char = shell_clone.proc.read(STDOUT).await;

                self_clone.terminal.write(char.expect("Option sening to terminal"));
            }
        });

        shell.main();
    }

    pub fn update(&self, tekenen: &mut Tekenen) -> bool {
        self.executor.execute();

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

        self.terminal.render(tekenen, 0);

        platform.display_pixels(tekenen.get_pixels());

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