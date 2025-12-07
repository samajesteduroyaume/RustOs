/// Mock Serial Port pour les tests unitaires
/// 
/// Implémente SerialPort sans hardware réel, stocke les données dans un buffer

use super::serial_trait::SerialPort;
use core::fmt;
use alloc::vec::Vec;
use alloc::string::String;

/// Port série simulé pour les tests
pub struct MockSerial {
    /// Buffer de sortie
    pub output: Vec<u8>,
    /// Buffer d'entrée
    pub input: Vec<u8>,
    /// Position de lecture dans le buffer d'entrée
    read_pos: usize,
}

impl MockSerial {
    /// Crée un nouveau MockSerial
    pub const fn new() -> Self {
        Self {
            output: Vec::new(),
            input: Vec::new(),
            read_pos: 0,
        }
    }
    
    /// Récupère la sortie comme String
    pub fn output_as_string(&self) -> String {
        String::from_utf8_lossy(&self.output).into()
    }
    
    /// Ajoute des données au buffer d'entrée
    pub fn add_input(&mut self, data: &[u8]) {
        self.input.extend_from_slice(data);
    }
    
    /// Efface les buffers
    pub fn clear(&mut self) {
        self.output.clear();
        self.input.clear();
        self.read_pos = 0;
    }
}

impl SerialPort for MockSerial {
    fn init(&mut self) {
        // No-op pour le mock
    }
    
    fn write_byte(&mut self, byte: u8) {
        self.output.push(byte);
    }
    
    fn read_byte(&mut self) -> Option<u8> {
        if self.read_pos < self.input.len() {
            let byte = self.input[self.read_pos];
            self.read_pos += 1;
            Some(byte)
        } else {
            None
        }
    }
    
    fn is_write_ready(&self) -> bool {
        true // Toujours prêt
    }
    
    fn is_read_ready(&self) -> bool {
        self.read_pos < self.input.len()
    }
}

impl fmt::Write for MockSerial {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::fmt::Write;

    #[test]
    fn test_mock_serial_write() {
        let mut serial = MockSerial::new();
        serial.write_byte(b'A');
        serial.write_byte(b'B');
        assert_eq!(serial.output, vec![b'A', b'B']);
    }

    #[test]
    fn test_mock_serial_write_str() {
        let mut serial = MockSerial::new();
        write!(serial, "Hello").unwrap();
        assert_eq!(serial.output_as_string(), "Hello");
    }

    #[test]
    fn test_mock_serial_read() {
        let mut serial = MockSerial::new();
        serial.add_input(b"Test");
        
        assert_eq!(serial.read_byte(), Some(b'T'));
        assert_eq!(serial.read_byte(), Some(b'e'));
        assert_eq!(serial.read_byte(), Some(b's'));
        assert_eq!(serial.read_byte(), Some(b't'));
        assert_eq!(serial.read_byte(), None);
    }

    #[test]
    fn test_mock_serial_clear() {
        let mut serial = MockSerial::new();
        serial.write_byte(b'A');
        serial.add_input(b"B");
        
        serial.clear();
        
        assert_eq!(serial.output.len(), 0);
        assert_eq!(serial.input.len(), 0);
        assert_eq!(serial.read_pos, 0);
    }
}
