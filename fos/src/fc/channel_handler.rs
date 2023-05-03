use std::rc::Rc;

use super::string_channel::*;

pub struct Readable;
pub struct Writable;
pub struct Closed;

pub struct RawHandler {
    tx: Option<Rc<Tx>>,
    rx: Option<Rx>,
}

impl RawHandler {
    pub async fn read(&self) -> Option<String> {
        self.rx.as_ref()?.read().await
    }

    pub async fn read_char(&self) -> Option<char> {
        self.rx.as_ref()?.read_char().await
    }

    pub fn send(&self, str: &str) -> Option<()> {
        self.tx.as_ref()?.send(str)
    }

    pub fn send_char(&self, char: char) -> Option<()> {
        self.tx.as_ref()?.send_char(char)
    }

    fn copy(&self) -> RawHandler {
        let tx = if let Some(ref tx) = self.tx {
            Some(Rc::clone(tx))
        } else {
            None
        };

        RawHandler { tx, rx: None }
    }
}

pub struct ChannelHandler<Readabylity, Writability> {
    readability: std::marker::PhantomData<Readabylity>,
    writabiliry: std::marker::PhantomData<Writability>,
    pub raw: RawHandler,
}

impl ChannelHandler<Readable, Writable> {
    pub fn new() -> Self {
        let (tx, rx) = new_string_channel();

        Self {
            readability: std::marker::PhantomData::<Readable>,
            writabiliry: std::marker::PhantomData::<Writable>,
            raw: RawHandler {
                tx: Some(Rc::new(tx)),
                rx: Some(rx),
            },
        }
    }
}

impl<R, W> ChannelHandler<R, W> {
    pub fn copy(&self) -> ChannelHandler<Closed, W> {
        ChannelHandler {
            readability: std::marker::PhantomData::<Closed>,
            writabiliry: std::marker::PhantomData::<W>,
            raw: self.raw.copy(),
        }
    }

    pub fn copy_raw(&self) -> RawHandler {
        self.raw.copy()
    }
}

impl<T> ChannelHandler<Readable, T> {
    pub async fn read(&self) -> Option<String> {
        self.raw.read().await
    }

    pub async fn read_char(&self) -> Option<char> {
        self.raw.read_char().await
    }

    pub fn close_read(self) -> ChannelHandler<Closed, T> {
        ChannelHandler {
            readability: std::marker::PhantomData::<Closed>,
            writabiliry: std::marker::PhantomData::<T>,
            raw: self.raw,
        }
    }
}

impl<T> ChannelHandler<T, Writable> {
    pub fn write(&self, str: &str) -> Option<()> {
        self.raw.send(str)
    }

    pub fn write_char(&self, char: char) -> Option<()> {
        self.raw.send_char(char)
    }

    pub fn close_write(self) -> ChannelHandler<T, Closed> {
        ChannelHandler {
            readability: std::marker::PhantomData::<T>,
            writabiliry: std::marker::PhantomData::<Closed>,
            raw: self.raw,
        }
    }
}

pub fn new_channel_handler() -> ChannelHandler<Readable, Writable> {
    ChannelHandler::new()
}

#[cfg(test)]
mod test {
    use super::super::future::Executor;
    use super::*;

    const STR_A: &str = "a";
    // const STR_B: &str = "b";
    // const STR_AB: &str = "ab";

    #[test]
    fn simple_handler() {
        let channel = new_channel_handler();

        // First message
        let sent = channel.write(STR_A);
        let recv = Executor::block(channel.read());

        assert_eq!(sent, Some(()));
        assert_eq!(recv, Some(STR_A.to_string()));
    }

    #[test]
    fn copy_handler() {
        let reciver = new_channel_handler();
        let sender = reciver.copy();

        // First message
        let sent = sender.write(STR_A);
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
        let sent = sender.write(STR_A);
        let recv = Executor::block(reciver.read());

        assert_eq!(sent, Some(()));
        assert_eq!(recv, Some(STR_A.to_string()));
    }
}
