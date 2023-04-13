use std::{pin::Pin};

pub use std::{future::Future, task::{Poll, Context}};

use debug_cell::RefCell;

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

use crate::Table;

async fn tu(c1: char) {
    let t1 = Task::new(c1);
    t1.await;
}

pub fn tst() {
    let executor = Executor::new();

    let t1 = tu('c');
    let t2 = tu('d');

    executor.add_task(t1);
    executor.add_task(t2);

    loop {
        if executor.execute() {
            break
        }
    }
}

struct Executor {
    tasks: Table<Box<RefCell<dyn Future<Output = ()>>>>,
}

impl Executor {
    fn new() -> Self {
        Executor {
            tasks: Table::new(),
        }
    }

    fn add_task<F: Future<Output = ()> + 'static>(&self, task: F) {
        self.tasks.add(Box::new(RefCell::new(task)));
    }

    fn execute(&self) -> bool {
        let count = self.tasks.filter(Box::new(|task: &Box<RefCell<dyn Future<Output = ()>>>| -> bool {
            let mut pinned = unsafe {
                Pin::new_unchecked( task.borrow_mut())
            };

            let fake_cx: u64 = 0;
            let mut fake_cx = unsafe { std::mem::transmute(&fake_cx) };

            match Future::poll(pinned.as_mut(), &mut fake_cx) {
                Poll::Ready(()) => {
                    false
                },
                Poll::Pending => {
                    true
                 },
            }
        }));

        println!("{count}");
        return count == 0;
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

