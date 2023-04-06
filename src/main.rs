use std::{cell::RefCell};

mod platforms;
use platforms::{SDLPlatform, Event, Tekenen};

mod terminal;
use terminal::Terminal;

pub struct Root {
    platform: RefCell<Box<SDLPlatform>>,
    tekenen: Tekenen,
    terminal: Terminal,
    fs: Fs,
}

type Readers = Vec<Box<dyn Fn(char)>>;

pub struct Fs {
    raw_readers: RefCell<Vec<RefCell<Readers>>>,
    processes_to_raw: RefCell<Vec<RefCell<Vec<FileDescriptor>>>>,
}

type FileDescriptor = u32;
impl Fs {
    pub const fn new() -> Self {
        Fs {
            raw_readers: RefCell::new(vec![]),
            processes_to_raw: RefCell::new(vec![]),
        }
    }

    pub fn spawn<Child: Process>(&self) -> Child {
        let child_pid = self.processes_to_raw.borrow().len() as Pid;
        let child = Child::new(child_pid as u32);
        self.processes_to_raw.borrow_mut().push(RefCell::new(vec![]));

        self.open(child_pid); // stdin
        self.open(child_pid); // stdout
        self.open(child_pid); // stderr

        child.main(&self);

        return child
    }

    pub fn open(&self, pid: Pid) -> FileDescriptor {
        let raw_descriptor = self.raw_readers.borrow().len() as u32;
        self.raw_readers.borrow_mut().push(RefCell::new(vec![]));

        let mapper = self.processes_to_raw.borrow();
        let pid_mapping = mapper.get(pid as usize).expect("No PID mapping");
        pid_mapping.borrow_mut().push(raw_descriptor);

        return (pid_mapping.borrow().len() - 1) as FileDescriptor;
    }

    pub fn read(&self, pid: Pid, descriptor: FileDescriptor, callback: Box<dyn Fn(char)>) {
        let raw = self.processes_to_raw.borrow();
        let mapper = raw.get(pid as usize).expect("No PID mapping");
        let raw = *mapper.borrow().get(descriptor as usize).expect("No descriptor");
        
        let mapper = self.raw_readers.borrow_mut();
        let mut readers = mapper.get(raw as usize).expect("No raders").borrow_mut();
        readers.push(callback);
    }

    pub fn write(&self, pid: Pid, descriptor: FileDescriptor, c: char) {
        let binding = self.processes_to_raw.borrow();
        let mapper = binding.get(pid as usize).expect("No PID mapping");
        let raw = *mapper.borrow().get(descriptor as usize).expect("No descriptor");
        let binding = self.raw_readers.borrow();
        let readers = binding.get(raw as usize).expect("No raders");
        
        for reader in readers.borrow().iter() {
            reader(c);
        }
    }
}

pub type Pid = u32;

pub trait Process {
    fn new(pid: Pid) -> Self;
    fn main(&self, fs: &Fs);
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