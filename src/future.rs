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
    let t1 = Task::new(c1);
    let t2 = Task::new(c2);

    t1.await;
    t2.await;
}

// fn main() {
//     // let mut t1 = Task::new('a');
//     let mut t1 = t('a', 'b');
//     // let mut t2 = Task::new('b');
//     let mut t2 = t('d', 'e');

//     println!("Starting tasks");

//     let ts = unsafe {
//         vec![Pin::new_unchecked(&mut t1), Pin::new_unchecked(&mut t2)]
//     };
    
//     block_on(ts);

//     println!("Task Ended");
// }

struct FakeWaker { }

fn fake_waker_clone() -> RawWaker {
    panic!("Fake Walker")
}

const VTABLE: RawWakerVTable = RawWakerVTable::new(
    |_| fake_waker_clone(),   // clone
    |_| (),    // wake
    |_| (), // wake by ref (don't decrease refcount)
    |_marker| (), // decrease refcount
);


fn mywaker_into_waker(s: *const FakeWaker) -> Waker {
    let raw_waker = RawWaker::new(s as *const (), &VTABLE);
    unsafe { Waker::from_raw(raw_waker) }
}

struct Fholder<F: Future> {
    future: F,
    done: bool,
    output: Option<F::Output>,
}

fn block_on<F: Future>(futures: Vec<Pin<&mut F>>) -> Vec<Fholder<Pin<&mut F>>> {
    let waker = mywaker_into_waker(&FakeWaker { });
    let mut cx = Context::from_waker(&waker);



    // SAFETY: we shadow `future` so it can't be accessed again.
    let mut futures: Vec<Fholder<Pin<&mut F>>> = futures.into_iter().map(|f| 
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

    return futures
}

// Unused ?

