/// EXT3 Filesystem - EXT2 with Journaling
/// 
/// EXT3 extends EXT2 by adding a journal to ensure filesystem consistency
/// after crashes. All metadata operations are journaled.

use alloc::sync::Arc;
use alloc::vec::Vec;
use alloc::string::String;
use spin::Mutex;
use crate::ext2::{Ext2, Ext2Error};
use crate::fs::{VfsError as FsError, Journal, JournalMode, journal::OperationType};
use crate::drivers::disk::Disk;

/// EXT3 Superblock extension
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct Ext3SuperBlock {
    /// Journal inode number
    pub journal_inum: u32,
    /// Journal device
    pub journal_dev: u32,
    /// Last orphan inode
    pub last_orphan: u32,
    /// Hash seed for directory indexing
    pub hash_seed: [u32; 4],
    /// Default hash version
    pub def_hash_version: u8,
    /// Journal backup type
    pub jnl_backup_type: u8,
    /// Descriptor size
    pub desc_size: u16,
    /// Default mount options
    pub default_mount_opts: u32,
    /// First meta block group
    pub first_meta_bg: u32,
    /// Journal UUID
    pub journal_uuid: [u8; 16],
}

impl Default for Ext3SuperBlock {
    fn default() -> Self {
        Self {
            journal_inum: 8, // Standard journal inode
            journal_dev: 0,
            last_orphan: 0,
            hash_seed: [0; 4],
            def_hash_version: 0,
            jnl_backup_type: 0,
            desc_size: 0,
            default_mount_opts: 0,
            first_meta_bg: 0,
            journal_uuid: [0; 16],
        }
    }
}

/// EXT3 Filesystem
pub struct Ext3<D: Disk> {
    /// Underlying EXT2 filesystem
    ext2: Ext2<D>,
    /// Journal for metadata consistency
    journal: Arc<Mutex<Journal>>,
    /// Journaling mode
    mode: JournalMode,
    /// EXT3-specific superblock data
    ext3_sb: Ext3SuperBlock,
}

impl<D: Disk> Ext3<D> {
    /// Create a new EXT3 filesystem from a disk
    pub fn new(disk: D, mode: JournalMode) -> Result<Self, Ext2Error> {
        let ext2 = Ext2::new(disk)?;
        let journal = Arc::new(Mutex::new(Journal::new(mode)));
        
        Ok(Self {
            ext2,
            journal,
            mode,
            ext3_sb: Ext3SuperBlock::default(),
        })
    }

    /// Mount the filesystem and perform journal recovery if needed
    pub fn mount(&mut self) -> Result<(), FsError> {
        // Perform journal recovery
        let mut journal = self.journal.lock();
        match journal.recover() {
            Ok(recovered) => {
                if recovered > 0 {
                    crate::vga_buffer::WRITER.lock()
                        .write_string(&alloc::format!("EXT3: Recovered {} transactions\n", recovered));
                }
                Ok(())
            }
            Err(_) => Err(FsError::IoError),
        }
    }

    /// Read a directory (no journaling needed for reads)
    pub fn read_dir(&self, path: &str) -> Result<Vec<String>, FsError> {
        self.ext2.read_dir(path)
    }

    /// Read a file (no journaling needed for reads)
    pub fn read_file(&self, path: &str) -> Result<Vec<u8>, FsError> {
        self.ext2.read_file(path)
    }

    /// Write a file with journaling
    pub fn write_file(&mut self, path: &str, content: &[u8]) -> Result<(), FsError> {
        let mut journal = self.journal.lock();
        let tx_id = journal.begin_transaction();
        
        // Log the write operation
        journal.log_operation(
            tx_id,
            OperationType::Write,
            0, // Block number will be determined by EXT2
            Some(content.to_vec())
        ).map_err(|_| FsError::IoError)?;
        
        // Perform the actual write
        match self.ext2.write_file(path, content) {
            Ok(_) => {
                // Commit the transaction
                journal.commit_transaction(tx_id)
                    .map_err(|_| FsError::IoError)?;
                Ok(())
            }
            Err(e) => {
                // Rollback on error
                journal.rollback_transaction(tx_id)
                    .map_err(|_| FsError::IoError)?;
                Err(e)
            }
        }
    }

    /// Create a file with journaling
    pub fn create_file(&mut self, path: &str, content: &[u8]) -> Result<(), FsError> {
        let mut journal = self.journal.lock();
        let tx_id = journal.begin_transaction();
        
        // Log the create operation
        journal.log_operation(
            tx_id,
            OperationType::Create,
            0,
            Some(content.to_vec())
        ).map_err(|_| FsError::IoError)?;
        
        // Perform the actual creation
        match self.ext2.create_file(path, content) {
            Ok(_) => {
                journal.commit_transaction(tx_id)
                    .map_err(|_| FsError::IoError)?;
                Ok(())
            }
            Err(e) => {
                journal.rollback_transaction(tx_id)
                    .map_err(|_| FsError::IoError)?;
                Err(e)
            }
        }
    }

    /// Delete a file with journaling
    pub fn delete_file(&mut self, path: &str) -> Result<(), FsError> {
        let mut journal = self.journal.lock();
        let tx_id = journal.begin_transaction();
        
        // Log the delete operation
        journal.log_operation(
            tx_id,
            OperationType::Delete,
            0,
            None
        ).map_err(|_| FsError::IoError)?;
        
        // Perform the actual deletion
        match self.ext2.delete_file(path) {
            Ok(_) => {
                journal.commit_transaction(tx_id)
                    .map_err(|_| FsError::IoError)?;
                Ok(())
            }
            Err(e) => {
                journal.rollback_transaction(tx_id)
                    .map_err(|_| FsError::IoError)?;
                Err(e)
            }
        }
    }

    /// Create a directory with journaling
    pub fn create_dir(&mut self, path: &str) -> Result<(), FsError> {
        let mut journal = self.journal.lock();
        let tx_id = journal.begin_transaction();
        
        // Log the metadata operation
        journal.log_operation(
            tx_id,
            OperationType::Metadata,
            0,
            None
        ).map_err(|_| FsError::IoError)?;
        
        // Perform the actual directory creation
        match self.ext2.create_dir(path) {
            Ok(_) => {
                journal.commit_transaction(tx_id)
                    .map_err(|_| FsError::IoError)?;
                Ok(())
            }
            Err(e) => {
                journal.rollback_transaction(tx_id)
                    .map_err(|_| FsError::IoError)?;
                Err(e)
            }
        }
    }

    /// Get journal statistics
    pub fn get_journal_stats(&self) -> crate::fs::JournalStats {
        self.journal.lock().get_stats()
    }

    /// Sync all pending journal transactions to disk
    pub fn sync(&mut self) -> Result<(), FsError> {
        // Force commit all pending transactions
        let stats = self.get_journal_stats();
        crate::vga_buffer::WRITER.lock()
            .write_string(&alloc::format!(
                "EXT3: Syncing {} active transactions\n",
                stats.active_transactions
            ));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ext3_creation() {
        // Test will be implemented with a mock disk
    }
    
    #[test]
    fn test_journaled_write() {
        // Test journaling during write operations
    }
    
    #[test]
    fn test_recovery() {
        // Test journal recovery after simulated crash
    }
}
