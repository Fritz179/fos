use std::{cell::RefCell, rc::Rc};

mod platforms;
use platforms::{SDLPlatform, Event, Tekenen};

mod terminal;
use terminal::Terminal;

pub struct Root {
    platform: RefCell<Box<SDLPlatform>>,
    tekenen: Tekenen,
    terminal: Rc<Terminal>,
    fs: Fs,
}

type Readers = Vec<Box<dyn Fn(char)>>;

pub struct Fs {
    readers_map: RefCell<Vec<Readers>>,
    pid_map: RefCell<Vec<Vec<FileDescriptor>>>,
}

type FileDescriptor = u32;
impl Fs {
    pub const fn new() -> Self {
        Fs {
            readers_map: RefCell::new(vec![]),
            pid_map: RefCell::new(vec![]),
        }
    }

    pub fn spawn<Child: Process>(&self) -> Rc<Child> {
        let mut pid_map = self.pid_map.borrow_mut();
    
        let child_pid = pid_map.len() as Pid;
        pid_map.push(vec![]);

        drop(pid_map);

        let child = Rc::new(Child::new(child_pid as u32));


        self.open(child_pid); // stdin
        self.open(child_pid); // stdout
        self.open(child_pid); // stderr

        child.main(&self);

        return child
    }

    pub fn open(&self, pid: Pid) -> FileDescriptor {
        let mut readers_map = self.readers_map.borrow_mut();

        let raw_descriptor = readers_map.len() as u32;
        readers_map.push(vec![]);

        let mut pid_map = self.pid_map.borrow_mut();

        let pid_mapping = pid_map.get_mut(pid as usize).expect("No PID mapping");
        let file_id = pid_mapping.len() as FileDescriptor;
        pid_mapping.push(raw_descriptor);

        return file_id;
    }

    pub fn read(&self, pid: Pid, descriptor: FileDescriptor, callback: Box<dyn Fn(char)>) {
        let pid_map = self.pid_map.borrow_mut();
        let pid_map = pid_map.get(pid as usize).expect("No PID mapping");
        let raw = *pid_map.get(descriptor as usize).expect("No descriptor");
        
        let mut readers = self.readers_map.borrow_mut();
        let readers = readers.get_mut(raw as usize).expect("No raders");
        readers.push(callback);
    }

    pub fn write(&self, pid: Pid, descriptor: FileDescriptor, c: char) {
        let pid_map = self.pid_map.borrow_mut();
        let pid_map = pid_map.get(pid as usize).expect("No PID mapping");
        let raw = *pid_map.get(descriptor as usize).expect("No descriptor");
        
        let readers = self.readers_map.borrow();
        let readers = readers.get(raw as usize).expect("No raders");
        
        for reader in readers.iter() {
            reader(c);
        }
    }
}

pub type Pid = u32;

pub trait Process {
    fn new(pid: Pid) -> Self;
    fn main(self: &Rc<Self>, fs: &Fs);
}

impl Root {
    fn new() -> Root {
        let fs = Fs::new();

        Root {
            platform: RefCell::new(platforms::SDLPlatform::new(800, 600)),
            tekenen: Tekenen::new(800, 600),
            terminal: fs.spawn::<Terminal>(),
            fs,
        }
    }

    fn update(&self) -> bool {
        let mut platform = self.platform.borrow_mut();

        while let Some(event) = platform.read_events() {
            match event {
                Event::Quit => {
                    // true indicates to interrupt the loop
                    return true;
                },
                Event::KeyDown { repeat: false, char, keycode, .. } => {
                    if let Some(c) = char {
                        self.fs.write(0, 0, c)
                    } else {
                        println!("{}", keycode)
                    }
                }
                _ => {
                    println!("Unhandled event: {:?}", event);
                }
            }
        }

        self.terminal.render(&self.tekenen, 0);

        platform.display_pixels(&self.tekenen);

        // should not stop
        return false
    }
}

fn main() {
    let root = Root::new();

    SDLPlatform::set_interval(Box::new(move || {
        return root.update();
    }), 60);
}