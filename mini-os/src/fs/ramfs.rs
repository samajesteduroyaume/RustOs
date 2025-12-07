use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::Mutex;

use crate::fs::vfs_core::*;

/// Structure représentant une inode en mémoire
struct RamInodeData {
    id: InodeId,
    mode: FileMode,
    file_type: FileType,
    size: u64,
    content: Vec<u8>,
    // Pour les répertoires : map de nom -> inode_id
    children: BTreeMap<String, InodeId>,
    nlinks: u32,
    uid: u32,
    gid: u32,
    atime: u64,
    mtime: u64,
    ctime: u64,
}

impl RamInodeData {
    fn new(id: InodeId, mode: FileMode, file_type: FileType) -> Self {
        Self {
            id,
            mode,
            file_type,
            size: 0,
            content: Vec::new(),
            children: BTreeMap::new(),
            nlinks: 1,
            uid: 0,
            gid: 0,
            atime: 0,
            mtime: 0,
            ctime: 0,
        }
    }
}

// Old RamInode implementation removed. 
// See RamInodeOps below for the active implementation.

// Removed broken initial implementation.
// Use RamFileSystemRef below.

pub struct RamSuperblock {
    fs_id: FsId,
}

impl Superblock for RamSuperblock {
    fn fs_name(&self) -> &str {
        "ramfs"
    }

    fn fs_id(&self) -> FsId {
        self.fs_id
    }

    fn block_size(&self) -> u32 {
        4096
    }

    fn total_blocks(&self) -> u64 {
        0 // Unlimited (RAM)
    }

    fn free_blocks(&self) -> u64 {
        0
    }

    fn total_inodes(&self) -> u64 {
        0
    }

    fn free_inodes(&self) -> u64 {
        0
    }

    fn is_readonly(&self) -> bool {
        false
    }

    fn root_inode(&self) -> InodeId {
        1 // Root is always 1
    }
}

// === Refactored Implementation with Inner State ===

struct RamFsInner {
    inodes: Mutex<BTreeMap<InodeId, Arc<Mutex<RamInodeData>>>>,
    next_inode_id: Mutex<InodeId>,
}

pub struct RamFileSystemRef {
    inner: Arc<RamFsInner>,
    sb: Arc<RamSuperblock>,
}

impl RamFileSystemRef {
    pub fn new() -> Self {
        let sb = Arc::new(RamSuperblock { fs_id: 1 });
        let inner = Arc::new(RamFsInner {
            inodes: Mutex::new(BTreeMap::new()),
            next_inode_id: Mutex::new(2),
        });

        let root_data = Arc::new(Mutex::new(RamInodeData::new(
            1,
            FileMode::new(0o755),
            FileType::Directory,
        )));
        inner.inodes.lock().insert(1, root_data);

        Self { inner, sb }
    }
}

impl FileSystemOps for RamFileSystemRef {
    fn superblock(&self) -> Arc<dyn Superblock> {
        self.sb.clone()
    }

    fn get_inode(&self, inode_id: InodeId) -> VfsResult<Arc<Mutex<dyn InodeOps>>> {
        let inodes = self.inner.inodes.lock();
        if let Some(data) = inodes.get(&inode_id) {
            let inode_ops = RamInodeOps {
                data: data.clone(),
                fs_inner: self.inner.clone(),
            };
            Ok(Arc::new(Mutex::new(inode_ops)))
        } else {
            Err(VfsError::NotFound)
        }
    }

    fn sync(&self) -> VfsResult<()> { Ok(()) }
    fn unmount(&self) -> VfsResult<()> { Ok(()) }
}

struct RamInodeOps {
    data: Arc<Mutex<RamInodeData>>,
    fs_inner: Arc<RamFsInner>,
}

impl InodeOps for RamInodeOps {
    fn read(&self, offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        let data = self.data.lock();
        if offset >= data.content.len() as u64 {
            return Ok(0);
        }
        let start = offset as usize;
        let available = data.content.len() - start;
        let len = core::cmp::min(available, buf.len());
        buf[0..len].copy_from_slice(&data.content[start..start + len]);
        Ok(len)
    }

    fn write(&mut self, offset: u64, buf: &[u8]) -> VfsResult<usize> {
        let mut data = self.data.lock();
        let end = offset as usize + buf.len();
        if end > data.content.len() {
            data.content.resize(end, 0);
        }
        data.content[offset as usize..end].copy_from_slice(buf);
        if end as u64 > data.size {
            data.size = end as u64;
        }
        Ok(buf.len())
    }

    fn stat(&self) -> VfsResult<FileStat> {
        let data = self.data.lock();
        let mut stat = FileStat::new(data.id, data.file_type);
        stat.mode = data.mode;
        stat.size = data.size;
        stat.nlinks = data.nlinks;
        Ok(stat)
    }

