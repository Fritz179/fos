mod platforms;
pub use platforms::*;

pub mod fc;

mod shell;

mod root;
pub use root::*;

pub fn main<Platform: PlatformTrait + 'static>() {
    let root = Spawner::spawn_root::<Platform>();

    let mut tekenen = Tekenen::new(800, 600);

    Platform::set_interval(Box::new(move || {
        return root.update(&mut tekenen);
    }), 60);
}