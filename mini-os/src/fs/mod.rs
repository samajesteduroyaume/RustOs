pub mod fd;
pub mod vfs_core;
pub mod vfs_inode;
pub mod vfs_dentry;
pub mod vfs_mount;
pub mod ramfs;

pub use fd::{FileDescriptor, FileDescriptorTable, FileDescriptorManager, OpenMode, FD_MANAGER};
pub use vfs_core::*;
pub use vfs_inode::{Inode, InodeCache, INODE_CACHE, get_or_create_inode, put_inode};
pub use vfs_dentry::{Dentry, DentryCache, DENTRY_CACHE, path_lookup as vfs_path_lookup, create_root_dentry};
pub use vfs_mount::{MountPoint, MountFlags, MountManager, MOUNT_MANAGER, mount_root, mount_fs, unmount_fs};
pub use ramfs::RamFileSystemRef;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::sync::Arc;
use spin::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    static ref ROOT_DENTRY: Mutex<Option<Arc<Mutex<Dentry>>>> = Mutex::new(None);
}

/// Helper: Initialize default RamFS
pub fn init_vfs() -> VfsResult<()> {
    // Mount RamFS as root
    let fs = alloc::sync::Arc::new(RamFileSystemRef::new());
    
    // Create root inode
    let root_inode_ops = fs.get_inode(1)?; // Root ID is 1
    
    let root_inode = super::fs::get_or_create_inode(
        fs.superblock().fs_id(),
        1,
        super::fs::FileType::Directory,
        root_inode_ops
    );
    
    // Create root dentry
    let root_dentry = create_root_dentry(root_inode);
    *ROOT_DENTRY.lock() = Some(root_dentry.clone());
    
    // Register mount (Optional for this simplified version but good for completeness)
    // mount_root(fs)?; 
    
    Ok(())
}

/// Helper: Lookup path using global root
fn path_lookup(path: &str) -> VfsResult<Arc<Mutex<Dentry>>> {
    let root = ROOT_DENTRY.lock().as_ref().ok_or(VfsError::IoError)?.clone();
    vfs_path_lookup(path, root)
}

/// Helper: Check if path is directory
pub fn is_dir(path: &str) -> bool {
    match path_lookup(path) {
        Ok(dentry) => {
            let inode = dentry.lock().inode.clone();
            let is_directory = inode.lock().stat.file_type == FileType::Directory;
            is_directory
        },
        Err(_) => false
    }
}

/// Helper: List directory
pub fn vfs_ls(path: &str) -> VfsResult<Vec<String>> {
    let dentry = path_lookup(path)?;
    let inode = dentry.lock().inode.clone();
    
    // Lock inode node to read
    // Note: VFS architecture here distinguishes Inode object (wrapper) vs InodeOps
    // We need to access InodeOps. `inode` is Arc<Mutex<Inode>>.
    // Inode struct has `ops: Arc<Mutex<dyn InodeOps>>`.
    
    let ops = inode.lock().ops.clone();
    let entries = ops.lock().readdir()?;
    
    let mut names = Vec::new();
    for entry in entries {
        names.push(entry.name);
    }
    Ok(names)
}

/// Helper: Read file content
pub fn vfs_read_file(path: &str) -> VfsResult<Vec<u8>> {
    let dentry = path_lookup(path)?;
    let inode = dentry.lock().inode.clone();
    
    let mut buf = Vec::new();
    let stat = inode.lock().ops.lock().stat()?;
    buf.resize(stat.size as usize, 0);
    
    inode.lock().ops.lock().read(0, &mut buf)?;
    Ok(buf)
}

/// Helper: Write file content (Create or Overwrite)
pub fn vfs_write_file(path: &str, content: &[u8]) -> VfsResult<()> {
    // Try to open existing
    match path_lookup(path) {
        Ok(dentry) => {
            let inode = dentry.lock().inode.clone();
            inode.lock().ops.lock().truncate(0)?;
            inode.lock().ops.lock().write(0, content)?;
            Ok(())
        }
        Err(VfsError::NotFound) => {
            // Create new
            // Split path to find parent
            let path_string = String::from(path);
            let parts: Vec<&str> = path_string.rsplitn(2, '/').collect();
            let (filename, parent_path) = if parts.len() == 2 {
                (parts[0], parts[1])
            } else {
                (parts[0], ".")
            };
            
            let parent_path = if parent_path.is_empty() { "/" } else { parent_path };
            
            let parent_dentry = path_lookup(parent_path)?;
            let parent_inode = parent_dentry.lock().inode.clone();
            
            let _inode_id = parent_inode.lock().ops.lock().create(
                filename, 
                FileMode::new(0o644), 
                FileType::Regular
            )?;
            
            // Re-lookup to get the new file (dentry cache population)
            // Or just manually write content now we have ID? 
            // InodeOps create returns InodeId. To write to it, we need to load it.
            // Simplified: loop up again.
            
            // Note: path_lookup checks dentry cache or triggers lookup.
            // Since we created it in backend but not in dentry layer, path_lookup might fail if we don't invalidate cache or if we don't manually add dentry.
            // For now, assuming direct lookup works or simple path resolution.
            
            // To properly write, strictly we need to instantiate proper Dentry/Inode wrappers.
            // Let's rely on path_lookup finding it now that it exists in backend.
            let dentry = path_lookup(path)?;
            let inode = dentry.lock().inode.clone();
            inode.lock().ops.lock().write(0, content)?;
            
            Ok(())
        }
        Err(e) => Err(e)
    }
}

/// Helper: Make directory
pub fn vfs_mkdir(path: &str) -> VfsResult<()> {
    let path_string = String::from(path);
    let parts: Vec<&str> = path_string.rsplitn(2, '/').collect();
    let (dirname, parent_path) = if parts.len() == 2 {
        (parts[0], parts[1])
    } else {
        (parts[0], ".")
    };
    
    let parent_path = if parent_path.is_empty() { "/" } else { parent_path };
    
    let parent_dentry = path_lookup(parent_path)?;
    let parent_inode = parent_dentry.lock().inode.clone();
    
    parent_inode.lock().ops.lock().mkdir(dirname, FileMode::new(0o755))?;
    Ok(())
}

/// Helper: Remove file
pub fn vfs_remove_file(path: &str) -> VfsResult<()> {
    let path_string = String::from(path);
    let parts: Vec<&str> = path_string.rsplitn(2, '/').collect();
    let (filename, parent_path) = if parts.len() == 2 {
        (parts[0], parts[1])
    } else {
        (parts[0], ".")
    };
    
    let parent_path = if parent_path.is_empty() { "/" } else { parent_path };
    
    let parent_dentry = path_lookup(parent_path)?;
    let parent_inode = parent_dentry.lock().inode.clone();
    
    parent_inode.lock().ops.lock().unlink(filename)?;
    
    Ok(())
}
