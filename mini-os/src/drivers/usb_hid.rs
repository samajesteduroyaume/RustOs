/// USB HID (Human Interface Device) Driver
/// 
/// Gère les périphériques HID comme les claviers, souris, etc.

extern crate alloc;
use alloc::vec::Vec;
use alloc::format;
use super::usb_protocol::*;
use super::usb_controller::UsbError;
use crate::vga_buffer::WRITER;

/// Classe HID
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HidClass {
    None = 0,
    Keyboard = 1,
    Mouse = 2,
}

/// Protocole HID
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HidProtocol {
    None = 0,
    Keyboard = 1,
    Mouse = 2,
}

/// Requêtes HID
#[derive(Debug, Clone, Copy)]
pub enum HidRequest {
    GetReport = 0x01,
    GetIdle = 0x02,
    GetProtocol = 0x03,
    SetReport = 0x09,
    SetIdle = 0x0A,
    SetProtocol = 0x0B,
}

/// Type de rapport HID
#[derive(Debug, Clone, Copy)]
pub enum ReportType {
    Input = 1,
    Output = 2,
    Feature = 3,
}

/// Modificateurs de clavier
#[derive(Debug, Clone, Copy)]
pub struct KeyboardModifiers {
    pub left_ctrl: bool,
    pub left_shift: bool,
    pub left_alt: bool,
    pub left_gui: bool,
    pub right_ctrl: bool,
    pub right_shift: bool,
    pub right_alt: bool,
    pub right_gui: bool,
}

impl KeyboardModifiers {
    pub fn from_byte(byte: u8) -> Self {
        Self {
            left_ctrl: (byte & 0x01) != 0,
            left_shift: (byte & 0x02) != 0,
            left_alt: (byte & 0x04) != 0,
            left_gui: (byte & 0x08) != 0,
            right_ctrl: (byte & 0x10) != 0,
            right_shift: (byte & 0x20) != 0,
            right_alt: (byte & 0x40) != 0,
            right_gui: (byte & 0x80) != 0,
        }
    }

    pub fn to_byte(&self) -> u8 {
        let mut byte = 0u8;
        if self.left_ctrl { byte |= 0x01; }
        if self.left_shift { byte |= 0x02; }
        if self.left_alt { byte |= 0x04; }
        if self.left_gui { byte |= 0x08; }
        if self.right_ctrl { byte |= 0x10; }
        if self.right_shift { byte |= 0x20; }
        if self.right_alt { byte |= 0x40; }
        if self.right_gui { byte |= 0x80; }
        byte
    }
}

/// Rapport de clavier (Boot Protocol)
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct KeyboardReport {
    pub modifiers: u8,      // Modificateurs
    pub reserved: u8,       // Réservé
    pub keycodes: [u8; 6],  // Codes de touches pressées
}

impl KeyboardReport {
    pub fn new() -> Self {
        Self {
            modifiers: 0,
            reserved: 0,
            keycodes: [0; 6],
        }
    }

    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 8 {
            return None;
        }

        Some(Self {
            modifiers: data[0],
            reserved: data[1],
            keycodes: [data[2], data[3], data[4], data[5], data[6], data[7]],
        })
    }

    pub fn get_modifiers(&self) -> KeyboardModifiers {
        KeyboardModifiers::from_byte(self.modifiers)
    }

    pub fn has_key(&self, keycode: u8) -> bool {
        self.keycodes.contains(&keycode)
    }
}

/// Boutons de souris
#[derive(Debug, Clone, Copy)]
pub struct MouseButtons {
    pub left: bool,
    pub right: bool,
    pub middle: bool,
}

impl MouseButtons {
    pub fn from_byte(byte: u8) -> Self {
        Self {
            left: (byte & 0x01) != 0,
            right: (byte & 0x02) != 0,
            middle: (byte & 0x04) != 0,
        }
    }

    pub fn to_byte(&self) -> u8 {
        let mut byte = 0u8;
        if self.left { byte |= 0x01; }
        if self.right { byte |= 0x02; }
        if self.middle { byte |= 0x04; }
        byte
    }
}

/// Rapport de souris (Boot Protocol)
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct MouseReport {
    pub buttons: u8,    // Boutons
    pub x: i8,          // Déplacement X
    pub y: i8,          // Déplacement Y
    pub wheel: i8,      // Molette (optionnel)
}

impl MouseReport {
    pub fn new() -> Self {
        Self {
            buttons: 0,
            x: 0,
            y: 0,
            wheel: 0,
        }
    }

    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 3 {
            return None;
        }

        Some(Self {
            buttons: data[0],
            x: data[1] as i8,
            y: data[2] as i8,
            wheel: if data.len() > 3 { data[3] as i8 } else { 0 },
        })
    }

    pub fn get_buttons(&self) -> MouseButtons {
        MouseButtons::from_byte(self.buttons)
    }
}

/// Driver HID
pub struct UsbHidDriver {
    /// Type de périphérique HID
    pub device_type: HidClass,
    
    /// Protocole HID
    pub protocol: HidProtocol,
    
    /// Endpoint d'interruption IN
    pub endpoint_in: u8,
    
