use std::{cell::RefCell, rc::Rc, fmt};

mod platforms;
use platforms::{SDLPlatform, Event, Tekenen};

mod proc;
use proc::*;

mod fs;
pub use fs::*;

mod terminal;
use terminal::Terminal;

pub struct Root {
    platform: RefCell<Box<SDLPlatform>>,
    tekenen: Tekenen,
    terminal: Terminal,
    proc: Proc
}

impl Process for Root {
    fn new(proc: Proc) -> Root {
        Root {
            platform: RefCell::new(platforms::SDLPlatform::new(800, 600)),
            tekenen: Tekenen::new(800, 600),
            terminal: Terminal::new(),
            proc
        }
    }

    fn main(self: Rc<Self>) {
        let self_clone = Rc::clone(&self);

        self.proc.read(0, Box::new(move |c|{
            self_clone.terminal.write(c);
        }))
    }
}

impl Root {
    fn update(&self) -> bool {
        let mut platform = self.platform.borrow_mut();

        while let Some(event) = platform.read_events() {
            match event {
                Event::Quit => {
                    // true indicates to interrupt the loop
                    return true;
                },
                Event::KeyDown { char, keycode, .. } => {
                    if let Some(c) = char {
                        self.proc.write(0, c)
                    } else {
                        println!("{}", keycode)
                    }
                }
                // _ => {
                //     println!("Unhandled event: {:?}", event);
                // }
            }
        }

        self.terminal.render(&self.tekenen, 0);

        platform.display_pixels(&self.tekenen);

        // should not stop
        return false
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


fn main() {
    let fs = Rc::new(Fs::new());
    let spawner = Rc::new(Spawner::new(Rc::clone(&fs)));

    let root = spawner.spawn::<Root>();

    println!("{:?}", spawner);

    SDLPlatform::set_interval(Box::new(move || {
        return root.update();
    }), 60);

}