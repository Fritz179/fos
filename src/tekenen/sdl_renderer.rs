extern crate sdl2;

use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::EventPump;
use std::time::Duration;
use sdl2::rect::Rect;
use sdl2::event::Event;

use crate::{RendererTrait, Tekenen, Pixels};
use std::rc::{Rc, Weak};
use std::cell::{Ref};

pub struct SDLRenderer {
    canvas: Canvas<Window>,
    event_pump: EventPump,
    tekenen: Weak<Tekenen>,
}

impl RendererTrait for SDLRenderer {
    fn new(width: usize, height: usize, name: &str) -> Box<SDLRenderer> {

        let sdl_context = sdl2::init().expect("Cannoti init sdl2!");
        let video_subsystem = sdl_context.video().expect("Cannot init video");

        let window = video_subsystem.window(name, width as u32, height as u32)
            .position_centered()
            .build()
            .unwrap();

        let canvas = window.into_canvas().build().unwrap();
        let event_pump = sdl_context.event_pump().unwrap();

        let renderer = SDLRenderer {
            canvas,
            event_pump,
            tekenen: Weak::new()
        };

        return Box::new(renderer)
    }
    
    fn set_tekenen(&mut self, tekenen: Rc<Tekenen>) {
        self.tekenen = Rc::downgrade(&tekenen)
    }

    fn start(&mut self, _fps: u32) {
        'running: loop {
            let tekenen = self.tekenen.upgrade().expect("No tekenen");

            for event in self.event_pump.poll_iter() {

                // TODO: Could probably do with an if
                match event {
                    Event::Quit { .. } => {
                        break 'running
                    },
                    _ => {}
                }

                let should_stop = tekenen.event_handler(event);

                if should_stop {
                    break 'running
                }
            }
            
            let (width, height) = self.canvas.output_size().expect("Cannot get canvas size");
    
            // if self.width != width as usize || HEIGHT != height as usize {
            //     break 'running
            // }
    
            let creator = self.canvas.texture_creator();
            let sprite = Rect::new(0, 0, width, height);
            
            let mut texture = creator.create_texture(
                sdl2::pixels::PixelFormatEnum::RGBA32,
                sdl2::render::TextureAccess::Target,
                width, height
            ).unwrap();

            tekenen.update();
    
            // convert from [[[[u8]; 3]; WIDTH]; HEIGHT] to [u8; 3 * WIDTH * HEIGHT]
            let pixels: Ref<Pixels> = tekenen.get_pixels();
            // let slice: &[u8; 800 * 600 * 4] = unsafe { std::mem::transmute(&*pixels)};
            // println!("{}", pixels.len());

    
            texture.update(sprite, &pixels, (800 * 4) as usize).unwrap();
    
            let sprite = Rect::new(0, 0, width, height);
            self.canvas.copy(&texture, sprite, sprite).expect("Cannot copy texture to canvas.");
    
            self.canvas.present();
            
            std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        }
    }
}