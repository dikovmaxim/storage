#![allow(unused_imports)]
use fuse::{
    FileType, Request, FileAttr, Filesystem, ReplyAttr, ReplyDirectory,
    ReplyData, ReplyEntry, ReplyOpen, ReplyCreate, Session,
};
use time::Timespec;
use std::ffi::OsStr;
use std::path::Path;
use log::info;
use crate::manager::Kvs::Kvs;
use crate::storage::Inode::Inode;
use crate::storage::Inode::InodeKind;
use crate::storage::Directory::Directory;
use crate::storage::File::File;
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
    <fn new(storage: Kvs) -> Self {>
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

    //IMPLEMENTED
    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        let inode_used = Inode::get_by_id(ino as u128, &mut self.get_kvs());
        if inode_used.is_err() {
            reply.error(libc::ENOENT);
            return;
        }

        let inode = inode_used.unwrap();
        let attr = inode.get_attribute(&mut self.get_kvs());
        reply.attr(&TTL, &attr);
    }

    //IMPLEMENTED
    /// Map (parent inode, name) -> inode
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        println!("lookup called with parent={}, name={:?}", parent, name);

        // Fetch the parent inode
        let inode_used = Inode::get_by_id(parent as u128, &mut self.get_kvs());
        if inode_used.is_err() {
            reply.error(libc::ENOENT);
            return;
        }

        let inode = inode_used.unwrap();
        let target = inode.get_target(&mut self.get_kvs());
        if target.is_err() {
            reply.error(libc::ENOENT);
            return;
        }

        // Check if the file exists in the directory
        let directory = Directory::get_by_id(target.unwrap().1, &mut self.get_kvs());
        if directory.is_err() {
            reply.error(libc::ENOENT);
            return;
        }

        let dir = directory.unwrap();
        let entry = dir.find_entry(name.to_str().unwrap(), &mut self.get_kvs());
        if entry.is_none() {
            reply.error(libc::ENOENT);
            return;
        }

        // If the file exists, return its attributes
        let entry_inode = entry.unwrap();
        let attr = entry_inode.get_attribute(&mut self.get_kvs());
        reply.entry(&TTL, &attr, 0);
    }

    //IMPLEMENTED
    fn readdir(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        let inode_used = Inode::get_by_id(ino as u128, &mut self.get_kvs());
        if inode_used.is_err() {
            reply.error(libc::ENOENT);
            return;
        }
        let target: (InodeKind, u128) = inode_used.unwrap().get_target(&mut self.get_kvs()).unwrap();

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


    fn create(&mut self, _req: &Request, parent: u64, name: &OsStr, mode: u32, umask: u32, reply: ReplyCreate) { //Create a new file
        println!("create parent={}, name={:?}, mode={}, umask={}", parent, name, mode, umask);

        // Fetch the parent inode
        let parent_inode = Inode::get_by_id(parent as u128, &mut self.get_kvs());
        if parent_inode.is_err() {
            reply.error(libc::ENOENT);
            return;
        }

        let parent_inode = parent_inode.unwrap();
        let parent_target = parent_inode.get_target(&mut self.get_kvs());
        if parent_target.is_err() || parent_target.unwrap().0 != InodeKind::Directory {
            reply.error(libc::ENOTDIR);
            return;
        }

        // Create a new inode for the file
        let new_instances = File::create_new(
            parent as u128,
            name.to_str().unwrap().to_string(),
            &mut self.get_kvs()
        );

        let (new_inode, new_file) = new_instances.unwrap();
        let attr = new_inode.get_attribute(&mut self.get_kvs());

        reply.created(&TTL, &attr, 0, 0, 0); // Updated to use ReplyCreate's `created` method
    }
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
