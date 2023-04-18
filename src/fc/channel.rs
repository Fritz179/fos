use super::future::{Future, Poll, Context};
use std::{pin::Pin, rc::Rc, collections::VecDeque, cell::RefCell};

struct Shared<T> {
    buffer: VecDeque<T>,
    closed: bool,
}

pub struct Tx<T> {
    shared: Rc<RefCell<Shared<T>>>
}

impl<T> Tx<T> {
    pub fn send(&self, data: T) -> Option<()> {
        let mut shared = self.shared.as_ref().borrow_mut();

        if !shared.closed {
            shared.buffer.push_back(data);
            Some(())
        } else {
            None
        }

        
    }
}

impl<T> Drop for Tx<T> {
    fn drop(&mut self) {
        let mut shared = self.shared.as_ref().borrow_mut();

        shared.closed = true;
    }
}


pub struct Rx<T> {
    shared: Rc<RefCell<Shared<T>>>
}

struct ReadingTask<T> {
    shared: Rc<RefCell<Shared<T>>>
}

impl<T> Future for ReadingTask<T> {
    type Output = Option<T>;
    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<T>> {
        let mut shared = self.shared.as_ref().borrow_mut();

        let data = shared.buffer.pop_front();

        if shared.closed {
            Poll::Ready(None)
        } else  if let Some(data) = data {
            Poll::Ready(Some(data))
        } else {
            Poll::Pending
        }
    }
}

impl<T> Rx<T> {
    pub async fn read(&self) -> Option<T> {
        let future = ReadingTask {
            shared: Rc::clone(&self.shared)
        };

        future.await
    }
}

impl<T> Drop for Rx<T> {
    fn drop(&mut self) {
        let mut shared = self.shared.as_ref().borrow_mut();

        shared.closed = true;
    }
}


pub fn new_channel<T>() -> (Rc<Tx<T>>, Rx<T>) {
    let shared = Rc::new(
        RefCell::new(Shared {
            buffer: VecDeque::new(),
            closed: false,
        })
    );

    let tx = Tx {
        shared: Rc::clone(&shared)
    };

    let rx = Rx {
        shared
    };

    (
        Rc::new(tx),
        rx
    )
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::future::Executor;

    #[test]
    fn transmit_sequntial() {
        let (tx, rx) = new_channel();

        // First message
        let sent = tx.send(5);
        let recv = Executor::block(rx.read());

        assert_eq!(sent, Some(()));
        assert_eq!(recv, Some(5));
        
        // Second message
        let sent = tx.send(6);
        let recv = Executor::block(rx.read());

        assert_eq!(sent, Some(()));
        assert_eq!(recv, Some(6));
    }

    #[test]
    fn transmit_twice() {
        let (tx, rx) = new_channel();

        // Send bot messages
        let sent1 = tx.send(5);
        let sent2 = tx.send(6);

        // Recive both message
        let recv1 = Executor::block(rx.read());
        let recv2 = Executor::block(rx.read());

        // First message
        assert_eq!(sent1, Some(()));
        assert_eq!(recv1, Some(5));
        
        // Second message
        assert_eq!(sent2, Some(()));
        assert_eq!(recv2, Some(6));
    }

    #[test]
    fn rx_closed() {
        let (tx, rx) = new_channel();
        drop(rx);

        let sent = tx.send(5);

        assert_eq!(sent, None)
    }

    #[test]
    fn tx_closed() {
        let (tx, rx) = new_channel::<()>();
        drop(tx);

        let recv = Executor::block(rx.read());

        assert_eq!(recv, None)
    }

    #[test]
    fn multiple_senders() {
        let (tx1, rx) = new_channel();
        let tx2 = Rc::clone(&tx1);

        let send1 = tx1.send(5);
        let send2 = tx2.send(6);

        let recv1 = Executor::block(rx.read());
        let recv2 = Executor::block(rx.read());

        assert_eq!(send1, Some(()));
        assert_eq!(send2, Some(()));
        assert_eq!(recv1, Some(5));
        assert_eq!(recv2, Some(6));
    }
}