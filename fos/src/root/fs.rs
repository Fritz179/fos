use std::rc::Rc;
// use std::cell::RefCell;

use crate::{Proc, ROOT};

#[derive(Clone)]
pub struct FileDescriptor(usize);
pub const STDIN: FileDescriptor = FileDescriptor(0);
pub const STDOUT: FileDescriptor = FileDescriptor(1);
pub const STDERR: FileDescriptor = FileDescriptor(2);

use crate::fc::string_channel::*;

pub struct ChannelHolder {
    tx: Rc<Tx>,
    rx: Option<Rx>
}

pub enum FileDirectoryPipe {
    File(String),
    Directory(Vec<String>),
    Pipe(ChannelHolder)
}

pub struct EntryName(String);

pub struct Directory(Vec<(EntryName, InodeTypes)>);

pub enum InodeTypes {
    File(String),
    Directory(Directory),
    Pipe(ChannelHolder)
}

pub struct Inode {
    inode: InodeTypes
}

pub struct Fs {
    inode: Directory
}

impl Fs {
    pub fn new() -> Self {
        Fs {
            inode: Directory(vec![
                (EntryName("mount-file".to_string()), InodeTypes::File("content_of_mount_file".to_string())),
                (EntryName("mount_folder".to_string()), InodeTypes::Directory(Directory(vec![
                    (EntryName("sub_file_1".to_string()), InodeTypes::File("content_of_sub_file_1".to_string())),
                    (EntryName("sub_file_2".to_string()), InodeTypes::File("content_of_sub_file_2".to_string()))
                ])))
            ])
        }
    }
}

pub enum OpenError {
    ENOENT,
    ENODIR,
}

impl Proc {
    pub fn open(&self, filename: String) -> Result<FileDescriptor, OpenError> {
        let fs = &ROOT.fs;

        for entry in fs.inode.0.iter() {
            if let (name, InodeTypes::File(content)) = entry {

                if name.0 != filename {
                    continue;
                }

                let pipe = self.pipe();

                self.write(pipe.clone(), content);


                return Ok(pipe);
            }
        }

        Err(OpenError::ENOENT)
    }

    pub fn pipe(&self) -> FileDescriptor {
        let (tx, rx) = new_string_channel();
        let channel = FileDirectoryPipe::Pipe(ChannelHolder {
            tx: Rc::new(tx), 
            rx: Some(rx) 
        });

        let id = self.descriptor_table.add(channel);

        FileDescriptor(id)
    }

    pub async fn read(&self, descriptor: FileDescriptor) -> Option<String> {
        let fd = self.descriptor_table.get(descriptor.0);

        let rx = std::cell::Ref::map(fd, |node| {
            match node {
                FileDirectoryPipe::File(_) => { &None }
                FileDirectoryPipe::Directory(_) => { &None }
                FileDirectoryPipe::Pipe(channel) => { &channel.rx }
            }
        });

        if let Some(ref rx) = *rx {
            // println!("Reading: {descriptor}");

            rx.read().await
        } else {
            None
        }
    }

    pub async fn read_char(&self, descriptor: FileDescriptor) -> Option<char> {
        let fd = self.descriptor_table.get(descriptor.0);

        let rx = std::cell::Ref::map(fd, |node| {
            match node {
                FileDirectoryPipe::File(_) => { &None }
                FileDirectoryPipe::Directory(_) => { &None }
                FileDirectoryPipe::Pipe(channel) => { &channel.rx }
            }
        });

        if let Some(ref rx) = *rx {
            rx.read_char().await
        } else {
            None
        }
    }

    pub fn write(&self, descriptor: FileDescriptor, content: &str) -> Option<()> {
        let fd = self.descriptor_table.get(descriptor.0);

        // println!("Writng: {descriptor}, {char}");

        match &*fd {
            FileDirectoryPipe::File(_) => { None }
            FileDirectoryPipe::Directory(_) => { None }
            FileDirectoryPipe::Pipe(channel) => {
                let tx = &channel.tx;

                tx.send(content)
            }
        }
    }


    pub fn write_char(&self, descriptor: FileDescriptor, content: char) -> Option<()> {
        let fd = self.descriptor_table.get(descriptor.0);

        // println!("Writng: {descriptor}, {char}");

        match &*fd {
            FileDirectoryPipe::File(_) => { None }
            FileDirectoryPipe::Directory(_) => { None }
            FileDirectoryPipe::Pipe(channel) => {
                let tx = &channel.tx;

                tx.send_char(content)
            }
        }
    }
}

// pipes -> multiple sender, single reciver
// using Futures -> read one char per Future
// open also has await? Performance = more state

// -	Regular or ordinary file
// d	Directory file
// l	Link file
// b	Block special file => buffered access, chunks of data
// p	Named pipe file => interproces communication
// c	Character special file => direct access, byte by byte
// s	Socket file => ip:socket