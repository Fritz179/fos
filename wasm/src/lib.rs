use std::{cell::RefCell, collections::VecDeque};

use wasm_bindgen::prelude::*;
use fos::{PlatformTrait, Event};

pub struct WASMTerminal {

}

type Callback = Box<dyn FnMut() -> bool>;

thread_local! {
    static ACTIVE_CALLBACK: RefCell<Option<Callback>> = RefCell::new(None);
    static KEY_QUEUE: RefCell<VecDeque<char>> = RefCell::new(VecDeque::new());
}


impl PlatformTrait for WASMTerminal {
    fn new(width: u32, height: u32) -> Box<Self> where Self: Sized {
        js_set_size(width, height);

        Box::new(WASMTerminal {
            
        })
    }

    fn display_pixels(&mut self, pixels: &fos::tekenen::Pixels) {

        // TODO: Use shared array buffers!!
        js_display_pixels(pixels.clone().into_boxed_slice())
    }

    fn read_events(&mut self) -> Option<fos::Event> {
        KEY_QUEUE.with(|queue| {
            let mut queue = queue.borrow_mut();
            let key = queue.pop_front();

            if let Some(key) = key {
                Some(Event::KeyDown {
                    repeat: false, 
                    char: Some(key), 
                    keycode: fos::Keycode::Temp, 
                    keymod: fos::Keymod { 
                        shift: false, 
                        ctrl: false, 
                        caps: false 
                    }
                })
            } else {
                None
            }
        })
    }

    fn set_interval(callback: Box<dyn FnMut() -> bool>, fps: u32) where Self: Sized {
        ACTIVE_CALLBACK.with(|active| {
            let mut active = active.borrow_mut();

            if active.is_some() {
                panic!("Only one interval supported");
            } else {
                active.insert(Box::new(callback));
            }
        });

        js_set_interval(fps)
    }
}

#[wasm_bindgen]
pub fn wasm_start() {
    fos::main::<WASMTerminal>()
}

#[wasm_bindgen]
pub fn wasm_key_down(key: char) {
    KEY_QUEUE.with(|queue| {
        let mut queue = queue.borrow_mut();

        queue.push_back(key)
    })
}

#[wasm_bindgen]
pub fn wasm_run_callback() {
    ACTIVE_CALLBACK.with(|active| {
        let mut active = active.borrow_mut();

        let active = active.as_mut();

        if let Some(active) = active {
            let should_stop = active();
        } else {
            panic!("No callback set!");
        }
    })
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen]
    fn js_set_size(width: u32, height: u32);

    #[wasm_bindgen]
    fn js_set_interval(fps: u32);

    #[wasm_bindgen]
    fn js_display_pixels(pixels: Box<[u8]>);
}