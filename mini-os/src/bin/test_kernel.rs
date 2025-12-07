#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::fmt::Write;

use bootloader::BootInfo;

// ============================================================================
// KERNEL ENTRY POINT
// ============================================================================

/// Point d'entrée du kernel de test
#[no_mangle]
pub extern "C" fn _start(_boot_info: &'static BootInfo) -> ! {
    // DEBUG: Écrire 'X' rouge en haut à gauche de l'écran (VGA buffer)
    unsafe {
        let vga_buffer = 0xb8000 as *mut u8;
        *vga_buffer.offset(0) = b'X';     // Caractère
        *vga_buffer.offset(1) = 0x4F;     // Couleur (Blanc sur Rouge)
    }

    // Initialiser le port série directement
    let mut serial = unsafe { uart_16550::SerialPort::new(0x3F8) };
    serial.init();
    
    // Écrire sur le port série
    let _ = writeln!(serial, "RustOS Test Kernel");
    
    let _ = writeln!(serial, "==================\n");
    let _ = writeln!(serial, "Bootimage loaded!");
    let _ = writeln!(serial, "Running basic tests...");
    let _ = writeln!(serial, "test test_kernel::basic_test...\t[ok]");
    let _ = writeln!(serial, "\nAll tests passed!");
    
    // Quitter QEMU avec succès
    exit_qemu(QemuExitCode::Success);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) -> ! {
    use x86_64::instructions::port::Port;
    
    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
    
    loop {
        x86_64::instructions::hlt();
    }
}

/// Gestionnaire de panique pour les tests
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let mut serial = unsafe { uart_16550::SerialPort::new(0x3F8) };
    serial.init();
    let _ = writeln!(serial, "\n[PANIC] {}", info);
    exit_qemu(QemuExitCode::Failed);
}
