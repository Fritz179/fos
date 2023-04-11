use sdl2::keyboard::Keycode;
use std::time::Duration;

use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::EventPump;
use std::time::SystemTime;

pub mod colors;
pub mod font;

mod tekenen;
pub use tekenen::Tekenen;

pub struct SDLPlatform {
    canvas: Canvas<Window>,
    event_pump: EventPump,
    start: SystemTime,
    active: bool,
}

#[derive(Debug)]
pub struct Keymod {
    shift: bool,
    ctrl: bool,
    caps: bool,
}

#[derive(Debug)]
pub enum Event {
    KeyDown {
        repeat: bool,
        char: Option<char>,
        keycode: Keycode,
        keymod: Keymod,
    },
    Quit,
}

impl SDLPlatform {
    pub fn new(width: u32, height: u32) -> Box<SDLPlatform> {
        let sdl_context = sdl2::init().expect("Cannot init sdl2!");
        let video_subsystem = sdl_context.video().expect("Cannot init video");

        let window = video_subsystem
            .window("Salve!", width as u32, height as u32)
            .position_centered()
            .build()
            .expect("Cannot create window!");

        let canvas = window.into_canvas().build().expect("Cannot create canvas!");
        let event_pump = sdl_context.event_pump().expect("Cannot create event_pump!");

        let io_manger = SDLPlatform {
            canvas,
            event_pump,
            start: SystemTime::now(),
            active: true,
        };

        return Box::new(io_manger);
    }

    pub fn display_pixels(&mut self, tekenen: &Tekenen) {
        let pixels = tekenen.pixels.borrow();

        let (width, height) = self.canvas.output_size().expect("Cannot get canvas size");

        assert!(
            width * height * 4 == pixels.len() as u32,
            "Cannot render pixels!, Expected: {}, found: {}",
            width * height * 4,
            pixels.len()
        );

        let creator = self.canvas.texture_creator();
        let sprite = Rect::new(0, 0, width, height);

        let mut texture = creator
            .create_texture(
                sdl2::pixels::PixelFormatEnum::RGBA32,
                sdl2::render::TextureAccess::Target,
                width,
                height,
            )
            .unwrap();

        texture
            .update(sprite, &*pixels, (800 * 4) as usize)
            .unwrap();

        let sprite = Rect::new(0, 0, width, height);
        self.canvas
            .copy(&texture, sprite, sprite)
            .expect("Cannot copy texture to canvas.");

        self.canvas.present();
    }

    pub fn read_events(&mut self) -> Option<Event> {
        for event in self.event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => {
                    self.active = false;
                    return Some(Event::Quit);
                }
                sdl2::event::Event::KeyDown {
                    keymod,
                    keycode,
                    repeat,
                    ..
                } => {
                    if let Some(keycode) = keycode {
                        let shift_mod: bool = keymod.bits()
                            & (sdl2::keyboard::Mod::LSHIFTMOD.bits()
                                | sdl2::keyboard::Mod::RSHIFTMOD.bits())
                            != 0;
                        let ctrl_mod: bool = keymod.bits()
                            & (sdl2::keyboard::Mod::LCTRLMOD.bits()
                                | sdl2::keyboard::Mod::RCTRLMOD.bits())
                            != 0;
                        let caps_mod: bool =
                            keymod.bits() & sdl2::keyboard::Mod::CAPSMOD.bits() != 0;

                        let charcode = keycode as u32;
                        let mut char = None;

                        // Standard ascii code
                        if charcode >= ' ' as u32 && charcode <= '~' as u32 {
                            char = char::from_u32(charcode);
                        }

                        if keycode == Keycode::Return {
                            char = Some('\n')
                        }

                        return Some(Event::KeyDown {
                            repeat,
                            char,
                            keycode,
                            keymod: Keymod {
                                shift: shift_mod,
                                ctrl: ctrl_mod,
                                caps: caps_mod,
                            },
                        });
                    }
                }
                _ => {
                    // println!("Unhandled event: {:?}", event);
                }
            }
        }

        return None;
    }

    pub fn set_interval(callback: Box<dyn Fn() -> bool>, fps: u32) {
        // let now = std::time::SystemTime::now();

        'running: loop {
            // TODO: pass time since startup to callback
            let should_stop = callback();

            if should_stop {
                break 'running;
            }

            std::thread::sleep(Duration::new(0, 1_000_000_000u32 / fps));
        }
    }
}
