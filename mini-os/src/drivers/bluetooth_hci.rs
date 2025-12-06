/// Bluetooth HCI (Host Controller Interface) Layer
/// 
/// Gère la communication avec le contrôleur Bluetooth

use alloc::vec::Vec;
use alloc::string::String;
use crate::vga_buffer::WRITER;

/// Types de paquets HCI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HciPacketType {
    Command = 0x01,
    AclData = 0x02,
    ScoData = 0x03,
    Event = 0x04,
}

/// Groupes de commandes HCI
#[derive(Debug, Clone, Copy)]
pub enum HciOpCodeGroup {
    LinkControl = 0x01,
    LinkPolicy = 0x02,
    ControllerBaseband = 0x03,
    InformationalParameters = 0x04,
    StatusParameters = 0x05,
    Testing = 0x06,
    LEController = 0x08,
}

/// Commandes HCI courantes
#[derive(Debug, Clone, Copy)]
pub struct HciOpCode(pub u16);

impl HciOpCode {
    // Link Control Commands
    pub const INQUIRY: Self = Self(0x0401);
    pub const INQUIRY_CANCEL: Self = Self(0x0402);
    pub const CREATE_CONNECTION: Self = Self(0x0405);
    pub const DISCONNECT: Self = Self(0x0406);
    pub const ACCEPT_CONNECTION: Self = Self(0x0409);
    pub const REJECT_CONNECTION: Self = Self(0x040A);
    pub const LINK_KEY_REPLY: Self = Self(0x040B);
    pub const LINK_KEY_NEGATIVE_REPLY: Self = Self(0x040C);
    pub const PIN_CODE_REPLY: Self = Self(0x040D);
    pub const PIN_CODE_NEGATIVE_REPLY: Self = Self(0x040E);
    pub const REMOTE_NAME_REQUEST: Self = Self(0x0419);
    
    // Controller & Baseband Commands
    pub const RESET: Self = Self(0x0C03);
    pub const SET_EVENT_MASK: Self = Self(0x0C01);
    pub const WRITE_LOCAL_NAME: Self = Self(0x0C13);
    pub const READ_LOCAL_NAME: Self = Self(0x0C14);
    pub const WRITE_SCAN_ENABLE: Self = Self(0x0C1A);
    pub const WRITE_CLASS_OF_DEVICE: Self = Self(0x0C24);
    
    // Informational Parameters
    pub const READ_BD_ADDR: Self = Self(0x1009);
    pub const READ_LOCAL_VERSION: Self = Self(0x1001);
    pub const READ_LOCAL_FEATURES: Self = Self(0x1003);
    pub const READ_BUFFER_SIZE: Self = Self(0x1005);

    pub fn ogf(&self) -> u8 {
        ((self.0 >> 10) & 0x3F) as u8
    }

    pub fn ocf(&self) -> u16 {
        self.0 & 0x3FF
    }
}

/// Événements HCI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HciEventCode {
    InquiryComplete = 0x01,
    InquiryResult = 0x02,
    ConnectionComplete = 0x03,
    ConnectionRequest = 0x04,
    DisconnectionComplete = 0x05,
    RemoteNameRequestComplete = 0x07,
    EncryptionChange = 0x08,
    CommandComplete = 0x0E,
    CommandStatus = 0x0F,
    HardwareError = 0x10,
    RoleChange = 0x12,
    NumberOfCompletedPackets = 0x13,
    PINCodeRequest = 0x16,
    LinkKeyRequest = 0x17,
    LinkKeyNotification = 0x18,
    MaxSlotsChange = 0x1B,
    ReadRemoteFeaturesComplete = 0x0B,
    ReadRemoteVersionComplete = 0x0C,
}

/// Adresse Bluetooth (BD_ADDR)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BdAddr(pub [u8; 6]);

impl BdAddr {
    pub fn new(addr: [u8; 6]) -> Self {
        Self(addr)
    }

    pub fn from_string(s: &str) -> Option<Self> {
        // Format: XX:XX:XX:XX:XX:XX
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 6 {
            return None;
        }

        let mut addr = [0u8; 6];
        for (i, part) in parts.iter().enumerate() {
            addr[i] = u8::from_str_radix(part, 16).ok()?;
        }

        Some(Self(addr))
    }

