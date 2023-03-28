trait Trait {
    fn fun(&mut self);
    fn new(renderer: Rc<Renderer>) -> Self where Self: Sized;
}

struct App {
    renderer: Rc<Renderer>,
    num: i32
}

impl Trait for App {
    fn fun(&mut self) {

    }

    fn new(renderer: Rc<Renderer>) -> App {
        App {
            renderer,
            num: 7
        }
    }
}

use std::{cell::{RefCell}, rc::{Weak, Rc}};
// RefCell<Weak<RefCell<dyn AppTrait>>>
struct Renderer {
    app: RefCell<Weak<Rc<RefCell<dyn Trait>>>>
}

impl Renderer {
    fn new<AppStruct: Trait + 'static>() -> Rc<RefCell<AppStruct>> {

        let renderer = Rc::new(Renderer {
            app: RefCell::new(Weak::new())
        });

        let app = AppStruct::new(Rc::clone(&renderer));

        let appref = Rc::new(RefCell::new(app));
        let appclone = Rc::clone(&appref);

        // let mut renderer_app = renderer.app.borrow_mut().to_owned();

        let traitclone: Rc<RefCell<dyn Trait>> = Rc::clone(&(appclone as Rc<RefCell<dyn Trait>>));
        let clone = Rc::downgrade(&Rc::new(traitclone));

        let mut borrow = renderer.app.borrow_mut();
        *borrow = clone;
       

        return appref // as Rc<RefCell<Box<App>>>
    }
}

fn main() {
    let app = Renderer::new::<App>();

    app.borrow_mut().fun();
}