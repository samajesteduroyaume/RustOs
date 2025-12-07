
pub mod tables;
pub mod madt;
pub mod fadt;

use core::ptr::read_volatile;
use self::tables::{RsdpDescriptor, SdtHeader};
use self::madt::Madt;
use self::fadt::Fadt;

/// Signature RSD PTR
const RSDP_SIGNATURE: &[u8; 8] = b"RSD PTR ";

/// Trouve la structure RSDP en mémoire
pub fn find_rsdp() -> Option<tables::RsdpDescriptor> {
    // Scan typical memory areas for RSDP:
    // 1. EBDA (Extended BIOS Data Area)
    // 2. Main BIOS area (0xE0000 - 0xFFFFF)
    
    // For simplicity in this mini-os, we might scan the BIOS area.
    // NOTE: In a real UEFI env, we'd get this from bootloader info.
    // If using Multiboot2, we can parse generic tags.
    
    // Let's implement a naive scan in 0xE0000-0xFFFFF for now, assuming legacy boot or simple env.
    // Virtual mapping assumed: Identity mapped lower memory (or we need to translate).
    // Assuming 0xE0000 is accessible at 0xE0000 for now.
    
    let start_addr = 0xE0000;
    let end_addr = 0xFFFFF;
    
    let mut addr = start_addr;
    while addr < end_addr {
        if unsafe { check_signature(addr as *const u8) } {
            let rsdp = unsafe { read_volatile(addr as *const tables::RsdpDescriptor) };
             if rsdp.validate() {
                 return Some(rsdp);
             }
        }
        addr += 16;
    }
    
    None
}

/// Trouve la table MADT via le RSDP
pub fn find_madt(rsdp: &RsdpDescriptor) -> Option<Madt> {
    let rsdt_addr = rsdp.rsdt_address as *const SdtHeader;
    let rsdt = unsafe { read_volatile(rsdt_addr) };
    
    // Check RSDT signature "RSDT"
    if &rsdt.signature != b"RSDT" {
        return None;
    }
    
    let entry_count = (rsdt.length as usize - core::mem::size_of::<SdtHeader>()) / 4;
    let entries_ptr = unsafe { (rsdt_addr as *const u8).add(core::mem::size_of::<SdtHeader>()) as *const u32 };
    
    for i in 0..entry_count {
        let entry_addr = unsafe { *entries_ptr.add(i) };
        let header_ptr = entry_addr as *const SdtHeader;
        let header = unsafe { read_volatile(header_ptr) };
        
        if &header.signature == b"APIC" {
            // Found MADT
            let madt_ptr = entry_addr as *const Madt;
            return Some(unsafe { read_volatile(madt_ptr) });
        }
    }
    
    None
}

/// Trouve la table FADT via le RSDP
pub fn find_fadt(rsdp: &RsdpDescriptor) -> Option<Fadt> {
    let rsdt_addr = rsdp.rsdt_address as *const SdtHeader;
    let rsdt = unsafe { read_volatile(rsdt_addr) };
    
    if &rsdt.signature != b"RSDT" {
        return None;
    }
    
    let entry_count = (rsdt.length as usize - core::mem::size_of::<SdtHeader>()) / 4;
    let entries_ptr = unsafe { (rsdt_addr as *const u8).add(core::mem::size_of::<SdtHeader>()) as *const u32 };
    
    for i in 0..entry_count {
        let entry_addr = unsafe { *entries_ptr.add(i) };
        let header_ptr = entry_addr as *const SdtHeader;
        let header = unsafe { read_volatile(header_ptr) };
        
        if &header.signature == b"FACP" {
            // Found FADT (Signature is "FACP")
            let fadt_ptr = entry_addr as *const Fadt;
            return Some(unsafe { read_volatile(fadt_ptr) });
        }
    }
    
    None
}

unsafe fn check_signature(ptr: *const u8) -> bool {
    for i in 0..8 {
        if *ptr.add(i) != RSDP_SIGNATURE[i] {
            return false;
        }
    }
    true
}
