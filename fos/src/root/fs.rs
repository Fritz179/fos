pub mod descriptor;
pub mod pipe;

use descriptor::{ReadableWritableDescriptor, ReadableDescriptor, WritableDescriptor};
use pipe::{new_pipe, PipeReader, PipeWriter};

use crate::{Proc, ROOT};

pub enum FileDirectoryPipe {
    File(String),
    Directory(Vec<String>),
}

pub struct Name(pub String);

pub struct Directory(pub Vec<(Name, Inode)>);

pub enum Inode {
    File(String),
    Directory(Directory),
}

pub struct Fs {
    mount: Directory,
}

impl Fs {
    pub fn new() -> Self {
        Fs {
            mount: Directory(vec![
                (
                    Name("mount-file".to_string()),
                    Inode::File("content_of_mount_file".to_string()),
                ),
                (
                    Name("mount_folder".to_string()),
                    Inode::Directory(Directory(vec![
                        (
                            Name("sub_file_1".to_string()),
                            Inode::File("content_of_sub_file_1".to_string()),
                        ),
                        (
                            Name("sub_file_2".to_string()),
                            Inode::File("content_of_sub_file_2".to_string()),
                        ),
                    ])),
                ),
            ]),
        }
    }
}

#[derive(Debug)]
pub enum OpenError {
    NoEntry,
    NoDirectoy,
    IsDirectory,
    IsFile
}

impl Proc {
    pub fn open(
        &self,
        filename: String,
    ) -> Result<ReadableWritableDescriptor<PipeReader, PipeWriter>, OpenError> {
        let fs = &ROOT.fs;

        for (name, entry) in fs.mount.0.iter() {
            if name.0 != filename {
                continue;
            }

            if let Inode::File(content) = entry {
                
                let pipe = self.pipe();

                pipe.write(content);

                return Ok(pipe);
            }
        }

        Err(OpenError::NoEntry)
    }

    pub fn open_dir(&self, dirname: String) ->  Result<&Directory, OpenError> {
        let fs = &ROOT.fs;

        for (name, entry) in fs.mount.0.iter() {
            if name.0 != dirname {
                continue;
            }

            if let Inode::Directory(directory) = entry {
                return Ok(directory);
            }

            return Err(OpenError::IsFile)
        }

        Err(OpenError::NoEntry)
    }

    pub fn pipe(&self) -> ReadableWritableDescriptor<PipeReader, PipeWriter> {
        let (reader, writer) = new_pipe();

        ReadableWritableDescriptor::<PipeReader, PipeWriter>::new(reader, writer)
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
