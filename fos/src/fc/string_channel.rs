use super::future::{Context, Future, Poll};
use std::{cell::RefCell, pin::Pin, rc::{Rc, Weak}};

struct Shared {
    buffer: String,
    closed: bool,
}

pub struct Tx {
    shared: Rc<RefCell<Shared>>,
}

impl Tx {
    pub fn send(&self, data: &str) -> Option<()> {
        let mut shared = self.shared.as_ref().borrow_mut();

        if !shared.closed {
            shared.buffer.push_str(data);
            Some(())
        } else {
            None
        }
    }

    pub fn send_char(&self, data: char) -> Option<()> {
        let mut shared = self.shared.as_ref().borrow_mut();

        if !shared.closed {
            shared.buffer.push(data);
            Some(())
        } else {
            None
        }
    }
}

impl Drop for Tx {
    fn drop(&mut self) {
        let mut shared = self.shared.as_ref().borrow_mut();

        shared.closed = true;
    }
}

pub struct WeakTx {
    shared: Weak<RefCell<Shared>>,
}

impl WeakTx {
    pub fn send(&self, data: &str) -> Option<()> {
        let shared = self.shared.upgrade()?;
        let mut shared = shared.borrow_mut();

        if !shared.closed {
            shared.buffer.push_str(data);
            Some(())
        } else {
            None
        }
    }

    pub fn send_char(&self, data: char) -> Option<()> {
        let shared = self.shared.upgrade()?;
        let mut shared = shared.borrow_mut();

        if !shared.closed {
            shared.buffer.push(data);
            Some(())
        } else {
            None
        }
    }
}

pub struct Rx {
    shared: Rc<RefCell<Shared>>,
}

struct ReadingTask {
    shared: Rc<RefCell<Shared>>,
}

impl Future for ReadingTask {
    type Output = Option<String>;

    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<String>> {
        let mut shared = self.shared.as_ref().borrow_mut();

        let mut data = String::new();

        // take the data in the buffer and set an empty string in its place
        std::mem::swap(&mut data, &mut shared.buffer);

        if shared.closed {
            Poll::Ready(None)
        } else if !data.is_empty() {
            Poll::Ready(Some(data))
        } else {
            Poll::Pending
        }
    }
}

impl Rx {
    pub async fn read(&self) -> Option<String> {
        let future = ReadingTask {
            shared: Rc::clone(&self.shared),
        };

        future.await
    }

    pub async fn read_char(&self) -> Option<char> {
        let future = ReadingTask {
            shared: Rc::clone(&self.shared),
        };

        let string = future.await;

        if let Some(mut string) = string {
            // SAFETY:
            // Since we just read a string we know the channel isn't closed.
            // Also we know that we read the whole string so the buffer is empty.
            // Also the some variant always has a string with length > 0.
            // Therfore we can just memswap it back.

            let char = string.remove(0);
            std::mem::swap(&mut string, &mut self.shared.borrow_mut().buffer);

            Some(char)
        } else {
            None
        }
    }
}

impl Drop for Rx {
    fn drop(&mut self) {
        let mut shared = self.shared.as_ref().borrow_mut();

        shared.closed = true;
    }
}

pub fn new_string_channel() -> (Tx, Rx) {
    let shared = Rc::new(RefCell::new(Shared {
        buffer: String::new(),
        closed: false,
    }));

    let tx = Tx {
        shared: Rc::clone(&shared),
    };

    let rx = Rx { shared };

    (tx, rx)
}

#[cfg(test)]
mod test {
    use super::super::future::Executor;
    use super::*;

    const STR_A: &str = "a";
    const STR_B: &str = "b";
    const STR_AB: &str = "ab";

    #[test]
    fn transmit_sequntial() {
        let (tx, rx) = new_string_channel();

        // First message
        let sent = tx.send(STR_A);
        let recv = Executor::block(rx.read());

        assert_eq!(sent, Some(()));
        assert_eq!(recv, Some(STR_A.to_string()));

        // Second message
        let sent = tx.send(&STR_B);
        let recv = Executor::block(rx.read());

        assert_eq!(sent, Some(()));
        assert_eq!(recv, Some(STR_B.to_string()));
    }

    #[test]
    fn transmit_twice() {
        let (tx, rx) = new_string_channel();

        // Send bot messages
        let sent1 = tx.send(&STR_A);
        let sent2 = tx.send(&STR_B);

        // Recive both message
        let recv = Executor::block(rx.read());

        // Both messages
        assert_eq!(sent1, Some(()));
        assert_eq!(sent2, Some(()));
        assert_eq!(recv, Some(STR_AB.to_string()));
    }

    #[test]
    fn rx_closed() {
        let (tx, rx) = new_string_channel();
        drop(rx);

        let sent = tx.send(&STR_A);

        assert_eq!(sent, None)
    }

    #[test]
    fn tx_closed() {
        let (tx, rx) = new_string_channel();
        drop(tx);

        let recv1 = Executor::block(rx.read());
        let recv2 = Executor::block(rx.read_char());

        assert_eq!(recv1, None);
        assert_eq!(recv2, None);
    }

    #[test]
    fn multiple_senders() {
        let (tx, rx) = new_string_channel();

        let tx1 = Rc::new(tx);
        let tx2 = Rc::clone(&tx1);

        let send1 = tx1.send(&STR_A);
        let send2 = tx2.send(&STR_B);

        let recv = Executor::block(rx.read());

        assert_eq!(send1, Some(()));
        assert_eq!(send2, Some(()));
        assert_eq!(recv, Some(STR_AB.to_string()));
    }

    #[test]
    fn send_char() {
        let (tx, rx) = new_string_channel();

        let send1 = tx.send(&STR_A);
        let send2 = tx.send_char('b');

        let recv = Executor::block(rx.read());

        assert_eq!(send1, Some(()));
        assert_eq!(send2, Some(()));
        assert_eq!(recv, Some(STR_AB.to_string()));
    }

    #[test]
    fn read_char() {
        let (tx, rx) = new_string_channel();

        let send = tx.send(&STR_AB);

        let recv1 = Executor::block(rx.read_char());
        let recv2 = Executor::block(rx.read());

        assert_eq!(send, Some(()));
        assert_eq!(recv1, Some('a'));
        assert_eq!(recv2, Some(STR_B.to_string()));
    }
}
