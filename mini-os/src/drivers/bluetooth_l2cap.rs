/// Bluetooth L2CAP (Logical Link Control and Adaptation Protocol)
/// 
/// Gère les canaux logiques et la fragmentation/réassemblage des paquets

use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use super::bluetooth_hci::BluetoothError;

/// Identifiants de canaux L2CAP réservés
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum L2capCid {
    Null = 0x0000,
    SignalingChannel = 0x0001,
    ConnectionlessChannel = 0x0002,
    AmpManagerProtocol = 0x0003,
    AttributeProtocol = 0x0004,
    LESignalingChannel = 0x0005,
    SecurityManagerProtocol = 0x0006,
    BrEdrSecurityManager = 0x0007,
}

/// Codes de commande L2CAP
#[derive(Debug, Clone, Copy)]
pub enum L2capCommandCode {
    CommandReject = 0x01,
    ConnectionRequest = 0x02,
    ConnectionResponse = 0x03,
    ConfigurationRequest = 0x04,
    ConfigurationResponse = 0x05,
    DisconnectionRequest = 0x06,
    DisconnectionResponse = 0x07,
    EchoRequest = 0x08,
    EchoResponse = 0x09,
    InformationRequest = 0x0A,
    InformationResponse = 0x0B,
}

/// En-tête L2CAP
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct L2capHeader {
    pub length: u16,        // Longueur des données
    pub channel_id: u16,    // Identifiant du canal
}

impl L2capHeader {
    pub fn new(length: u16, channel_id: u16) -> Self {
        Self {
            length,
            channel_id,
        }
    }

    pub fn as_bytes(&self) -> [u8; 4] {
        [
            (self.length & 0xFF) as u8,
            (self.length >> 8) as u8,
            (self.channel_id & 0xFF) as u8,
            (self.channel_id >> 8) as u8,
        ]
    }

    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 4 {
            return None;
        }

        Some(Self {
            length: u16::from_le_bytes([data[0], data[1]]),
            channel_id: u16::from_le_bytes([data[2], data[3]]),
        })
    }
}

/// En-tête de commande de signalisation L2CAP
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct L2capSignalingHeader {
    pub code: u8,           // Code de commande
    pub identifier: u8,     // Identifiant de transaction
    pub length: u16,        // Longueur des données
}

impl L2capSignalingHeader {
    pub fn new(code: L2capCommandCode, identifier: u8, length: u16) -> Self {
        Self {
            code: code as u8,
            identifier,
            length,
        }
    }
}

/// Requête de connexion L2CAP
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct L2capConnectionRequest {
    pub psm: u16,           // Protocol/Service Multiplexer
    pub source_cid: u16,    // Source Channel ID
}

/// Réponse de connexion L2CAP
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct L2capConnectionResponse {
    pub destination_cid: u16,   // Destination Channel ID
    pub source_cid: u16,        // Source Channel ID
    pub result: u16,            // Résultat
    pub status: u16,            // Statut
}

/// Résultats de connexion
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum L2capConnectionResult {
    Success = 0x0000,
    Pending = 0x0001,
    PsmNotSupported = 0x0002,
    SecurityBlock = 0x0003,
    NoResources = 0x0004,
}

/// Canal L2CAP
#[derive(Debug)]
pub struct L2capChannel {
    /// Identifiant du canal local
    pub local_cid: u16,
    
    /// Identifiant du canal distant
    pub remote_cid: u16,
    
    /// PSM (Protocol/Service Multiplexer)
    pub psm: u16,
    
    /// MTU (Maximum Transmission Unit)
    pub mtu: u16,
    
    /// Canal connecté
    pub connected: bool,
    
    /// Buffer de réception
    pub rx_buffer: Vec<u8>,
}

impl L2capChannel {
    pub fn new(local_cid: u16, psm: u16) -> Self {
        Self {
            local_cid,
            remote_cid: 0,
            psm,
            mtu: 672, // MTU par défaut
            connected: false,
            rx_buffer: Vec::new(),
        }
    }

    /// Envoie des données sur le canal
    pub fn send(&self, data: &[u8]) -> Result<(), BluetoothError> {
        if !self.connected {
            return Err(BluetoothError::ConnectionFailed);
        }

        // TODO: Fragmenter et envoyer via HCI ACL
        Ok(())
    }

    /// Reçoit des données du canal
    pub fn receive(&mut self, data: &[u8]) {
        self.rx_buffer.extend_from_slice(data);
    }

    /// Lit les données reçues
    pub fn read(&mut self) -> Vec<u8> {
        let data = self.rx_buffer.clone();
        self.rx_buffer.clear();
        data
    }
}

/// Gestionnaire L2CAP
pub struct L2capManager {
    /// Canaux actifs (CID -> Channel)
    channels: BTreeMap<u16, L2capChannel>,
    
    /// Prochain CID disponible
    next_cid: u16,
    
    /// Identifiant de transaction
    next_identifier: u8,
}

impl L2capManager {
    /// Crée un nouveau gestionnaire L2CAP
    pub fn new() -> Self {
        Self {
            channels: BTreeMap::new(),
            next_cid: 0x0040, // Les CID < 0x0040 sont réservés
            next_identifier: 1,
        }
    }

    /// Alloue un nouveau CID
    fn allocate_cid(&mut self) -> u16 {
        let cid = self.next_cid;
        self.next_cid += 1;
        cid
    }

