use console::{Term, Key, style};

use std::cell::RefCell;

use fos::{tekenen::Pixels, Event, Keycode, Keymod, PlatformTrait};

thread_local! {
    static NEXT: RefCell<Option<char>> = RefCell::new(None);
    static TERM: Term = Term::stdout()
}

pub struct ShellPlatform {
    width: u32,
    height: u32,
}

impl PlatformTrait for ShellPlatform {
    fn new(width: u32, height: u32) -> Box<ShellPlatform> {
        let io_manger = ShellPlatform {
            width,
            height
        };

        Box::new(io_manger)
    }

    fn display_pixels(&mut self, pixels: &Pixels) {

        TERM.with(|term| {
            let _ = term.clear_screen();
        });

        let scale = 2;

        let mut output = String::new();


        for y in 0..self.height / scale / 2 {


            for x in 0..self.width / scale {
                let i = (y * self.width + x) * scale * 4;
                let color = pixels[i as usize];

                if color == 0 {
                    output.push_str(&style("  ").on_black().to_string())
                } else {
                    output.push_str(&style("  ").on_white().to_string())
                }
            } 

            output.push('\n')
        }   

        println!("{}", output)

    }

    fn read_events(&mut self) -> Option<Event> {
        let char = NEXT.with(|next| {
            let mut next = next.borrow_mut();

            next.take()
        });

        if let Some(key) = char {

            Some(Event::KeyDown {
                repeat: false,
                char: Some(key),
                keycode: Keycode::Temp,
                keymod: Keymod {
                    shift: false,
                    ctrl: false,
                    caps: false
                }
            })
        } else {
            println!("{:?}", char);
            None
        }
    }

    fn set_interval(mut callback: Box<dyn FnMut() -> bool>, _fps: u32) {
        // let now = std::time::SystemTime::now();

        'running: loop {
            let char = TERM.with(|term| {
                term.read_key()
            });

            match char {
                Ok(Key::Char(key)) => {
                    NEXT.with(|next| {
                        let mut next = next.borrow_mut();

                        let _ = next.insert(key);
                    })
                }
                Ok(Key::Enter) => {
                    NEXT.with(|next| {
                        let mut next = next.borrow_mut();

                        let _ = next.insert('\n');
                    })
                }
                _ => {
                    println!("{:?}", char);
                }
            };

            // TODO: pass time since startup to callback
            let should_stop = callback();

            if should_stop {
                break 'running;
            }
        
            // std::thread::sleep(Duration::new(0, 1_000_000_000u32 / fps));
        }
    }
}

fn main() {
    fos::main::<ShellPlatform>()
}
