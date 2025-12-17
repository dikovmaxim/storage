use fuse::FileAttr;

pub trait FsManagable {
    fn get_entry_attributes(&self, inode: u128) -> FileAttr;
}