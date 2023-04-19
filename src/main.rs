mod platforms;
use platforms::{SDLPlatform, tekenen::Tekenen, Platform};

pub mod fc;

mod shell;

mod root;
pub use root::*;


fn main() {
    let root = Spawner::spawn_root();

    let mut tekenen = Tekenen::new(800, 600);

    SDLPlatform::set_interval(&mut move || {
        return root.update(&mut tekenen);
    }, 60);
}