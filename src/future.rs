use std::pin::Pin;

pub use std::{future::Future, task::{Poll, Context}};

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
        if self.time >= 2 {
            println!("Ready task: {}, time: {}", self.char, self.time);
            Poll::Ready(self.time)
        } else {
            println!("Updating task: {}, time: {}", self.char, self.time);
            self.get_mut().time += 1;
            Poll::Pending
        }
    }
}

async fn t(c1: char, c2: char) -> u32 {
    let t1 = Task::new(c1);
    let t2 = Task::new(c2);

    println!("1");
    t1.await;
    println!("2");
    let t = t2.await;
    println!("3");
    return t;
}

pub fn tst() {
    // let mut t1 = Task::new('a');
    let mut t1 = t('a', 'b');
    // let mut t2 = Task::new('b');
    let mut t2 = t('d', 'e');

    println!("Starting tasks");

    let ts = vec![&mut t1, &mut t2];
    
    let t = block_on(ts);

    println!("Task Ended");
}

fn block_on<F: Future>(futures: Vec<&mut F>) -> Vec<F::Output> {
    let x: u64 = 0;
    let mut cx = unsafe { std::mem::transmute(&x) };

    struct Fholder<'a, F: Future> {
        future: Pin<&'a mut F>,
        done: bool,
        output: Option<F::Output>,
    }

    let mut futures = futures.into_iter().map(|f| unsafe {
        Pin::new_unchecked(f)
    });

    // SAFETY: we shadow `future` so it can't be accessed again.
    let mut futures: Vec<Fholder<F>> = futures.into_iter().map(|f| 
        Fholder {
            future: f,
            done: false,
            output: None
        }
    ).collect();

    loop {
        let done = &mut true;

        futures.iter_mut().for_each(|future| {
            if !future.done {
                match Future::poll(future.future.as_mut(), &mut cx) {
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

    let t: Vec<F::Output> = futures.into_iter().map(|t| t.output.expect("No output")).collect();

    return t
}

// Unused ?

