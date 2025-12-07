/// Module de communication série pour les tests QEMU
/// 
/// Ce module fournit une interface pour écrire sur le port série COM1 (0x3F8),
/// permettant d'afficher les résultats des tests dans la console de l'hôte.

use uart_16550::SerialPort;
use spin::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    /// Port série COM1 (0x3F8) pour la sortie de test
    pub static ref SERIAL1: Mutex<SerialPort> = {
        let mut serial_port = unsafe { SerialPort::new(0x3F8) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

/// Fonction interne pour écrire sur le port série
#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;
    
    interrupts::without_interrupts(|| {
        SERIAL1
            .lock()
            .write_fmt(args)
            .expect("Printing to serial failed");
    });
}

/// Macro pour écrire sur le port série (comme print!)
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*))
    };
}

/// Macro pour écrire sur le port série avec retour à la ligne (comme println!)
#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_serial_output() {
        serial_print!("test_serial_output... ");
        serial_println!("[ok]");
    }
}
