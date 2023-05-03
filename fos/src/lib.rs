mod platforms;
use std::{rc::Rc, ops::Deref};

use once_cell::sync::Lazy;

pub use platforms::*;

pub mod fc;

mod shell;

mod root;
pub use root::*;

struct RootWrapper {
    inner: Rc<Root>,
}

impl RootWrapper {
    fn new() -> Self {
        RootWrapper {
            inner: Spawner::spawn_root(),
        }
    }
}

impl Deref for RootWrapper {
    type Target = Rc<Root>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

// SAFETY:
// We never use threads.
unsafe impl Sync for RootWrapper { }
unsafe impl Send for RootWrapper { }

static ROOT: Lazy<RootWrapper> = Lazy::new(|| {
    RootWrapper::new()
});


pub fn main<Platform: PlatformTrait + 'static>() {
    let root = Rc::clone(&ROOT.inner);
    root.main(vec![]);

    // Create and set the platform
    let platform = Platform::new(800, 600) as Box<dyn PlatformTrait + 'static>;
    *ROOT.platform.borrow_mut() = Some(platform);

    let mut tekenen = Tekenen::new(800, 600);

    tekenen.background(tekenen::BLACK);

    println!("All initialized!");
    Platform::set_interval(Box::new(move || {
        ROOT.update(&mut tekenen)
    }), 60);
}