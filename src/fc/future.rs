use std::pin::Pin;

pub use std::{future::Future, task::{Poll, Context}};

use std::cell::RefCell;

use crate::fc::table::Table;

pub struct Executor {
    tasks: Table<Box<RefCell<dyn Future<Output = ()>>>>,
    queue: RefCell<Vec<Box<RefCell<dyn Future<Output = ()>>>>>,
}

impl Executor {
    pub const fn new() -> Self {
        Executor {
            tasks: Table::new(),
            queue: RefCell::new(vec![]),
        }
    }

    pub fn add_task<F: Future<Output = ()> + 'static>(&self, task: F) {
        self.queue.borrow_mut().push(Box::new(RefCell::new(task)));
    }

    pub fn execute(&self) -> bool {
        let mut queue = self.queue.borrow_mut();

        while queue.len() != 0 {
            self.tasks.add(queue.pop().unwrap());
        };

        drop(queue);

        let count = self.tasks.filter(&|task: &Box<RefCell<dyn Future<Output = ()>>>| -> bool {
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
        });

        println!("{count}");
        return count == 0;
    }
}

impl std::fmt::Debug for Executor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Executor")
            //  .field("x", &self.x)
            //  .field("y", &self.y)
            .finish()
    }
}

impl Executor {
    // block one single task
    pub fn block<F: Future>(mut future: F) -> F::Output {
        let fake_cx: u64 = 0;
        let mut fake_cx = unsafe { std::mem::transmute(&fake_cx) };

        // SAFETY: we shadow `future` so it can't be accessed again.
        let mut future = unsafe { Pin::new_unchecked(&mut future) };

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
    pub fn block_all<F: Future>(mut futures: Vec<F>) -> Vec<F::Output> {
        let fake_cx: u64 = 0;
        let mut fake_cx = unsafe { std::mem::transmute(&fake_cx) };

        struct FutureHolder<'a, F: Future> {
            future: Pin<&'a mut F>,
            done: bool,
            output: Option<F::Output>,
        }

        // SAFETY: we shadow `future` so it can't be accessed again.
        let mut futures: Vec<FutureHolder<F>> = futures.iter_mut().map(|f: &mut F| 
            FutureHolder {
                future: unsafe { Pin::new_unchecked( f) },
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
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;

    struct Task {
        id: char,
        count: i32,
        report: Rc<RefCell<Vec<char>>>
    }
    
    impl Task {
        async fn new_value(id: char, count: i32, report: Rc<RefCell<Vec<char>>>) -> char {
            let task = Task {
                id,
                count,
                report,
            };

            task.await
        }

        async fn new_global(id: char, count: i32, report: Rc<RefCell<Vec<char>>>) {
            let task = Task {
                id,
                count,
                report,
            };

            task.await;
        }
    }
    
    impl Future for Task {
        type Output = char;
        fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
            if self.count <= 0 {
                Poll::Ready(self.id)
            } else {
                self.report.borrow_mut().push(self.id);
                self.get_mut().count -= 1;
                Poll::Pending
            }
        }
    }

    #[test]
    fn block() {
        let report = Rc::new(RefCell::new(vec![]));
    
        let a = Task::new_value('a', 2, Rc::clone(&report));
    
        let value = Executor::block(a);

        assert_eq!(value, 'a');
        assert_eq!(*report.borrow(), vec!['a', 'a']);
    }

    #[test]
    fn block_all_a() {
        let report = Rc::new(RefCell::new(vec![]));
    
        let a = Task::new_value('a', 3, Rc::clone(&report));
        let b = Task::new_value('b', 2, Rc::clone(&report));
    
        let value = Executor::block_all(vec![a, b]);

        assert_eq!(value, vec!['a', 'b']);
        assert_eq!(*report.borrow(), vec!['a', 'b', 'a', 'b', 'a']);
    }

    #[test]
    fn block_all_b() {
        let report = Rc::new(RefCell::new(vec![]));
    
        let a = Task::new_value('a', 2, Rc::clone(&report));
        let b = Task::new_value('b', 3, Rc::clone(&report));
    
        let value = Executor::block_all(vec![a, b]);

        assert_eq!(value, vec!['a', 'b']);
        assert_eq!(*report.borrow(), vec!['a', 'b', 'a', 'b', 'b']);
    }
    
    #[test]
    fn simple_executor() {
        let executor = Executor::new();
        let report = Rc::new(RefCell::new(vec![]));
    
        let a = Task::new_global('a', 2, Rc::clone(&report));
    
        executor.add_task(a);
    
        loop {
            if executor.execute() {
                break
            }
        };

        assert_eq!(*report.borrow(), vec!['a', 'a']);
    }

    #[test]
    fn executor() {
        let executor = Executor::new();
        let report = Rc::new(RefCell::new(vec![]));
    
        let a = Task::new_global('a', 3, Rc::clone(&report));
        let b = Task::new_global('b', 2, Rc::clone(&report));
    
        executor.add_task(a);
        executor.add_task(b);
    
        loop {
            if executor.execute() {
                break
            }
        };

        assert_eq!(*report.borrow(), vec!['a', 'b', 'a', 'b', 'a']);
    }

    #[test]
    fn executor_mixed() {
        let executor = Executor::new();
        let report = Rc::new(RefCell::new(vec![]));
    
        let a = Task::new_global('a', 4, Rc::clone(&report));
        let b = Task::new_global('b', 2, Rc::clone(&report));
        let c = Task::new_global('c', 2, Rc::clone(&report));
    
        executor.add_task(a);
        executor.add_task(b);
    
        executor.execute();
        executor.add_task(c);

        executor.execute();
        executor.execute();
        executor.execute();


        assert_eq!(*report.borrow(), vec!['a', 'b', 'a', 'b', 'c', 'a', 'c', 'a']);
    }
}