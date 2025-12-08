#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(abi_x86_interrupt)]
#![feature(allocator_api)]
#![feature(ptr_metadata)]
#![feature(slice_ptr_get)]
#![feature(slice_range)]
#![feature(step_trait)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

// Importations de base de la bibliothèque standard
use core::panic::PanicInfo;

// Importations pour l'allocation
extern crate alloc;

// Modules du noyau
pub mod memory;
pub mod interrupts;
pub mod keyboard;
pub mod power;
pub mod process;
pub mod scheduler;
pub mod syscall;
pub mod fs;
#[cfg(feature = "smp")]
pub mod acpi;
#[cfg(feature = "smp")]
pub mod smp;
pub mod fat32;
pub mod ext2;
pub mod ext3;
pub mod ext4;
pub mod fs_manager;  // Gestionnaire EXT4
pub mod gpt;
pub mod ring3;
pub mod ring3_memory;
pub mod ring3_example;
pub mod vga_buffer;  // ← Ajouté pour les drivers
pub mod drivers;
pub mod net;
pub mod ipc;
// pub mod vm; // Disabled - depends on Limine

// Modules pour les tests QEMU
#[macro_use]
pub mod serial;
pub mod test_runner;

// Ré-export des types importants
pub use process::{Process, ProcessManager, ProcessState};
pub use scheduler::Scheduler;
// pub use scheduler::SchedulerPolicy; // TODO: Re-enable when policy module is fixed
pub use syscall::{SyscallHandler, SyscallNumber, SyscallResult, SyscallError};

// Gestionnaire de panique personnalisé
// Gestionnaire de panique personnalisé
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_runner::test_panic_handler(info)
}

// Gestionnaire d'erreur d'allocation
#[cfg(test)]
#[alloc_error_handler]
fn alloc_error_handler(_layout: core::alloc::Layout) -> ! {
    panic!("allocation error")
}

// Fonction utilitaire pour les tests
pub fn test_runner(tests: &[&dyn Fn()]) {
    // Exécute chaque test
    for test in tests {
        test();
    }
    
    // Sortie en cas de succès
    // Ici, vous pourriez éteindre la machine virtuelle ou redémarrer
    loop {}
}

// Point d'entrée pour les tests
#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Note: L'initialisation complète du kernel n'est pas nécessaire pour tous les tests
    // Les tests qui nécessitent du hardware peuvent être marqués avec #[ignore]
    
    serial_println!("RustOS Test Suite");
    serial_println!("=================\n");
    
    test_main();  // Exécute tous les tests
    
    loop {
        x86_64::instructions::hlt();
    }
}
