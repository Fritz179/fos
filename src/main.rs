mod tekenen;
use tekenen::*;
use std::{rc::{Rc, Weak}, cell::{RefCell}};

mod terminal;
use terminal::Terminal;

use std::collections::HashMap;

type Filedescriptor = u32;
pub const STD_IN: Filedescriptor = 1;
pub const STD_OUT: Filedescriptor = 2;
pub const STD_ERR: Filedescriptor = 3;

pub struct AppManager {
    renderer: Rc<Tekenen>,
    buffer: String,
    terminal: Option<Rc<Terminal>>,
    readers: HashMap<Filedescriptor, Box<dyn Fn(char)>>,
}


impl AppTrait for AppManager {
    fn update(&mut self, time: u64) {
        self.renderer.background(colors::GRAY);

        if let Some(terminal) = &self.terminal {
            terminal.render(&self.renderer, time);
        } 
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
                        self.write(STD_IN, '\n');
                        false
                    }
                    Keycode::Space => {
                        self.write(STD_IN, ' ');
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
                                let c = char::from_u32((key as i32 + capital) as u32).expect("msg");
                                self.write(STD_IN, c);
                                false
                            },
                            _ => false
                        }
                    }
                }
            },
            _ => false
        }
    }

    fn new(renderer: Rc<Tekenen>) -> Rc<RefCell<AppManager>> {
        let app = AppManager {
            renderer,
            buffer: String::new(),
            terminal: None,
            readers: HashMap::new(),
        };

        let appref = Rc::new(RefCell::new(app));

        let terminal = Rc::new(Terminal::new(Rc::downgrade(&Rc::clone(&appref))));
        terminal.main(Rc::downgrade(&terminal));

        appref.borrow_mut().terminal = Some(terminal);

        return appref
    }
}

impl AppManager {
    fn write(&mut self, file: Filedescriptor, c: char) {
        let reader = self.readers.get(&file);

        if let Some(reader) = reader {
            println!("Sending: {}, to ${}", c, file);
            reader(c);
        } else {
            println!("No one reading: ${}", file)
        }
    }
    fn read(&mut self, file: Filedescriptor, callback: Box<dyn Fn(char)>) {
        self.readers.insert(file, callback);
    }
}

fn main() {
    let tekenen = Tekenen::new(800, 600);
    let _app = Tekenen::new_app::<AppManager>(&tekenen);

    tekenen.start(60);
}