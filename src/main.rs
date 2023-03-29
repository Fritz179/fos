mod tekenen;
use tekenen::*;
use std::rc::Rc;

struct App {
    renderer: Rc<Tekenen>,
    buffer: String
}

impl AppTrait for App {
    fn update(&mut self, time: u64) {
        self.renderer.background(colors::GRAY);

        self.renderer.draw_terminal(&self.buffer, time);
    }

    fn event_handler(&mut self, event: Event) -> bool {
        // println!("{:?}", event);

        match event {
            Event::KeyDown { keycode: Some(key), keymod, /* repeat: false, */ ..} => {
                // println!("{}, {:?}", key, keymod);

                let shift_mod: bool = keymod.bits() & (Mod::LSHIFTMOD.bits() | Mod::RSHIFTMOD.bits()) != 0;
                let ctrl_mod: bool = keymod.bits() & (Mod::LCTRLMOD.bits() | Mod::RCTRLMOD.bits()) != 0;
                let caps_mod: bool = keymod.bits() & Mod::CAPSMOD.bits() != 0;

                match key {
                    Keycode::Escape => true,
                    Keycode::Return => {
                        self.buffer.push('\n');
                        false
                    }
                    Keycode::Space => {
                        self.buffer.push(' ');
                        false
                    }
                    Keycode::Backspace => {
                        if ctrl_mod {
                            let mut did_delete_char = false; 

                            while self.buffer.len() != 0 {
                                let char = self.buffer.pop();

                                if let Some(' ') = char {
                                    if did_delete_char {
                                        self.buffer.push(' ');
                                        break
                                    }
                                    continue
                                }

                                // TODO: Could read before deliting and inserting
                                if let Some('\n') = char {
                                    self.buffer.push('\n');
                                    break
                                }

                                did_delete_char = true;
                            }

                            return false
                        }

                        self.buffer.pop();
                        false
                    }
                    _ => {
                        const START: i32 = FIRST_CHAR as i32;
                        const END: i32 = LAST_CHAR as i32;

                        // println!("{}, {}, {:?}", key as i32, END, keymod);

                        let mut capital = 0;
                        if  shift_mod || caps_mod {
                            capital = -32
                        }

                        match key as i32 {
                            START..=END => {
                                let chr = char::from_u32((key as i32 + capital) as u32).expect("msg");
                                self.buffer.push(chr);
                                false
                            },
                            _ => false
                        }
                    }
                }
            },
            Event::KeyUp { keycode: Some(key), repeat: false, ..} => {
                // println!("{}", key);

                match key {
                    _ => false
                }
            },
            _ => false
        }
    }

    fn new(renderer: Rc<Tekenen>) -> App {
        App {
            renderer,
            buffer: "".to_string()
        }
    }
}

impl App {

}

fn main() {
    let tekenen = Tekenen::new(800, 600);
    let _app = Tekenen::new_app::<App>(&tekenen);

    tekenen.start(60);
}