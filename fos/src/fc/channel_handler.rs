use std::rc::Rc;

use super::string_channel::*;


pub struct Readable;
pub struct Writable;
pub struct Closed;

pub struct Handler<Readabylity, Writability> {
    readability: std::marker::PhantomData<Readabylity>,
    writabiliry: std::marker::PhantomData<Writability>,
    tx: Option<Rc<Tx>>,
    rx: Option<Rx>
}

impl Handler<Readable, Writable> {
    fn new() -> Self {
        let (tx, rx) = new_string_channel();

        Self {
            readability: std::marker::PhantomData::<Readable>,
            writabiliry: std::marker::PhantomData::<Writable>,
            tx: Some(Rc::new(tx)), 
            rx: Some(rx)
        }
    }
}

impl<R, W> Handler<R, W> {
    pub fn copy(&self) -> Handler<Closed, W> {
        Handler {
            readability: std::marker::PhantomData::<Closed>,
            writabiliry: std::marker::PhantomData::<W>,
            tx: if let Some(ref tx) = self.tx { Some(Rc::clone(tx)) } else { None },
            rx: None
        }
    }
}

impl<T> Handler<Readable, T> {
    async fn read(&self) -> Option<String>  {
        self.rx.as_ref()?.read().await
    }

    fn close_read(self) -> Handler<Closed, T> {
        Handler {
            readability: std::marker::PhantomData::<Closed>,
            writabiliry: std::marker::PhantomData::<T>,
            tx: self.tx, 
            rx: None,
        }
    }
}

impl<T> Handler<T, Writable> {
    fn send(&self, str: &str) -> Option<()> {
        self.tx.as_ref()?.send(str)
    }

    fn close_write(self) -> Handler<T, Closed> {
        Handler {
            readability: std::marker::PhantomData::<T>,
            writabiliry: std::marker::PhantomData::<Closed>,
            tx: None, 
            rx: self.rx,
        }
    }
}

pub fn new_channel_handler() -> Handler<Readable, Writable> {
    Handler::new()
}

 #[cfg(test)]
mod test {
    use super::*;
    use super::super::future::Executor;

    const STR_A: &str = "a";
    // const STR_B: &str = "b";
    // const STR_AB: &str = "ab";

    #[test]
    fn simple_handler() {
        let channel = new_channel_handler();

        // First message
        let sent = channel.send(STR_A);
        let recv = Executor::block(channel.read());

        assert_eq!(sent, Some(()));
        assert_eq!(recv, Some(STR_A.to_string()));
    }

    #[test]
    fn copy_handler() {
        let reciver = new_channel_handler();
        let sender = reciver.copy();

        // First message
        let sent = sender.send(STR_A);
        let recv = Executor::block(reciver.read());

        assert_eq!(sent, Some(()));
        assert_eq!(recv, Some(STR_A.to_string()));
    }

    #[test]
    fn close_writeable() {
        let reciver = new_channel_handler();
        let sender = reciver.copy();
        let reciver = reciver.close_write();

        // First message
        let sent = sender.send(STR_A);
        let recv = Executor::block(reciver.read());

        assert_eq!(sent, Some(()));
        assert_eq!(recv, Some(STR_A.to_string()));
    }
}