use std::{future::Future, pin::Pin};

#[derive(Debug, PartialEq)]
pub enum IOError {
    ChannelClosed,
    Empty
}

pub trait ReadableDescriptor {
    fn read(&self, len: u32) -> Pin<Box<dyn Future<Output = Result<String, IOError>>>>;
    fn read_char(&self) -> Pin<Box<dyn Future<Output = Result<char, IOError>>>>;
    fn read_sync(&self, len: u32) -> Result<String, IOError>;
    fn read_char_sync(&self) -> Result<char, IOError>;
}

pub trait WritableDescriptor {
    fn write(&self, str: &str) -> Result<(), IOError>;
    fn write_char(&self, char: char) -> Result<(), IOError>;
}

pub struct ReadableWritableDescriptor<Reader: ReadableDescriptor, Writer: WritableDescriptor>  {
    reader: Reader,
    writer: Writer,
}

impl<R: ReadableDescriptor, W: WritableDescriptor> ReadableWritableDescriptor<R, W> {
    pub fn new(reader: R, writer: W) -> Self {
        Self {
            reader,
            writer
        }
    }
}

impl<R: ReadableDescriptor, W: WritableDescriptor> ReadableDescriptor for ReadableWritableDescriptor<R, W> {
    fn read(&self, len: u32) -> Pin<Box<dyn Future<Output = Result<String, IOError>>>> {
        self.reader.read(len)
    }

    fn read_char(&self) -> Pin<Box<dyn Future<Output = Result<char, IOError>>>> {
        self.reader.read_char()
    }

    fn read_sync(&self, len: u32) -> Result<String, IOError> {
        self.reader.read_sync(len)
    }

    fn read_char_sync(&self) -> Result<char, IOError> {
        self.reader.read_char_sync()
    }
}

impl<R: ReadableDescriptor, W: WritableDescriptor> WritableDescriptor for ReadableWritableDescriptor<R, W> {
    fn write(&self, str: &str) -> Result<(), IOError> {
        self.writer.write(str)
    }

    fn write_char(&self, char: char) -> Result<(), IOError> {
        self.writer.write_char(char)
    }
}

impl<R: ReadableDescriptor, W: WritableDescriptor> ReadableWritableDescriptor<R, W> {
    fn close_read(self) -> W {
        self.writer
    }

    fn close_write(self) -> R {
        self.reader
    }
}