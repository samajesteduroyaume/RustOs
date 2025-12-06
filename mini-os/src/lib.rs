#![no_std]
#![feature(alloc_error_handler)]
#![feature(abi_x86_interrupt)]
#![feature(allocator_api)]
#![feature(ptr_metadata)]
#![feature(slice_ptr_get)]
#![feature(slice_range)]
#![feature(step_trait)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

// Importations de base de la bibliothèque standard
use core::panic::PanicInfo;

// Importations pour l'allocation
extern crate alloc;

// Modules du noyau
pub mod memory;
pub mod process;
pub mod scheduler;
pub mod syscall;

// Ré-export des types importants
pub use process::{Process, ProcessManager, ProcessState};
pub use scheduler::{Scheduler, SchedulerPolicy};
pub use syscall::{SyscallHandler, SyscallNumber, SyscallResult, SyscallError};

// Gestionnaire de panique personnalisé
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // Affiche le message de panique si disponible
    let _msg = info.message();
    // Ici, vous devriez avoir une fonction pour écrire sur l'écran ou le port série
    // Par exemple: vga_buffer::_print(format_args!("PANIC: {}\n", msg));
    
    // Boucle infinie pour arrêter l'exécution
    loop {}
}

// Gestionnaire d'erreur d'allocation
#[alloc_error_handler]
fn alloc_error_handler(layout: core::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

// Fonction utilitaire pour les tests
#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
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
    // Exécute les tests
    test_runner(&[]);
    
    // Boucle infinie après les tests
    loop {}
}