    fn lookup(&self, name: &str) -> VfsResult<InodeId> {
        let data = self.data.lock();
        if data.file_type != FileType::Directory { return Err(VfsError::NotDirectory); }
        if name == "." { return Ok(data.id); }
        if name == ".." { return Err(VfsError::NotSupported); } // Handled by dentry
        data.children.get(name).copied().ok_or(VfsError::NotFound)
    }

    fn create(&mut self, name: &str, mode: FileMode, file_type: FileType) -> VfsResult<InodeId> {
        let mut data = self.data.lock();
        if data.file_type != FileType::Directory { return Err(VfsError::NotDirectory); }
        if data.children.contains_key(name) { return Err(VfsError::AlreadyExists); }

        // Alloc ID
        let mut next_id = self.fs_inner.next_inode_id.lock();
        let id = *next_id;
        *next_id += 1;

        let new_data = Arc::new(Mutex::new(RamInodeData::new(id, mode, file_type)));
        self.fs_inner.inodes.lock().insert(id, new_data);
        
        data.children.insert(name.into(), id);
        Ok(id)
    }

    fn unlink(&mut self, name: &str) -> VfsResult<()> {
        let mut data = self.data.lock();
        if data.file_type != FileType::Directory { return Err(VfsError::NotDirectory); }
        if data.children.remove(name).is_some() { Ok(()) } else { Err(VfsError::NotFound) }
    }

    fn mkdir(&mut self, name: &str, mode: FileMode) -> VfsResult<InodeId> {
        self.create(name, mode, FileType::Directory)
    }

    fn rmdir(&mut self, name: &str) -> VfsResult<()> {
        self.unlink(name) // Simplified checks
    }

    fn readdir(&self) -> VfsResult<Vec<DirEntry>> {
        let data = self.data.lock();
        if data.file_type != FileType::Directory { return Err(VfsError::NotDirectory); }
        
        let mut entries = Vec::new();
        entries.push(DirEntry::new(data.id, ".".into(), FileType::Directory));
        // entries.push(DirEntry::new(0, "..".into(), FileType::Directory)); // Optional/Skip

        for (name, &id) in &data.children {
            // Get type from inner
            let inodes = self.fs_inner.inodes.lock();
            if let Some(child_data) = inodes.get(&id) {
                let t = child_data.lock().file_type;
                entries.push(DirEntry::new(id, name.clone(), t));
            }
        }
        Ok(entries)
    }

    fn truncate(&mut self, size: u64) -> VfsResult<()> {
        let mut data = self.data.lock();
        data.content.resize(size as usize, 0);
        data.size = size;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_ramfs_file_creation() {
        let fs = RamFileSystemRef::new();
        // Get root inode (ID 1)
        let root = fs.get_inode(1).expect("Should get root inode");
        
        // Root is ID 1.
        let file_id = root.lock().create("test.txt", FileMode::new(0o644), FileType::Regular)
            .expect("Should create file");
            
        // Check lookup
        let found_id = root.lock().lookup("test.txt").expect("Should find file");
        assert_eq!(file_id, found_id);
    }

    #[test_case]
    fn test_ramfs_read_write() {
        let fs = RamFileSystemRef::new();
        let root = fs.get_inode(1).expect("Should get root inode");
        
        let file_id = root.lock().create("data.bin", FileMode::new(0o644), FileType::Regular)
            .expect("Should create file");
            
        let file_inode = fs.get_inode(file_id).expect("Should get file inode");
        
        let data = b"Hello RamFS";
        let written = file_inode.lock().write(0, data).expect("Should write");
        assert_eq!(written, data.len());
        
        let mut buf = [0u8; 20];
        let read = file_inode.lock().read(0, &mut buf).expect("Should read");
        assert_eq!(read, data.len());
        assert_eq!(&buf[..read], data);
    }

    #[test_case]
    fn test_ramfs_mkdir() {
        let fs = RamFileSystemRef::new();
        let root = fs.get_inode(1).expect("Should get root inode");
        
        let dir_id = root.lock().mkdir("subdir", FileMode::new(0o755)).expect("Should mkdir");
        
        let dir_inode = fs.get_inode(dir_id).expect("Should get dir inode");
        assert_eq!(dir_inode.lock().stat().expect("stat").file_type, FileType::Directory);
        
        // Check parent link in root
        let found_id = root.lock().lookup("subdir").expect("Should find subdir");
        assert_eq!(found_id, dir_id);
    }

    #[test_case]
    fn test_ramfs_not_found() {
        let fs = RamFileSystemRef::new();
        let root = fs.get_inode(1).expect("Should get root inode");
        
        let err = root.lock().lookup("nonexistent");
        assert!(err.is_err());
    }
}
