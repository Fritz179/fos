use std::{cell::RefCell, rc::Rc, fmt};

mod proc;
pub use proc::*;

mod fs;
pub use fs::*;

mod spawner;
pub use spawner::*;

mod terminal;
use terminal::Terminal;

use crate::{platforms::{tekenen::Tekenen, Event, PlatformTrait}, shell::Shell, fc::future::Executor};

// static ro: RefCell<String> = todo!();

pub struct Root {
    pub platform: RefCell<Box<dyn PlatformTrait>>,
    terminal: Terminal,
    proc: Proc,
    pub fs: Fs,
    pub executor: Executor,
    pub spawner: Spawner
}

impl Process for Root {
    fn new(proc: Proc) -> Root {
        todo!();

        // let platform = RefCell::new(Platform::new(800, 600));
        // let terminal = Terminal::new();
        // let fs = Fs::new();
        // let executor = Executor::new();

        // Root {
        //     platform,
        //     terminal,
        //     proc,
        //     fs: Rc::new(fs),
        //     executor: Rc::new(executor),
        //     spawner: Rc::new(Spawner::new()),
        // }
    }

    fn get_process_name(&self) -> &str {
        "Root"
    }

    fn get_proc(&self) -> &Proc {
        &self.proc
    }
}

impl Root {
    pub fn new_2<Platform: PlatformTrait + 'static>(proc: Proc) -> Root {
        let platform = RefCell::new(Platform::new(800, 600));
        let terminal = Terminal::new();
        let fs = Fs::new();
        let executor = Executor::new();

        Root {
            platform,
            terminal,
            proc,
            fs: fs,
            executor: executor,
            spawner: Spawner::new(),
        }
    }

    pub fn main(self: &Rc<Self>) {
        let (shell, _) = self.proc.spawn::<Shell>();

        // pipe stdin to shell stdin
        let self_clone = Rc::clone(self);

        let shell_clone = Rc::clone(&shell);
        self.executor.add_task(async move {
            loop {
                let char = self_clone.proc.read(STDIN).await;
                shell_clone.proc.write(STDIN, &char.expect("Option sening to shell").to_string());
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
                        self.proc.write(STDIN, &c.to_string());
                    } else {
                        println!("unknown char {:?}", keycode)
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