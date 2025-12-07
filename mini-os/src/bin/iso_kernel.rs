#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::fmt::Write;

// ============================================================================
// MULTIBOOT2 HEADER
// ============================================================================

#[repr(C, align(8))]
struct Multiboot2Header {
    magic: u32,
    architecture: u32,
    header_length: u32,
    checksum: u32,
    // End tag
    end_tag_type: u16,
    end_tag_flags: u16,
    end_tag_size: u32,
}

const MULTIBOOT2_MAGIC: u32 = 0xE85250D6;
const MULTIBOOT2_ARCHITECTURE_I386: u32 = 0; // i386 (protected mode)
const HEADER_LENGTH: u32 = core::mem::size_of::<Multiboot2Header>() as u32;

#[used]
#[link_section = ".multiboot2"]
static MULTIBOOT2_HEADER: Multiboot2Header = Multiboot2Header {
    magic: MULTIBOOT2_MAGIC,
    architecture: MULTIBOOT2_ARCHITECTURE_I386,
    header_length: HEADER_LENGTH,
    checksum: 0u32.wrapping_sub(MULTIBOOT2_MAGIC)
        .wrapping_sub(MULTIBOOT2_ARCHITECTURE_I386)
        .wrapping_sub(HEADER_LENGTH),
    end_tag_type: 0,
    end_tag_flags: 0,
    end_tag_size: 8,
};

// ============================================================================
// KERNEL ENTRY POINT
// ============================================================================

/// Point d'entrée du kernel pour l'ISO
/// Le bootloader nous laisse en mode 32-bit (multiboot), 
/// mais nous compilons en 64-bit target.
/// C'est un problème potentiel si pas de trampoline.
/// Pour cet exemple, on suppose que GRUB arrive à nous charger.
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // DEBUG: Écrire 'I' (ISO) vert en haut à gauche
    unsafe {
        let vga_buffer = 0xb8000 as *mut u8;
        *vga_buffer.offset(0) = b'I';     // Caractère
        *vga_buffer.offset(1) = 0x2F;     // Couleur (Blanc sur Vert)
    }

    // Initialiser le port série
    let mut serial = unsafe { uart_16550::SerialPort::new(0x3F8) };
    serial.init();
    
    let _ = writeln!(serial, "RustOS ISO Kernel");
    let _ = writeln!(serial, "=================");
    let _ = writeln!(serial, "Successfully booted from ISO!");

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
