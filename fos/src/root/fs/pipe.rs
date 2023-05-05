use crate::fc::future::{Context, Future, Poll};
use std::{cell::RefCell, pin::Pin, rc::Rc};
use super::descriptor::{ReadableDescriptor, WritableDescriptor, IOError};

struct Shared {
    buffer: String,
    writers: Option<usize>,
}

impl Shared {
    fn is_open(&self) -> Result<(), IOError> {
        if self.writers.unwrap_or(0) != 0 {
            Ok(())
        } else {
            Err(IOError::ChannelClosed)
        }
    }
}

pub struct PipeWriter {
    shared: Rc<RefCell<Shared>>,
}

impl PipeWriter {
    fn clone(&self) -> Result<Self, IOError> {
        let shared = Rc::clone(&self.shared);

        if let Some(ref mut writers) = shared.borrow_mut().writers {
            *writers += 1
        } else {
            return Err(IOError::ChannelClosed)
        }

        Ok(PipeWriter {
            shared
        })
    }
}

impl WritableDescriptor for PipeWriter {
    fn write(&self, data: &str) -> Result<(), IOError> {
        let mut shared = self.shared.as_ref().borrow_mut();

        shared.is_open()?;

        shared.buffer.push_str(data);
        Ok(())
    }

    fn write_char(&self, data: char) -> Result<(), IOError> {
        let mut shared = self.shared.as_ref().borrow_mut();

        shared.is_open()?;

        shared.buffer.push(data);
        Ok(())
    }
}

impl Drop for PipeWriter {
    fn drop(&mut self) {
        let mut shared = self.shared.as_ref().borrow_mut();

        if let Some(ref mut writers) = shared.writers {
            *writers -= 1;
        }
    }
}

pub struct PipeReader {
    shared: Rc<RefCell<Shared>>,
}

struct ReadingTask {
    shared: Rc<RefCell<Shared>>,
}

impl Future for ReadingTask {
    type Output = Result<String, IOError>;

    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<String, IOError>> {
        let mut shared = self.shared.as_ref().borrow_mut();

        let mut data = String::new();

        // take the data in the buffer and set an empty string in its place
        std::mem::swap(&mut data, &mut shared.buffer);

        if !data.is_empty() {
            Poll::Ready(Ok(data))
        } else {

            // If there are no writers left we will never have anything to read.
            shared.is_open()?;

            Poll::Pending
        }
    }
}

impl ReadableDescriptor for PipeReader {
    fn read(&self, len: u32) -> Pin<Box<dyn Future<Output = Result<String, IOError>>>> {
        let future = Box::new(ReadingTask {
            shared: Rc::clone(&self.shared),
        });

        unsafe {
            Pin::new_unchecked(future)
        }
    }

    fn read_char(&self) -> Pin<Box<dyn Future<Output = Result<char, IOError>>>> {
        todo!()
        // let future = ReadingTask {
        //     shared: Rc::clone(&self.shared),
        // };

        // let string = future.await;

        // if let Ok(mut string) = string {
        //     // SAFETY:
        //     // Since we just read a string we know the channel isn't closed.
        //     // Also we know that we read the whole string so the buffer is empty.
        //     // Also the Ok variant always has a string with length > 0.
        //     // Therfore we can just memswap it back.

        //     let char = string.remove(0);
        //     std::mem::swap(&mut string, &mut self.shared.borrow_mut().buffer);

        //     Ok(char)
        // } else {
        //     None
        // }
    }

    fn read_sync(&self, len: u32) -> Result<String, IOError> {
        let mut shared = self.shared.as_ref().borrow_mut();

        let mut data = String::new();

        // take the data in the buffer and set an empty string in its place
        std::mem::swap(&mut data, &mut shared.buffer);

        if !data.is_empty() {
            Ok(data)
        } else {

            // If there are no writers left we will never have anything to read.
            shared.is_open()?;
            
            Err(IOError::Empty)
        }
    }

    fn read_char_sync(&self) -> Result<char, IOError> {
        todo!()
    }
}

impl Drop for PipeReader {
    fn drop(&mut self) {
        let mut shared = self.shared.as_ref().borrow_mut();

        shared.writers = None;
    }
}

pub fn new_pipe() -> (PipeReader, PipeWriter) {
    let shared = Rc::new(RefCell::new(Shared {
        buffer: String::new(),
        writers: Some(1)
    }));

    let reader = PipeReader{ shared: Rc::clone(&shared) };
    let writer = PipeWriter{ shared };

    (reader, writer)
}

#[cfg(test)]
mod test {
    use crate::fc::future::Executor;
    use super::*;

    const STR_A: &str = "a";
    const STR_B: &str = "b";
    const STR_AB: &str = "ab";

    const READ_SIZE: u32 = 100;

    #[test]
    fn transmit_sequntial() {
        let (rx, tx) = new_pipe();

        // First message
        let sent = tx.write(STR_A);
        let recv = Executor::block(rx.read(READ_SIZE));

        assert_eq!(sent, Ok(()));
        assert_eq!(recv, Ok(STR_A.to_string()));

        // Second message
        let sent = tx.write(&STR_B);
        let recv = Executor::block(rx.read(READ_SIZE));

        assert_eq!(sent, Ok(()));
        assert_eq!(recv, Ok(STR_B.to_string()));
    }

    #[test]
    fn transmit_twice() {
        let (rx, tx) = new_pipe();

        // Send bot messages
        let sent1 = tx.write(&STR_A);
        let sent2 = tx.write(&STR_B);

        // Recive both message
        let recv = Executor::block(rx.read(READ_SIZE));

        // Both messages
        assert_eq!(sent1, Ok(()));
        assert_eq!(sent2, Ok(()));
        assert_eq!(recv, Ok(STR_AB.to_string()));
    }

    #[test]
    fn rx_closed() {
        let (rx, tx) = new_pipe();
        drop(rx);

        let sent = tx.write(&STR_A);

        assert_eq!(sent, Err(IOError::ChannelClosed))
    }

    #[test]
    fn tx_closed() {
        let (rx, tx) = new_pipe();
        drop(tx);

        let recv1 = Executor::block(rx.read(READ_SIZE));
        let recv2 = Executor::block(rx.read_char());

        assert_eq!(recv1, Err(IOError::ChannelClosed));
        assert_eq!(recv2, Err(IOError::ChannelClosed));
    }

    #[test]
    fn multiple_senders() {
        let (rx, tx) = new_pipe();

        let tx1 = Rc::new(tx);
        let tx2 = Rc::clone(&tx1);

        let send1 = tx1.write(&STR_A);
        let send2 = tx2.write(&STR_B);

        let recv = Executor::block(rx.read(READ_SIZE));

        assert_eq!(send1, Ok(()));
        assert_eq!(send2, Ok(()));
        assert_eq!(recv, Ok(STR_AB.to_string()));
    }

    #[test]
    fn send_char() {
        let (rx, tx) = new_pipe();

        let send1 = tx.write(&STR_A);
        let send2 = tx.write_char('b');

        let recv = Executor::block(rx.read(READ_SIZE));

        assert_eq!(send1, Ok(()));
        assert_eq!(send2, Ok(()));
        assert_eq!(recv, Ok(STR_AB.to_string()));
    }

    #[test]
    fn read_char() {
        let (rx, tx) = new_pipe();

        let send = tx.write(&STR_AB);

        let recv1 = Executor::block(rx.read_char());
        let recv2 = Executor::block(rx.read(READ_SIZE));

        assert_eq!(send, Ok(()));
        assert_eq!(recv1, Ok('a'));
        assert_eq!(recv2, Ok(STR_B.to_string()));
    }
}
