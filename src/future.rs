use std::{pin::Pin, process::Output, rc::Rc};

pub use std::{future::Future, task::{Poll, Context}};

use debug_cell::{RefCell, RefMut};

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
    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
        println!("Task polled! {}", self.char);
        if self.time >= 5 {
            println!("Ready task: {}, time: {}", self.char, self.time);
            Poll::Ready(self.time)
        } else {
            println!("Updating task: {}, time: {}", self.char, self.time);
            self.get_mut().time += 1;
            Poll::Pending
        }
    }
}

async fn t(c1: char) -> u32 {
    let t1 = Task::new(c1);
    return t1.await;
}

async fn tu(c1: char) {
    let t1 = Task::new(c1);
    t1.await;
}

// global executor with dynamic return type => Box / Option<Box>

pub fn tst() {
    let mut tvec = vec![];

    let mut t1 = t('a');
    tvec.push(&mut t1);

    println!("Starting tasks");

    // let ts: i32 = vec![&mut t1];
    
    block_all(tvec);

    println!("Task Ended");

    tst2()
}

fn tst2() {
    let executor = Executor::new();

    let mut t1 = tu('c');
    let mut t2 = tu('d');

    executor.add_task(&mut t1);
    executor.add_task(&mut t2);

    executor.execute();
}

struct Poller<'a, F: Future<Output = ()> + ?Sized> {
    task: RefCell<Pin<&'a mut F>>,
}

impl<'a, F: Future<Output = ()> + ?Sized> Poller<'a, F> {
    fn poll(&self) {
        let fake_cx: u64 = 0;
        let mut fake_cx: Context = unsafe { std::mem::transmute(&fake_cx) };

        match Future::poll(self.task.borrow_mut().as_mut(), &mut fake_cx) {
            Poll::Ready(()) => {

            },
            Poll::Pending => {  },
        };
    }
}

struct Executor<'a> {
    tasks: RefCell<Vec<Box<Poller<'a, dyn Future<Output = ()>>>>>,
}

impl<'a> Executor<'a> {
    fn new() -> Executor<'a> {
        Executor {
            tasks: RefCell::new(vec![]),
        }
    }

    fn add_task<F: Future<Output = ()> + 'a>(&self, task: &'a mut F) {
        let mut tasks = self.tasks.borrow_mut();

        // let task = unsafe {
        //     Pin::new(&mut (task as &'a mut dyn Future<Output = ()>) )
        // }; // as Pin<&'a mut dyn Future<Output = ()> + !Unpin>;

        let mut task = task as &'a mut dyn Future<Output = ()>;

        let poller = Poller {
            task: RefCell::new(unsafe { std::mem::transmute(Pin::new_unchecked(task)) })
        };

        tasks.push(Box::new(poller));
    }

    fn execute(&self) {
        self.tasks.borrow().iter().for_each(|poller| {
            poller.poll();
        })
    }
}

// block one single task
fn block<F: Future>(future: &mut F) -> F::Output {
    let fake_cx: u64 = 0;
    let mut fake_cx = unsafe { std::mem::transmute(&fake_cx) };

    // SAFETY: we shadow `future` so it can't be accessed again.
    let mut future = unsafe { Pin::new_unchecked(future) };

    loop {
        match Future::poll(future.as_mut(), &mut fake_cx) {
            Poll::Ready(val) => {
                return  val;
            },
            Poll::Pending => { },
        };
    }
}

// block a vector of tasks
fn block_all<F: Future>(futures: Vec<&mut F>) -> Vec<F::Output> {
    let fake_cx: u64 = 0;
    let mut fake_cx = unsafe { std::mem::transmute(&fake_cx) };

    struct FutureHolder<'a, F: Future> {
        future: Pin<&'a mut F>,
        done: bool,
        output: Option<F::Output>,
    }

    // SAFETY: we shadow `future` so it can't be accessed again.
    let mut futures: Vec<FutureHolder<F>> = futures.into_iter().map(|f| 
        FutureHolder {
            future: unsafe { Pin::new_unchecked(f) },
            done: false,
            output: None
        }
    ).collect();

    loop {
        let done = &mut true;

        futures.iter_mut().for_each(|future| {
            if !future.done {
                match Future::poll(future.future.as_mut(), &mut fake_cx) {
                    Poll::Ready(val) => {
                        future.done = true;
                        future.output = Some(val);
                    },
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
    };

    // return only the outputs, in same order as recived order
    futures.into_iter().map(|t| t.output.expect("Promis must be completed")).collect()
}

