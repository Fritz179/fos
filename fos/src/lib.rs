mod platforms;
use std::{cell::{RefCell, UnsafeCell}, rc::{Rc, Weak}, ops::Deref, borrow::BorrowMut};

use once_cell::sync::Lazy;

pub use platforms::*;

pub mod fc;

mod shell;

mod root;
pub use root::*;

// SAFETY:
// We never use multiple threads.

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

unsafe impl Sync for RootWrapper { }
unsafe impl Send for RootWrapper { }
static ROOT: Lazy<RootWrapper> = Lazy::new(|| {
    RootWrapper::new()
});

pub fn main<Platform: PlatformTrait + 'static>() {
    ROOT.main();

    let platform = Platform::new(800, 600);
    let platform = platform as Box<dyn PlatformTrait + 'static>;

    let mut curr_platform = ROOT.platform.borrow_mut();
    *curr_platform = Some(platform);

    drop(curr_platform);

    println!("Dropped");


    let mut tekenen = Tekenen::new(800, 600);

    Platform::set_interval(Box::new(move || {
        return ROOT.update(&mut tekenen);
    }), 60);
}