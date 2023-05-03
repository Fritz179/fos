use crate::{Proc, ROOT};

use crate::fc::channel_handler::{self, Closed, Readable, Writable};

pub enum FileDirectoryPipe {
    File(String),
    Directory(Vec<String>),
    Pipe(channel_handler::RawHandler),
}

pub struct EntryName(String);

pub struct Directory(Vec<(EntryName, InodeTypes)>);

pub enum InodeTypes {
    File(String),
    Directory(Directory),
    Pipe(channel_handler::RawHandler),
}

pub struct Inode {
    inode: InodeTypes,
}

pub struct Fs {
    inode: Directory,
}

impl Fs {
    pub fn new() -> Self {
        Fs {
            inode: Directory(vec![
                (
                    EntryName("mount-file".to_string()),
                    InodeTypes::File("content_of_mount_file".to_string()),
                ),
                (
                    EntryName("mount_folder".to_string()),
                    InodeTypes::Directory(Directory(vec![
                        (
                            EntryName("sub_file_1".to_string()),
                            InodeTypes::File("content_of_sub_file_1".to_string()),
                        ),
                        (
                            EntryName("sub_file_2".to_string()),
                            InodeTypes::File("content_of_sub_file_2".to_string()),
                        ),
                    ])),
                ),
            ]),
        }
    }
}

pub enum OpenError {
    ENOENT,
    ENODIR,
}

impl Proc {
    pub fn open(
        &self,
        filename: String,
    ) -> Result<channel_handler::ChannelHandler<Readable, Closed>, OpenError> {
        let fs = &ROOT.fs;

        for entry in fs.inode.0.iter() {
            if let (name, InodeTypes::File(content)) = entry {
                if name.0 != filename {
                    continue;
                }

                let pipe = self.pipe();

                pipe.write(content);

                return Ok(pipe.close_write());
            }
        }

        Err(OpenError::ENOENT)
    }

    pub fn pipe(&self) -> channel_handler::ChannelHandler<Readable, Writable> {
        let holder = channel_handler::ChannelHandler::new();
        let raw = holder.copy_raw();

        self.descriptor_table.add(raw);

        holder
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
