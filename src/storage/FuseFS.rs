use fuse::{
    FileType, Request, FileAttr, Filesystem, ReplyAttr, ReplyDirectory,
    ReplyData, ReplyEntry, ReplyOpen, Session,
};
use time::Timespec;
use std::ffi::OsStr;
use std::path::Path;
use log::info;
use crate::manager::Kvs::Kvs;
use crate::storage::Inode::Inode;
use crate::storage::Inode::InodeKind;
use crate::storage::Directory::Directory;
use libc;

const TTL: Timespec = Timespec { sec: 1, nsec: 0 }; // 1s cache

const ROOT_INO: u64 = 1;
const FILE1_INO: u64 = 2;
const FILE2_INO: u64 = 3;
// Define a basic structure for the filesystem
struct MyFS {
    storage: Kvs, // variable for a KVS connection
}

impl MyFS {
    fn new(storage: Kvs) -> Self {
        MyFS { storage  }
    }

    pub fn get_kvs(&self) -> &Kvs {
        &self.storage
    }
}

fn file_attr(ino: u64) -> FileAttr {
    match ino {
        ROOT_INO => FileAttr {
            ino: ROOT_INO,
            size: 0,
            blocks: 0,
            atime: Timespec::new(0, 0),
            mtime: Timespec::new(0, 0),
            ctime: Timespec::new(0, 0),
            crtime: Timespec::new(0, 0),
            kind: FileType::Directory,
            perm: 0o755,
            nlink: 2,
            uid: 1000,
            gid: 1000,
            rdev: 0,
            flags: 0,
        },
        FILE1_INO | FILE2_INO => FileAttr {
            ino,
            size: 32, // or dummy_content.len() as u64
            blocks: 1,
            atime: Timespec::new(0, 0),
            mtime: Timespec::new(0, 0),
            ctime: Timespec::new(0, 0),
            crtime: Timespec::new(0, 0),
            kind: FileType::RegularFile,
            perm: 0o644,
            nlink: 1,
            uid: 1000,
            gid: 1000,
            rdev: 0,
            flags: 0,
        },
        _ => unreachable!(),
    }
}


impl Filesystem for MyFS {
    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        println!("getattr ino={}", ino);

        match ino {
            ROOT_INO | FILE1_INO | FILE2_INO => {
                let attr = file_attr(ino);
                reply.attr(&TTL, &attr);
            }
            _ => reply.error(libc::ENOENT),
        }
    }

    /// Map (parent inode, name) -> inode
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) { // IMPLEMENTED
        println!("lookup parent={}, name={:?}", parent, name);

        let inode_used = Inode::get_by_id(parent as u128, &mut self.get_kvs());
        println!("Inode fetched from KVS: {:?}", inode_used);

        //if inode result is error, return ENOENT
        if inode_used.is_err() {
            reply.error(libc::ENOENT);
            return;
        }


        let inode = inode_used.unwrap();
        let attr = inode.get_attribute(&mut self.get_kvs());

        reply.entry(&TTL, &attr, 0);
    }

    fn readdir(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        println!("readdir ino={}, offset={}", ino, offset);

        let inode_used = Inode::get_by_id(ino as u128, &mut self.get_kvs());
        if inode_used.is_err() {
            reply.error(libc::ENOENT);
            return;
        }
        println!("Inode fetched from KVS: {:?}", inode_used);

        let target: (InodeKind, u128) = inode_used.unwrap().get_target(&mut self.get_kvs()).unwrap();
        println!("Inode target fetched: {:?}", target);

        if target.0 != InodeKind::Directory {
            reply.error(libc::ENOTDIR);
            return;
        }

        let directory = Directory::get_by_id(target.1, &mut self.get_kvs());
        if directory.is_err() {
            reply.error(libc::ENOENT);
            return;
        }
        let dir = directory.unwrap();
        let updated_reply = dir.get_directory_reply(reply, offset, &mut self.get_kvs());
        reply = updated_reply;
        reply.ok();
    }

    /// Called before read; must *not* return ENOSYS or the kernel thinks "function not implemented".
    fn open(&mut self, _req: &Request, ino: u64, flags: u32, reply: ReplyOpen) {
        println!("open ino={}, flags={}", ino, flags);

        match ino {
            FILE1_INO | FILE2_INO => {
                // fh = 0 (no real file handle), keep flags as-is
                reply.opened(0, flags);
            }
            _ => reply.error(libc::ENOENT),
        }
    }

    fn read(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        size: u32,
        reply: ReplyData,
    ) {
        println!("read ino={}, offset={}, size={}", ino, offset, size);

        if ino != FILE1_INO && ino != FILE2_INO {
            reply.error(libc::ENOENT);
            return;
        }

        let dummy_content = b"Hello from networking filesystem!";
        let start = offset as usize;
        let end = (start + size as usize).min(dummy_content.len());

        reply.data(&dummy_content[start..end]);
    }

    // mkdir etc. can stay, but should probably return a fresh inode instead of 1.
}

pub fn initFuse(kvs: Kvs) {
    // Initialize logger
    env_logger::init();

    // Set up FUSE session
    let fs = MyFS::new(kvs);
    let mount_point = Path::new("/home/max/Desktop/storage/testfs");

    let mut session = Session::new(fs, &mount_point, &[]).unwrap();
    session.run().unwrap();

    // Your program will now be running and waiting for filesystem operations
    println!("Filesystem mounted at: {:?}", mount_point);
}
