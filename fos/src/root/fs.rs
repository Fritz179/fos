use std::error::Error;
use std::rc::Rc;
use std::fmt;
// use std::cell::RefCell;

use crate::Proc;

#[derive(Clone)]
pub struct FileDescriptor(usize);
pub const STDIN: FileDescriptor = FileDescriptor(0);
pub const STDOUT: FileDescriptor = FileDescriptor(1);
pub const STDERR: FileDescriptor = FileDescriptor(2);

use crate::fc::channel::{Tx, Rx, new_channel};

pub type TxRx = (Rc<Tx<char>>, Option<Rx<char>>);

pub enum FileDirectoryPipe {
    File(String),
    Directory(Vec<String>),
    Pipe(TxRx)
}

pub struct EntryName(String);

pub struct Directory(Vec<(EntryName, InodeTypes)>);

pub enum InodeTypes {
    File(String),
    Directory(Directory),
    Pipe(TxRx)
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

impl fmt::Debug for Fs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Fs")
            //  .field("x", &self.x)
            //  .field("y", &self.y)
            .finish()
    }
}

pub enum OpenError {
    ENOENT,
    ENODIR,
}

impl Proc {
    pub fn open(&self, filename: String) -> Result<FileDescriptor, OpenError> {
        let fs = &self.root.fs;

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

        return Err(OpenError::ENOENT);
    }

    pub fn pipe(&self) -> FileDescriptor {
        let (tx, rx) = new_channel();
        let channel = FileDirectoryPipe::Pipe((Rc::new(tx), Some(rx)));

        let id = self.descriptor_table.add(channel);

        return FileDescriptor(id);
    }

    pub async fn read(&self, descriptor: FileDescriptor) -> Option<char> {
        let fd = self.descriptor_table.get(descriptor.0);

        let rx = std::cell::Ref::map(fd, |node| {
            match node {
                FileDirectoryPipe::File(f) => { &None }
                FileDirectoryPipe::Directory(f) => { &None }
                FileDirectoryPipe::Pipe(txrx) => { &txrx.1 }
            }
        });

        if let Some(ref rx) = *rx {
            // println!("Reading: {descriptor}");

            return rx.read().await
        } else {
            None
        }
    }

    pub fn write(&self, descriptor: FileDescriptor, content: &str) -> Option<()> {
        let fd = self.descriptor_table.get(descriptor.0);

        // println!("Writng: {descriptor}, {char}");

        match &*fd {
            FileDirectoryPipe::File(f) => { None }
            FileDirectoryPipe::Directory(f) => { None }
            FileDirectoryPipe::Pipe(txrx) => {
                let tx = &txrx.0;

                for char in content.chars() {
                    tx.send(char);
                }

                None
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