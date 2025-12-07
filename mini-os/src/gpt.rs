use crate::drivers::disk::{Disk, DiskDriver, DiskError};
use alloc::vec::Vec;
use core::mem::size_of;
use core::slice;

const GPT_SIGNATURE: u64 = 0x5452415020494645; // "EFI PART" in little endian

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct GptHeader {
    pub signature: u64,
    pub revision: u32,
    pub header_size: u32,
    pub crc32: u32,
    pub reserved: u32,
    pub current_lba: u64,
    pub backup_lba: u64,
    pub first_usable_lba: u64,
    pub last_usable_lba: u64,
    pub disk_guid: [u8; 16],
    pub partition_entry_lba: u64,
    pub num_partition_entries: u32,
    pub size_of_partition_entry: u32,
    pub partition_entry_crc32: u32,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct GptPartitionEntry {
    pub type_guid: [u8; 16],
    pub partition_guid: [u8; 16],
    pub start_lba: u64,
    pub end_lba: u64,
    pub attributes: u64,
    pub partition_name: [u16; 36],
}

#[derive(Debug, Clone)]
pub struct Partition {
    pub start_lba: u64,
    pub end_lba: u64,
    pub size_sectors: u64,
    pub index: usize,
}

pub fn parse_gpt(disk: &mut DiskDriver) -> Result<Vec<Partition>, DiskError> {
    let mut partitions = Vec::new();
    let mut buffer = [0u8; 512];

    // Read Protective MBR (LBA 0) - Optional check, skipping for now
    
    // Read GPT Header (LBA 1)
    disk.read(1, &mut buffer)?;
    
    let header = unsafe { &*(buffer.as_ptr() as *const GptHeader) };
    
    // Verify signature
    if header.signature != GPT_SIGNATURE {
        return Ok(partitions); // Not a valid GPT
    }
    
    let num_entries = header.num_partition_entries;
    let entry_size = header.size_of_partition_entry;
    let entry_lba = header.partition_entry_lba;
    
    // Read Partition Entries
    // Assuming 512 byte sectors for simplicity in this loop logic
    // A real implementation would handle entry_size potentially crossing sector boundaries more robustly
    // Use a simple buffer large enough multiple entries or read sector by sector
    
    let entries_per_sector = 512 / entry_size as u64;
    
    for i in 0..num_entries {
        let current_entry_index = i as u64;
        let sector_offset = current_entry_index / entries_per_sector;
        let entry_offset_in_sector = (current_entry_index % entries_per_sector) * entry_size as u64;
        
        // Optimize: verify if we need to read a new sector
        // For simplicity, just read every time or cache?
        // Let's read the sector relative to entry_lba
        
        disk.read(entry_lba + sector_offset, &mut buffer)?;
        
        let entry_ptr = unsafe { 
            buffer.as_ptr().offset(entry_offset_in_sector as isize) as *const GptPartitionEntry 
        };
        let entry = unsafe { &*entry_ptr };
        
        // Check if entry is used (Partition Type GUID not zero)
        let is_unused = entry.type_guid.iter().all(|&b| b == 0);
        if !is_unused {
            partitions.push(Partition {
                start_lba: entry.start_lba,
                end_lba: entry.end_lba,
                size_sectors: entry.end_lba - entry.start_lba + 1,
                index: i as usize,
            });
        }
    }
    
    Ok(partitions)
}
