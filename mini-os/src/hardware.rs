use crate::vga_buffer::WRITER;
use raw_cpuid::CpuId;
use x86_64::instructions::port::Port;

/// Détecte le CPU et affiche le vendor
pub fn detect_cpu() {
    let cpuid = CpuId::new();
    if let Some(vf) = cpuid.get_vendor_info() {
        WRITER.lock().write_string("CPU Vendor: ");
        WRITER.lock().write_string(vf.as_str());
        WRITER.lock().write_string("\n");
    }
    if let Some(finfo) = cpuid.get_feature_info() {
        WRITER.lock().write_string(&format!("CPU Features: SSE{} SSE2{}\n",
            if finfo.has_sse() { "+" } else { "-" },
            if finfo.has_sse2() { "+" } else { "-" }));
    }
}

/// Scanne le bus PCI et affiche les périphériques détectés
pub fn scan_pci() {
    WRITER.lock().write_string("Scanning PCI devices...\n");
    for bus in 0..=255 {
        for device in 0..32 {
            for function in 0..8 {
                let vendor_id = pci_config_read_u16(bus, device, function, 0x00);
                if vendor_id == 0xFFFF { continue; }
                let device_id = pci_config_read_u16(bus, device, function, 0x02);
                WRITER.lock().write_string(&format!(
                    "PCI Device found: bus {} device {} function {} vendor {:#x} device {:#x}\n",
                    bus, device, function, vendor_id, device_id
                ));
            }
        }
    }
}

/// Lit un registre PCI Configuration Space
fn pci_config_read_u16(bus: u8, device: u8, function: u8, offset: u8) -> u16 {
    let address = 0x80000000
        | ((bus as u32) << 16)
        | ((device as u32 & 0x1F) << 11)
        | ((function as u32 & 0x07) << 8)
        | ((offset as u32) & 0xFC);
    
    unsafe {
        let mut port_address = Port::<u32>::new(0xCF8);
        let mut port_data = Port::<u32>::new(0xCFC);
        port_address.write(address);
        let val = port_data.read();
        if offset & 2 == 0 {
            (val & 0xFFFF) as u16
        } else {
            ((val >> 16) & 0xFFFF) as u16
        }
    }
}

// Multiboot2 n'est pas disponible, fonction désactivée
/// Détecte la RAM disponible (nécessite multiboot2)
pub fn detect_memory(_multiboot_address: usize) {
    WRITER.lock().write_string("Memory detection: multiboot2 not available\n");
    // TODO: Implémenter une détection de mémoire alternative
}
