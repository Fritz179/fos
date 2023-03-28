mod tekenen;
use tekenen::*;

use std::rc::Rc;

struct Vec2<T> {
    x: T,
    y: T
}

struct App {
    renderer: Rc<Tekenen>,
    pos: Vec2<i32>,
    vel: Vec2<i32>
}

impl AppTrait for App {
    fn update(&mut self) {
        self.pos.x += self.vel.x * 10;
        self.pos.y += self.vel.y * 10;


        self.renderer.background([51, 51, 51, 255]);
        self.renderer.rect(self.pos.x, self.pos.y, 50, 50, [255, 0, 0, 255])
    }

    fn event_handler(&mut self, event: Event) -> bool {
        // println!("{:?}", event);

        match event {
            Event::KeyDown { keycode: Some(key), repeat: false, ..} => {
                // println!("{}", key);

                match key {
                    Keycode::Escape => true,
                    Keycode::W | Keycode::Up => {
                        self.vel.y -= 1;
                        false
                    } ,
                    Keycode::D | Keycode::Right => {
                        self.vel.x += 1;
                        false
                    } ,
                    Keycode::S | Keycode::Down => {
                        self.vel.y += 1;
                        false
                    } ,
                    Keycode::A | Keycode::Left => {
                        self.vel.x -= 1;
                        false
                    } ,
                    _ => false
                }
            },
            Event::KeyUp { keycode: Some(key), repeat: false, ..} => {
                // println!("{}", key);

                match key {
                    Keycode::W | Keycode::Up => {
                        self.vel.y += 1;
                        false
                    } ,
                    Keycode::D | Keycode::Right => {
                        self.vel.x -= 1;
                        false
                    } ,
                    Keycode::S | Keycode::Down => {
                        self.vel.y -= 1;
                        false
                    } ,
                    Keycode::A | Keycode::Left => {
                        self.vel.x += 1;
                        false
                    } ,
                    _ => false
                }
            },
            _ => false
        }
    }

    fn new(renderer: Rc<Tekenen>) -> App {
        App {
            renderer,
            pos: Vec2 { x: 0, y: 0 },
            vel: Vec2 { x: 0, y: 0 },
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