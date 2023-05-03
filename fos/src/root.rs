use std::{
    cell::{RefCell, RefMut},
    rc::Rc,
};

mod proc;
pub use proc::*;

mod fs;
pub use fs::*;

mod spawner;
pub use spawner::*;

mod terminal;
use terminal::Terminal;

use crate::{
    fc::future::Executor,
    platforms::{tekenen::Tekenen, Event, PlatformTrait},
    shell::Shell,
};

pub struct Root {
    pub platform: RefCell<Option<Box<dyn PlatformTrait>>>,
    terminal: Terminal,
    proc: Proc,
    pub fs: Fs,
    pub executor: Executor,
    pub spawner: Spawner,
}

impl Process for Root {
    fn new(proc: Proc) -> Root {
        let terminal = Terminal::new();
        let fs = Fs::new();
        let executor = Executor::new();

        Root {
            platform: RefCell::new(None),
            terminal,
            proc,
            fs,
            executor,
            spawner: Spawner::new(),
        }
    }

    fn get_process_name(&self) -> &str {
        "Root"
    }

    fn get_proc(&self) -> &Proc {
        &self.proc
    }

    fn main(self: Rc<Self>, _: Vec<&str>) {
        let shell = self.proc.spawn::<Shell>();

        // pipe stdin to shell stdin
        let self_clone = Rc::clone(&self);

        let shell_clone = Rc::clone(&shell);
        self.executor.add_task(async move {
            loop {
                let char = self_clone.proc.stdin.read().await;
                shell_clone
                    .proc
                    .stdin
                    .raw
                    .send(&char.expect("Option sening to shell"));
            }
        });

        // pipe shell stdout to terminal
        let self_clone = Rc::clone(&self);

        let shell_clone = Rc::clone(&shell);
        self.executor.add_task(async move {
            loop {
                let string = shell_clone.proc.stdout.raw.read().await;

                self_clone
                    .terminal
                    .write(&string.expect("Option sending to terminal"));
            }
        });

        shell.main(vec![]);
    }
}

impl Root {
    pub fn update(&self, tekenen: &mut Tekenen) -> bool {
        self.executor.execute();

        let borrow = self.platform.borrow_mut();

        let mut platform = RefMut::map(borrow, |platform| {
            if let Some(inner) = platform {
                inner
            } else {
                panic!("No platfrom")
            }
        });
        // let mut platform = platform.borrow_mut();

        while let Some(event) = platform.read_events() {
            match event {
                Event::Quit => {
                    // true indicates to interrupt the loop
                    return true;
                }
                Event::KeyDown { char, keycode, .. } => {
                    if let Some(c) = char {
                        self.proc.stdin.raw.send_char(c);
                    } else {
                        println!("unknown char {:?}", keycode)
                    }
                }
            }
        }

        self.terminal.render(tekenen, 0);

        platform.display_pixels(tekenen.get_pixels());

        // should not stop
        false
    }
}
