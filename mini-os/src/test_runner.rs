/// Test runner pour l'environnement kernel QEMU
/// 
/// Ce module fournit l'infrastructure pour exécuter les tests unitaires
/// dans un environnement bare-metal QEMU avec sortie série et exit automatique.

use core::panic::PanicInfo;

/// Trait pour les tests exécutables
pub trait Testable {
    fn run(&self);
}

impl<T: Fn()> Testable for T {
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

/// Runner de tests principal
/// 
/// Exécute tous les tests fournis et affiche les résultats sur le port série.
/// En cas de succès, quitte QEMU avec le code de succès.
pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    serial_println!("================");
    
    for test in tests {
        test.run();
    }
    
    serial_println!("================");
    serial_println!("All tests passed!");
    exit_qemu(QemuExitCode::Success);
}

/// Codes de sortie pour QEMU
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    /// Tous les tests ont réussi
    Success = 0x10,
    /// Au moins un test a échoué
    Failed = 0x11,
}

/// Quitte QEMU avec un code de sortie spécifique
/// 
/// Utilise le port ISA debug exit (0xf4) pour signaler à QEMU de se terminer.
/// Le code de sortie réel de QEMU sera (exit_code << 1) | 1.
pub fn exit_qemu(exit_code: QemuExitCode) -> ! {
    use x86_64::instructions::port::Port;
    
    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
    
    // Si QEMU n'est pas configuré correctement, on boucle
    loop {
        x86_64::instructions::hlt();
    }
}

/// Gestionnaire de panique pour les tests
/// 
/// Affiche l'erreur sur le port série et quitte QEMU avec un code d'échec.
pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_runner_basics() {
        // Ce test vérifie que le test runner fonctionne
        assert_eq!(2 + 2, 4);
    }
}