    pub fn to_string(&self) -> String {
        format!(
            "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        )
    }
}

/// Paquet de commande HCI
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct HciCommandPacket {
    pub opcode: u16,
    pub parameter_length: u8,
}

impl HciCommandPacket {
    pub fn new(opcode: HciOpCode, parameters: &[u8]) -> Vec<u8> {
        let mut packet = Vec::new();
        packet.push(HciPacketType::Command as u8);
        packet.extend_from_slice(&opcode.0.to_le_bytes());
        packet.push(parameters.len() as u8);
        packet.extend_from_slice(parameters);
        packet
    }
}

/// Paquet d'événement HCI
#[derive(Debug, Clone)]
pub struct HciEventPacket {
    pub event_code: u8,
    pub parameter_length: u8,
    pub parameters: Vec<u8>,
}

impl HciEventPacket {
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 2 {
            return None;
        }

        let event_code = data[0];
        let parameter_length = data[1];

        if data.len() < 2 + parameter_length as usize {
            return None;
        }

        let parameters = data[2..2 + parameter_length as usize].to_vec();

        Some(Self {
            event_code,
            parameter_length,
            parameters,
        })
    }

    pub fn is_command_complete(&self) -> bool {
        self.event_code == HciEventCode::CommandComplete as u8
    }

    pub fn is_command_status(&self) -> bool {
        self.event_code == HciEventCode::CommandStatus as u8
    }
}

/// Paquet de données ACL
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct HciAclHeader {
    pub handle_and_flags: u16,  // Handle (12 bits) + PB (2 bits) + BC (2 bits)
    pub data_length: u16,
}

impl HciAclHeader {
    pub fn new(handle: u16, pb_flag: u8, bc_flag: u8, data_length: u16) -> Self {
        let handle_and_flags = (handle & 0x0FFF) | ((pb_flag as u16 & 0x03) << 12) | ((bc_flag as u16 & 0x03) << 14);
        Self {
            handle_and_flags,
            data_length,
        }
    }

    pub fn handle(&self) -> u16 {
        self.handle_and_flags & 0x0FFF
    }

    pub fn pb_flag(&self) -> u8 {
        ((self.handle_and_flags >> 12) & 0x03) as u8
    }

    pub fn bc_flag(&self) -> u8 {
        ((self.handle_and_flags >> 14) & 0x03) as u8
    }
}

/// Erreurs Bluetooth
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BluetoothError {
    NotFound,
    InitializationFailed,
    CommandFailed,
    Timeout,
    InvalidParameter,
    NotSupported,
    ConnectionFailed,
}

/// Contrôleur Bluetooth HCI
pub struct HciController {
    /// Adresse Bluetooth locale
    pub bd_addr: Option<BdAddr>,
    
    /// Nom local
    pub local_name: String,
    
    /// Version HCI
    pub hci_version: u8,
    
    /// Contrôleur initialisé
    pub initialized: bool,
}

impl HciController {
    /// Crée un nouveau contrôleur HCI
    pub fn new() -> Self {
        Self {
            bd_addr: None,
            local_name: String::from("RustOS BT"),
            hci_version: 0,
            initialized: false,
        }
    }

    /// Envoie une commande HCI
    pub fn send_command(&self, opcode: HciOpCode, parameters: &[u8]) -> Result<(), BluetoothError> {
        let packet = HciCommandPacket::new(opcode, parameters);
        
        // TODO: Envoyer le paquet au contrôleur
        WRITER.lock().write_string(&format!(
            "Envoi commande HCI: OpCode=0x{:04X}\n",
            opcode.0
        ));

        Ok(())
    }

    /// Attend un événement HCI
    pub fn wait_event(&self, timeout_ms: u32) -> Result<HciEventPacket, BluetoothError> {
        // TODO: Attendre et recevoir un événement
        Err(BluetoothError::Timeout)
    }

    /// Réinitialise le contrôleur
    pub fn reset(&mut self) -> Result<(), BluetoothError> {
        WRITER.lock().write_string("Réinitialisation contrôleur Bluetooth...\n");
        
        self.send_command(HciOpCode::RESET, &[])?;
        
        // Attendre l'événement Command Complete
        let event = self.wait_event(1000)?;
        
        if event.is_command_complete() {
            self.initialized = true;
            Ok(())
        } else {
            Err(BluetoothError::CommandFailed)
        }
    }

