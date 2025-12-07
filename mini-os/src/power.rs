use x86_64::instructions::port::Port;
use crate::acpi::{self, fadt::Fadt};

pub struct PowerManager {
    fadt: Option<Fadt>,
}

impl PowerManager {
    pub fn new() -> Self {
        let mut pm = Self { fadt: None };
        pm.init();
        pm
    }

    fn init(&mut self) {
        if let Some(rsdp) = acpi::find_rsdp() {
            if let Some(fadt) = acpi::find_fadt(&rsdp) {
                self.fadt = Some(fadt);
                self.enable_acpi(&fadt);
            }
        }
    }

    fn enable_acpi(&self, fadt: &Fadt) {
        // Init ACPI Mode if SMI_CMD is present and ACPI_ENABLE is set
        if fadt.smi_cmd != 0 && fadt.acpi_enable != 0 {
             let mut smi_port: Port<u8> = Port::new(fadt.smi_cmd as u16);
             unsafe { smi_port.write(fadt.acpi_enable) };
             // TODO: Wait for SCI_EN bit in PM1a_EVT_BLK to be set
        }
    }

    pub fn shutdown(&self) {
        crate::serial_println!("Shutting down...");
        
        // 1. Try ACPI Shutdown (QEMU S5)
        // Note: Proper way is to parse _S5 package in DSDT.
        // For QEMU, SLP_TYP is typically 5 (001b << 10 for SLP_TYPa).
        // Plus SLP_EN (1 << 13).
        
        // Fallback or QEMU-specific hardcoded values for S5
        if let Some(fadt) = &self.fadt {
             let pm1a_cnt_blk = fadt.pm1a_cnt_blk as u16;
             let mut port: Port<u16> = Port::new(pm1a_cnt_blk);
             
             // Try QEMU S5 sequence
             unsafe {
                 port.write(0x2000 | (5 << 10)); // SLP_EN | SLP_TYP=5
             }
        }
        
        // 2. QEMU specific shutdown port (older QEMU)
        let mut qemu_port: Port<u16> = Port::new(0x604);
        unsafe { qemu_port.write(0x2000) };

        // 3. Loop if failed
        crate::serial_println!("Shutdown failed. Halting.");
        loop { x86_64::instructions::hlt(); }
    }

    pub fn reboot(&self) {
        crate::serial_println!("Rebooting...");
        
        // 1. Try keyboard controller pulse
        let mut keyboard_cmd_port: Port<u8> = Port::new(0x64);
        unsafe {
            // Pulse bit 0 (Reset CPU)
            keyboard_cmd_port.write(0xFE);
        }
        
        // 2. FADT Reset Register (if available, ACPI v2.0+)
        // if let Some(fadt) = &self.fadt { ... }

        // 3. Triple Fault
        unsafe {
            // Load invalid IDT
             core::arch::asm!("lidt [{}]", in(reg) 0);
             core::arch::asm!("int3");
        }
        
        loop { x86_64::instructions::hlt(); }
    }
}

// Global instance
use spin::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref POWER_MANAGER: Mutex<PowerManager> = Mutex::new(PowerManager::new());
}

pub fn shutdown() -> ! {
    POWER_MANAGER.lock().shutdown();
    loop { x86_64::instructions::hlt(); }
}

pub fn reboot() -> ! {
    POWER_MANAGER.lock().reboot();
    loop { x86_64::instructions::hlt(); }
}
