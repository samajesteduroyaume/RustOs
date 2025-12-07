/// Trait d'abstraction pour les ports série
/// 
/// Permet d'implémenter différentes backends (UART, VGA, Mock) avec la même interface

use core::fmt;

/// Trait pour les ports série
pub trait SerialPort: fmt::Write {
    /// Initialise le port série
    fn init(&mut self);
    
    /// Écrit un octet sur le port
    fn write_byte(&mut self, byte: u8);
    
    /// Lit un octet du port (si disponible)
    fn read_byte(&mut self) -> Option<u8>;
    
    /// Vérifie si le port est prêt à écrire
    fn is_write_ready(&self) -> bool {
        true // Par défaut, toujours prêt
    }
    
    /// Vérifie si des données sont disponibles en lecture
    fn is_read_ready(&self) -> bool {
        false // Par défaut, pas de données
    }
}