    /// Intervalle de polling (ms)
    pub poll_interval: u8,
    
    /// Taille maximale de paquet
    pub max_packet_size: u16,
}

impl UsbHidDriver {
    /// Crée un nouveau driver HID
    pub fn new(device_type: HidClass, endpoint_in: u8, poll_interval: u8, max_packet_size: u16) -> Self {
        let protocol = match device_type {
            HidClass::Keyboard => HidProtocol::Keyboard,
            HidClass::Mouse => HidProtocol::Mouse,
            HidClass::None => HidProtocol::None,
        };

        Self {
            device_type,
            protocol,
            endpoint_in,
            poll_interval,
            max_packet_size,
        }
    }

    /// Définit le protocole (Boot ou Report)
    pub fn set_protocol(&mut self, protocol: HidProtocol) -> Result<(), UsbError> {
        let setup = SetupPacket {
            request_type: 0x21,  // Host to Device, Class, Interface
            request: HidRequest::SetProtocol as u8,
            value: protocol as u16,
            index: 0,
            length: 0,
        };

        // TODO: Envoyer le Setup Packet
        WRITER.lock().write_string(&format!(
            "Définition protocole HID: {:?}\n",
            protocol
        ));

        self.protocol = protocol;
        Ok(())
    }

    /// Définit l'idle rate
    pub fn set_idle(&mut self, duration: u8, report_id: u8) -> Result<(), UsbError> {
        let setup = SetupPacket {
            request_type: 0x21,  // Host to Device, Class, Interface
            request: HidRequest::SetIdle as u8,
            value: ((duration as u16) << 8) | (report_id as u16),
            index: 0,
            length: 0,
        };

        // TODO: Envoyer le Setup Packet
        WRITER.lock().write_string("Définition idle rate HID\n");

        Ok(())
    }

    /// Lit un rapport
    pub fn get_report(&self, report_type: ReportType, report_id: u8) -> Result<Vec<u8>, UsbError> {
        let setup = SetupPacket {
            request_type: 0xA1,  // Device to Host, Class, Interface
            request: HidRequest::GetReport as u8,
            value: ((report_type as u16) << 8) | (report_id as u16),
            index: 0,
            length: self.max_packet_size,
        };

        // TODO: Envoyer le Setup Packet et recevoir les données
        Ok(Vec::new())
    }

    /// Lit un rapport de clavier
    pub fn read_keyboard(&self) -> Result<KeyboardReport, UsbError> {
        // TODO: Lire via transfert d'interruption IN
        let data = [0u8; 8];
        
        KeyboardReport::from_bytes(&data).ok_or(UsbError::TransferFailed)
    }

    /// Lit un rapport de souris
    pub fn read_mouse(&self) -> Result<MouseReport, UsbError> {
        // TODO: Lire via transfert d'interruption IN
        let data = [0u8; 4];
        
        MouseReport::from_bytes(&data).ok_or(UsbError::TransferFailed)
    }

    /// Initialise le driver HID
    pub fn init(&mut self) -> Result<(), UsbError> {
        WRITER.lock().write_string(&format!(
            "Initialisation HID {:?}...\n",
            self.device_type
        ));

        // Définir le protocole Boot
        self.set_protocol(self.protocol)?;

        // Définir l'idle rate (0 = infini)
        self.set_idle(0, 0)?;

        WRITER.lock().write_string("HID initialisé\n");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_keyboard_modifiers() {
        let mods = KeyboardModifiers::from_byte(0x03); // Ctrl + Shift
        assert!(mods.left_ctrl);
        assert!(mods.left_shift);
        assert!(!mods.left_alt);
        
        assert_eq!(mods.to_byte(), 0x03);
    }

    #[test_case]
    fn test_keyboard_report() {
        let data = [0x02, 0x00, 0x04, 0x05, 0x00, 0x00, 0x00, 0x00];
        let report = KeyboardReport::from_bytes(&data).unwrap();
        
        assert_eq!(report.modifiers, 0x02); // Left Shift
        assert!(report.has_key(0x04));
        assert!(report.has_key(0x05));
    }

    #[test_case]
    fn test_mouse_buttons() {
        let buttons = MouseButtons::from_byte(0x05); // Left + Middle
        assert!(buttons.left);
        assert!(!buttons.right);
        assert!(buttons.middle);
        
        assert_eq!(buttons.to_byte(), 0x05);
    }

    #[test_case]
    fn test_mouse_report() {
        let data = [0x01, 10, 255, 0]; // Left button, X=10, Y=-1
        let report = MouseReport::from_bytes(&data).unwrap();
        
        assert_eq!(report.buttons, 0x01);
        assert_eq!(report.x, 10);
        assert_eq!(report.y, -1);
    }

    #[test_case]
    fn test_hid_driver_creation() {
        let driver = UsbHidDriver::new(HidClass::Keyboard, 0x81, 10, 8);
        assert_eq!(driver.device_type, HidClass::Keyboard);
        assert_eq!(driver.protocol, HidProtocol::Keyboard);
        assert_eq!(driver.endpoint_in, 0x81);
    }
}
