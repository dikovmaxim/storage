#![allow(unused_imports)]

use fuse::FileAttr;

pub trait FsManagable {
    fn get_entry_attributes(&self, inode: u128) -> FileAttr;
}

pub trait CopyableFs {
    fn copy_entry_with_new_id(&self) -> Self;
}