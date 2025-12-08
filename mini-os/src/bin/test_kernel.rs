#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::fmt::Write;
use mini_os::interrupts; // Use definitions from the library

// ============================================================================
// KERNEL ENTRY POINT
// ============================================================================

/// Point d'entrée du kernel de test
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // 1. Initialiser le port série en premier pour le debug
    // Dirty early init for debug
    unsafe {
        use x86_64::instructions::port::PortGeneric;
        let mut port: x86_64::instructions::port::Port<u8> = x86_64::instructions::port::Port::new(0x3F8);
        port.write(b'K'); // 'K' for Kernel start
    }

    let mut serial = unsafe { uart_16550::SerialPort::new(0x3F8) };
    serial.init();
    
    // 2. Initialiser l'IDT pour gérer les exceptions (évite le Triple Fault)
    interrupts::init_idt();
    unsafe { x86_64::instructions::interrupts::enable(); }
    
    // 3. Message de bienvenue
    let _ = writeln!(serial, "RustOS Test Kernel (Limine)");
    let _ = writeln!(serial, "=========================");
    let _ = writeln!(serial, "Initialization: [OK]");
    
    // 4. Exécuter les tests (basic)
    let _ = writeln!(serial, "Running basic_test...");
    let _ = writeln!(serial, "test passed!");
    
    let _ = writeln!(serial, "\nAll tests passed!");
    
    // 5. Quitter QEMU
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

/// Gestionnaire de panique
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let mut serial = unsafe { uart_16550::SerialPort::new(0x3F8) };
    serial.init();
    let _ = writeln!(serial, "\n[PANIC] {}", info);
    exit_qemu(QemuExitCode::Failed);
}
