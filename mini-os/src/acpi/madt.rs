
use super::tables::SdtHeader;
use alloc::vec::Vec;

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct Madt {
    pub header: SdtHeader,
    pub local_apic_address: u32,
    pub flags: u32,
}

#[derive(Debug)]
pub struct ProcessorInfo {
    pub processor_id: u8,
    pub apic_id: u8,
    pub flags: u32,
}

pub fn parse_madt(madt_ptr: *const Madt) -> Vec<ProcessorInfo> {
    let mut processors = Vec::new();
    
    let madt = unsafe { *madt_ptr };
    let header_len = core::mem::size_of::<Madt>();
    let total_len = madt.header.length as usize;
    
    let mut offset = header_len;
    let start_ptr = madt_ptr as *const u8;
    
    while offset < total_len {
        let entry_ptr = unsafe { start_ptr.add(offset) };
        let entry_type = unsafe { *entry_ptr };
        let entry_len = unsafe { *entry_ptr.add(1) };
        
        if entry_type == 0 { // Processor Local APIC
            let processor_id = unsafe { *entry_ptr.add(2) };
            let apic_id = unsafe { *entry_ptr.add(3) };
            let flags = unsafe { 
                let ptr = entry_ptr.add(4) as *const u32;
                ptr.read_unaligned()
            };
            
            // Check if processor is enabled (Bit 0 of flags)
            if flags & 1 == 1 {
                processors.push(ProcessorInfo {
                    processor_id,
                    apic_id,
                    flags,
                });
            }
        }
        
        offset += entry_len as usize;
    }
    
    processors
}