    /// Lit l'adresse BD_ADDR
    pub fn read_bd_addr(&mut self) -> Result<BdAddr, BluetoothError> {
        self.send_command(HciOpCode::READ_BD_ADDR, &[])?;
        
        let event = self.wait_event(1000)?;
        
        if event.is_command_complete() && event.parameters.len() >= 7 {
            let mut addr = [0u8; 6];
            addr.copy_from_slice(&event.parameters[1..7]);
            let bd_addr = BdAddr::new(addr);
            self.bd_addr = Some(bd_addr);
            
            WRITER.lock().write_string(&format!(
                "Adresse Bluetooth: {}\n",
                bd_addr.to_string()
            ));
            
            Ok(bd_addr)
        } else {
            Err(BluetoothError::CommandFailed)
        }
    }

    /// Définit le nom local
    pub fn write_local_name(&mut self, name: &str) -> Result<(), BluetoothError> {
        let mut params = [0u8; 248];
        let name_bytes = name.as_bytes();
        let len = name_bytes.len().min(248);
        params[..len].copy_from_slice(&name_bytes[..len]);
        
        self.send_command(HciOpCode::WRITE_LOCAL_NAME, &params)?;
        self.local_name = name.into();
        
        Ok(())
    }

    /// Active le scan (découverte de périphériques)
    pub fn enable_scan(&self, inquiry: bool, page: bool) -> Result<(), BluetoothError> {
        let scan_enable = (inquiry as u8) | ((page as u8) << 1);
        self.send_command(HciOpCode::WRITE_SCAN_ENABLE, &[scan_enable])?;
        
        WRITER.lock().write_string(&format!(
            "Scan activé: inquiry={}, page={}\n",
            inquiry, page
        ));
        
        Ok(())
    }

    /// Démarre une recherche de périphériques
    pub fn start_inquiry(&self, duration: u8, max_responses: u8) -> Result<(), BluetoothError> {
        let params = [
            0x33, 0x8B, 0x9E, // LAP (General Inquiry)
            duration,         // Inquiry Length (1.28s * duration)
            max_responses,    // Num Responses
        ];
        
        self.send_command(HciOpCode::INQUIRY, &params)?;
        
        WRITER.lock().write_string("Recherche de périphériques Bluetooth...\n");
        
        Ok(())
    }

    /// Initialise le contrôleur
    pub fn init(&mut self) -> Result<(), BluetoothError> {
        WRITER.lock().write_string("Initialisation Bluetooth HCI...\n");

        // Réinitialiser
        self.reset()?;

        // Lire l'adresse
        self.read_bd_addr()?;

        // Définir le nom
        self.write_local_name("RustOS Bluetooth")?;

        WRITER.lock().write_string("Bluetooth HCI initialisé\n");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_bd_addr() {
        let addr = BdAddr::new([0x00, 0x1A, 0x7D, 0xDA, 0x71, 0x13]);
        assert_eq!(addr.into(), "00:1A:7D:DA:71:13");
    }

    #[test_case]
    fn test_bd_addr_from_string() {
        let addr = BdAddr::from_string("AA:BB:CC:DD:EE:FF").unwrap();
        assert_eq!(addr.0, [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
    }

    #[test_case]
    fn test_opcode() {
        let opcode = HciOpCode::RESET;
        assert_eq!(opcode.ogf(), 0x03);
        assert_eq!(opcode.ocf(), 0x03);
    }

    #[test_case]
    fn test_acl_header() {
        let header = HciAclHeader::new(0x123, 0x02, 0x00, 100);
        assert_eq!(header.handle(), 0x123);
        assert_eq!(header.pb_flag(), 0x02);
        assert_eq!(header.bc_flag(), 0x00);
        assert_eq!(header.data_length, 100);
    }

    #[test_case]
    fn test_hci_controller() {
        let controller = HciController::new();
        assert!(!controller.initialized);
        assert_eq!(controller.local_name, "RustOS BT");
    }
}