    /// Obtient le prochain identifiant de transaction
    fn next_identifier(&mut self) -> u8 {
        let id = self.next_identifier;
        self.next_identifier = self.next_identifier.wrapping_add(1);
        if self.next_identifier == 0 {
            self.next_identifier = 1;
        }
        id
    }

    /// Crée un canal L2CAP
    pub fn create_channel(&mut self, psm: u16) -> Result<u16, BluetoothError> {
        let cid = self.allocate_cid();
        let channel = L2capChannel::new(cid, psm);
        self.channels.insert(cid, channel);
        Ok(cid)
    }

    /// Envoie une requête de connexion
    pub fn connect(&mut self, cid: u16, remote_psm: u16) -> Result<(), BluetoothError> {
        let channel = self.channels.get_mut(&cid)
            .ok_or(BluetoothError::NotFound)?;

        // Créer la requête de connexion
        let request = L2capConnectionRequest {
            psm: remote_psm,
            source_cid: cid,
        };

        // TODO: Envoyer la requête via le canal de signalisation

        Ok(())
    }

    /// Traite un paquet L2CAP reçu
    pub fn handle_packet(&mut self, data: &[u8]) -> Result<(), BluetoothError> {
        if data.len() < 4 {
            return Err(BluetoothError::InvalidParameter);
        }

        let header = L2capHeader::from_bytes(data)
            .ok_or(BluetoothError::InvalidParameter)?;

        let payload = &data[4..];
        
        // Copier channel_id pour éviter une référence non alignée au champ de struct packed
        let channel_id = header.channel_id;

        if channel_id == L2capCid::SignalingChannel as u16 {
            self.handle_signaling(payload)?;
        } else if let Some(channel) = self.channels.get_mut(&channel_id) {
            channel.receive(payload);
        }

        Ok(())
    }

    /// Traite un paquet de signalisation
    fn handle_signaling(&mut self, data: &[u8]) -> Result<(), BluetoothError> {
        if data.len() < 4 {
            return Err(BluetoothError::InvalidParameter);
        }

        let code = data[0];
        let identifier = data[1];
        let length = u16::from_le_bytes([data[2], data[3]]);

        match code {
            0x02 => self.handle_connection_request(&data[4..]),
            0x03 => self.handle_connection_response(&data[4..]),
            0x06 => self.handle_disconnection_request(&data[4..]),
            _ => Ok(()),
        }
    }

    /// Traite une requête de connexion
    fn handle_connection_request(&mut self, data: &[u8]) -> Result<(), BluetoothError> {
        if data.len() < 4 {
            return Err(BluetoothError::InvalidParameter);
        }

        let psm = u16::from_le_bytes([data[0], data[1]]);
        let source_cid = u16::from_le_bytes([data[2], data[3]]);

        // TODO: Créer un canal et envoyer une réponse

        Ok(())
    }

    /// Traite une réponse de connexion
    fn handle_connection_response(&mut self, data: &[u8]) -> Result<(), BluetoothError> {
        if data.len() < 8 {
            return Err(BluetoothError::InvalidParameter);
        }

        let destination_cid = u16::from_le_bytes([data[0], data[1]]);
        let source_cid = u16::from_le_bytes([data[2], data[3]]);
        let result = u16::from_le_bytes([data[4], data[5]]);

        if result == L2capConnectionResult::Success as u16 {
            if let Some(channel) = self.channels.get_mut(&source_cid) {
                channel.remote_cid = destination_cid;
                channel.connected = true;
            }
        }

        Ok(())
    }

    /// Traite une requête de déconnexion
    fn handle_disconnection_request(&mut self, data: &[u8]) -> Result<(), BluetoothError> {
        if data.len() < 4 {
            return Err(BluetoothError::InvalidParameter);
        }

        let destination_cid = u16::from_le_bytes([data[0], data[1]]);
        let source_cid = u16::from_le_bytes([data[2], data[3]]);

        // TODO: Fermer le canal et envoyer une réponse
        self.channels.remove(&destination_cid);

        Ok(())
    }

    /// Obtient un canal
    pub fn get_channel(&self, cid: u16) -> Option<&L2capChannel> {
        self.channels.get(&cid)
    }

    /// Obtient un canal mutable
    pub fn get_channel_mut(&mut self, cid: u16) -> Option<&mut L2capChannel> {
        self.channels.get_mut(&cid)
    }

    /// Ferme un canal
    pub fn close_channel(&mut self, cid: u16) -> Result<(), BluetoothError> {
        self.channels.remove(&cid)
            .ok_or(BluetoothError::NotFound)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_l2cap_header() {
        let header = L2capHeader::new(100, 0x0040);
        assert_eq!(header.length, 100);
        assert_eq!(header.channel_id, 0x0040);

        let bytes = header.as_bytes();
        let parsed = L2capHeader::from_bytes(&bytes).unwrap();
        assert_eq!(parsed.length, 100);
        assert_eq!(parsed.channel_id, 0x0040);
    }

    #[test_case]
    fn test_l2cap_channel() {
        let channel = L2capChannel::new(0x0040, 0x0001);
        assert_eq!(channel.local_cid, 0x0040);
        assert_eq!(channel.psm, 0x0001);
        assert!(!channel.connected);
    }

    #[test_case]
    fn test_l2cap_manager() {
        let mut manager = L2capManager::new();
        let cid = manager.create_channel(0x0001).unwrap();
        assert!(cid >= 0x0040);
        
        let channel = manager.get_channel(cid).unwrap();
        assert_eq!(channel.psm, 0x0001);
    }
}
