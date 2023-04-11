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

mod map;
pub use map::Table;

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

        // pipe stdin to shell stdin
        let self_clone = Rc::clone(&self);
        self.proc.read(
            STDIN,
            Box::new(move |char| {
                let fs = self_clone.proc.fs.upgrade().expect("No Fs");
                fs.write(shell_pid, 0, char);
            }),
        );

        // pipe shell stdout to terminal
        let self_clone = Rc::clone(&self);
        let fs = self_clone.proc.fs.upgrade().expect("No Fs");
        fs.read(
            shell_pid,
            STDOUT,
            Box::new(move |char| {
                self_clone.terminal.write(char);
            }),
        );

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
                        self.proc.write(0, c)
                    } else {
                        println!("{}", keycode)
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
    future::tst();

    let fs = Rc::new(Fs::new());
    let spawner = Rc::new(Spawner::new(Rc::clone(&fs)));

    let (root, _pid) = spawner.spawn::<Root>();
    root.main();

    println!("{:?}", spawner);

    SDLPlatform::set_interval(Box::new(move || {
        return root.update();
    }), 60);
}