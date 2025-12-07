
use core::ptr::{read_volatile, write_volatile};

pub struct LocalApic {
    base_address: u64,
}

impl LocalApic {
    pub const fn new(base_address: u64) -> Self {
        Self { base_address }
    }

    unsafe fn read(&self, reg: u32) -> u32 {
        read_volatile((self.base_address + reg as u64) as *const u32)
    }

    unsafe fn write(&self, reg: u32, value: u32) {
        write_volatile((self.base_address + reg as u64) as *mut u32, value);
    }

    pub fn id(&self) -> u32 {
        unsafe { self.read(0x020) >> 24 }
    }

    pub fn version(&self) -> u32 {
        unsafe { self.read(0x030) }
    }

    pub fn eoi(&self) {
        unsafe { self.write(0x0B0, 0) }
    }

    pub fn enable(&self) {
        unsafe {
            // Spurious Interrupt Vector Register
            // Set bit 8 to enable APIC
            // Set vector to 0xFF (spurious vector)
            self.write(0x0F0, self.read(0x0F0) | 0x100 | 0xFF);
        }
    }
    
    pub fn clear_error(&mut self) {
        unsafe {
            self.write(0x280, 0);
        }
    }
    
    // Envoi d'une interruption IPI (Inter-Processor Interrupt)
    pub fn send_ipi(&self, apic_id: u32, vector: u8) {
        unsafe {
            // ICR High: Destination APIC ID
            self.write(0x310, apic_id << 24);
            // ICR Low: Vector | Delivery Mode (0 = Fixed) | Level (1 = Assert) | Trigger (0 = Edge)
            self.write(0x300, vector as u32);
        }
    }
    
    // Envoi d'une interruption INIT
    pub fn send_init(&self, apic_id: u32) {
         unsafe {
            self.write(0x310, apic_id << 24);
            // ICR Low: Init (5 << 8) | Level (1) | Assert (1)
            self.write(0x300, 0x00004500);
        }
    }
    
    // Envoi d'une interruption SIPI (Start-up IPI)
    pub fn send_sipi(&self, apic_id: u32, trampoline_page: u8) {
        unsafe {
             self.write(0x310, apic_id << 24);
             // ICR Low: SIPI (6 << 8) | Vector (trampoline_page)
             self.write(0x300, 0x00004600 | trampoline_page as u32);
        }
    }
}

/// Signale la fin d'interruption (EOI) au LAPIC courant.
/// Suppose l'adresse de base standard 0xFEE00000.
pub fn signal_eoi() {
    unsafe { core::ptr::write_volatile(0xFEE000B0 as *mut u32, 0); }
}
