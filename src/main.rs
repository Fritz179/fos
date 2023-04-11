use std::{cell::RefCell, fmt, rc::Rc, future::Future, pin::Pin, task::{Context, Poll, Waker, RawWaker, RawWakerVTable}, sync::{Arc, Mutex}};

mod platforms;
use platforms::{Event, SDLPlatform, Tekenen};

mod proc;
use proc::*;

mod fs;
pub use fs::*;

mod terminal;
use terminal::Terminal;

mod shell;
use shell::Shell;

mod map;
pub use map::Table;

pub struct Root {
    platform: RefCell<Box<SDLPlatform>>,
    tekenen: Tekenen,
    terminal: Terminal,
    proc: Proc,
}

impl Process for Root {
    fn new(proc: Proc) -> Root {
        Root {
            platform: RefCell::new(platforms::SDLPlatform::new(800, 600)),
            tekenen: Tekenen::new(800, 600),
            terminal: Terminal::new(),
            proc,
        }
    }
}

impl Root {
    fn main(self: &Rc<Self>) {
        let (shell, shell_pid) = self.proc.spawn::<Shell>();

        // pipe stdin to shell stdin
        let self_clone = Rc::clone(&self);
        self.proc.read(
            STDIN,
            Box::new(move |char| {
                let fs = self_clone.proc.fs.upgrade().expect("No Fs");
                fs.write(shell_pid, 0, char);
            }),
        );

        // pipe shell stdout to terminal
        let self_clone = Rc::clone(&self);
        let fs = self_clone.proc.fs.upgrade().expect("No Fs");
        fs.read(
            shell_pid,
            STDOUT,
            Box::new(move |char| {
                self_clone.terminal.write(char);
            }),
        );

        shell.main();
    }

    fn update(&self) -> bool {
        let mut platform = self.platform.borrow_mut();

        while let Some(event) = platform.read_events() {
            match event {
                Event::Quit => {
                    // true indicates to interrupt the loop
                    return true;
                }
                Event::KeyDown { char, keycode, .. } => {
                    if let Some(c) = char {
                        self.proc.write(0, c)
                    } else {
                        println!("{}", keycode)
                    }
                } // _ => {
                  //     println!("Unhandled event: {:?}", event);
                  // }
            }
        }

        self.terminal.render(&self.tekenen, 0);

        platform.display_pixels(&self.tekenen);

        // should not stop
        return false;
    }
}

impl fmt::Debug for Root {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Root")
            //  .field("x", &self.x)
            //  .field("y", &self.y)
            .finish()
    }
}

// #[tokio::main]
// async fn main() {
//     let fs = Rc::new(Fs::new());
//     let spawner = Rc::new(Spawner::new(Rc::clone(&fs)));

//     let (root, _pid) = spawner.spawn::<Root>();
//     root.main();

//     println!("{:?}", spawner);

//     SDLPlatform::set_interval(Box::new(move || {
//         return root.update();
//     }), 60);

// }

struct Task {
    char: char,
    time: u32
}

impl Task {
    fn new(char: char) -> Task {
        Task {
            char,
            time: 0
        }
    }
}

impl Future for Task {
    type Output = u32;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.time >= 10 {
            println!("Ready task: {}, time: {}", self.char, self.time);
            Poll::Ready(self.time)
        } else {
            println!("Updating task: {}, time: {}", self.char, self.time);
            self.get_mut().time += 1;
            Poll::Pending
        }
    }
}

async fn t(c1: char, c2: char) {
    let mut t1 = Task::new(c1);
    let mut t2 = Task::new(c2);

    t1.await;
    t2.await;
}

fn main() {
    // let mut t1 = Task::new('a');
    let mut t1 = t('a', 'b');
    // let mut t2 = Task::new('b');
    let mut t2 = t('d', 'e');

    println!("Starting tasks");

    let ts = unsafe {
        vec![Pin::new_unchecked(&mut t1), Pin::new_unchecked(&mut t2)]
    };
    
    block_on(ts);

    println!("Task Ended");
}

use std::sync::Condvar;

#[derive(Default)]
struct Parker(Mutex<bool>, Condvar);

impl Parker {
    fn park(&self) {
        let mut resumable = self.0.lock().unwrap();
        while !*resumable {
            resumable = self.1.wait(resumable).unwrap();
        }
        *resumable = false;
    }

    fn unpark(&self) {
        *self.0.lock().unwrap() = true;
        self.1.notify_one();
    }
}


#[derive(Clone)]
struct MyWaker {
    parker: Arc<Parker>,
}

fn mywaker_wake(s: &MyWaker) {
    let waker_arc = unsafe { Arc::from_raw(s) };
    waker_arc.parker.unpark();
}

fn mywaker_clone(s: &MyWaker) -> RawWaker {
    let arc = unsafe { Arc::from_raw(s) };
    std::mem::forget(arc.clone()); // increase ref count
    RawWaker::new(Arc::into_raw(arc) as *const (), &VTABLE)
}

const VTABLE: RawWakerVTable = unsafe {
    RawWakerVTable::new(
        |s| mywaker_clone(&*(s as *const MyWaker)),   // clone
        |s| mywaker_wake(&*(s as *const MyWaker)),    // wake
        |s| (*(s as *const MyWaker)).parker.unpark(), // wake by ref (don't decrease refcount)
        |s| drop(Arc::from_raw(s as *const MyWaker)), // decrease refcount
    )
};


fn mywaker_into_waker(s: *const MyWaker) -> Waker {
    let raw_waker = RawWaker::new(s as *const (), &VTABLE);
    unsafe { Waker::from_raw(raw_waker) }
}

fn block_on<F: Future>(mut futures: Vec<Pin<&mut F>>) {

    let parker = Arc::new(Parker::default());
    let mywaker = Arc::new(MyWaker {
        parker: parker.clone(),
    });
    // const waker = unsafe {  };
    let waker = mywaker_into_waker(Arc::into_raw(mywaker));
    let mut cx = Context::from_waker(&waker);

    struct Fholder<F> {
        future: F,
        done: bool
    }

    // SAFETY: we shadow `future` so it can't be accessed again.
    let mut futures: Vec<Fholder<Pin<&mut F>>> = futures.into_iter().map(|mut f| unsafe {
        Fholder {
            future: f,
            done: false,
        }
    }).collect();

    loop {
        let done = &mut true;

        futures.iter_mut().for_each(|future| {
            if !future.done {
                match Future::poll(future.future.as_mut(), &mut cx) {
                    Poll::Ready(val) => { future.done = true },
                    Poll::Pending => { },
                };
            }

            if !future.done {
                *done = false
            }
        });

        if *done {
            break 
        }
    }
}

// Unused ?

