/// USB Protocol - Implémentation du protocole USB
/// 
/// Gère les descripteurs, requêtes, et transferts USB

use alloc::vec::Vec;
use alloc::string::String;

/// Erreurs USB
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbError {
    NotFound,
    Timeout,
    IoError,
    InvalidDescriptor,
    InvalidArgument,
    DeviceNotResponding,
    TransferFailed,
    BufferTooSmall,
    NotSupported,
}

/// Types de transfert USB
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferType {
    Control,        // Transfert de contrôle
    Bulk,           // Transfert en masse
    Interrupt,      // Transfert d'interruption
    Isochronous,    // Transfert isochrone
}

/// Direction de transfert
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferDirection {
    HostToDevice,   // OUT
    DeviceToHost,   // IN
}

/// Type de descripteur USB
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DescriptorType {
    Device = 0x01,
    Configuration = 0x02,
    String = 0x03,
    Interface = 0x04,
    Endpoint = 0x05,
    DeviceQualifier = 0x06,
    OtherSpeedConfiguration = 0x07,
    InterfacePower = 0x08,
}

/// Descripteur de périphérique USB
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct DeviceDescriptor {
    pub length: u8,                 // Taille du descripteur (18 octets)
    pub descriptor_type: u8,        // Type de descripteur (0x01)
    pub usb_version: u16,           // Version USB (BCD)
    pub device_class: u8,           // Classe du périphérique
    pub device_subclass: u8,        // Sous-classe
    pub device_protocol: u8,        // Protocole
    pub max_packet_size: u8,        // Taille max du paquet EP0
    pub vendor_id: u16,             // ID vendeur
    pub product_id: u16,            // ID produit
    pub device_version: u16,        // Version du périphérique (BCD)
    pub manufacturer_index: u8,     // Index string fabricant
    pub product_index: u8,          // Index string produit
    pub serial_index: u8,           // Index string numéro de série
    pub num_configurations: u8,     // Nombre de configurations
}

impl DeviceDescriptor {
    pub fn new() -> Self {
        Self {
            length: 18,
            descriptor_type: DescriptorType::Device as u8,
            usb_version: 0x0200,  // USB 2.0
            device_class: 0,
            device_subclass: 0,
            device_protocol: 0,
            max_packet_size: 64,
            vendor_id: 0,
            product_id: 0,
            device_version: 0x0100,
            manufacturer_index: 0,
            product_index: 0,
            serial_index: 0,
            num_configurations: 1,
        }
    }

    pub fn usb_version_string(&self) -> String {
        let major = (self.usb_version >> 8) & 0xFF;
        let minor = (self.usb_version >> 4) & 0x0F;
        format!("{}.{}", major, minor)
    }
}

/// Descripteur de configuration USB
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct ConfigurationDescriptor {
    pub length: u8,                 // Taille du descripteur (9 octets)
    pub descriptor_type: u8,        // Type de descripteur (0x02)
    pub total_length: u16,          // Taille totale de la configuration
    pub num_interfaces: u8,         // Nombre d'interfaces
    pub configuration_value: u8,    // Valeur de configuration
    pub configuration_index: u8,    // Index string configuration
    pub attributes: u8,             // Attributs
    pub max_power: u8,              // Puissance max (en unités de 2mA)
}

impl ConfigurationDescriptor {
    pub fn new() -> Self {
        Self {
            length: 9,
            descriptor_type: DescriptorType::Configuration as u8,
            total_length: 9,
            num_interfaces: 0,
            configuration_value: 1,
            configuration_index: 0,
            attributes: 0x80,  // Bus powered
            max_power: 50,     // 100mA
        }
    }

    pub fn is_self_powered(&self) -> bool {
        (self.attributes & 0x40) != 0
    }

    pub fn supports_remote_wakeup(&self) -> bool {
        (self.attributes & 0x20) != 0
    }

    pub fn max_power_ma(&self) -> u16 {
        self.max_power as u16 * 2
    }
}

/// Descripteur d'interface USB
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct InterfaceDescriptor {
    pub length: u8,                 // Taille du descripteur (9 octets)
    pub descriptor_type: u8,        // Type de descripteur (0x04)
    pub interface_number: u8,       // Numéro d'interface
    pub alternate_setting: u8,      // Paramètre alternatif
    pub num_endpoints: u8,          // Nombre d'endpoints
    pub interface_class: u8,        // Classe d'interface
    pub interface_subclass: u8,     // Sous-classe
    pub interface_protocol: u8,     // Protocole
    pub interface_index: u8,        // Index string interface
}

/// Descripteur d'endpoint USB
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct EndpointDescriptor {
    pub length: u8,                 // Taille du descripteur (7 octets)
    pub descriptor_type: u8,        // Type de descripteur (0x05)
    pub endpoint_address: u8,       // Adresse de l'endpoint
    pub attributes: u8,             // Attributs
    pub max_packet_size: u16,       // Taille max du paquet
    pub interval: u8,               // Intervalle de polling
}

impl EndpointDescriptor {
    pub fn endpoint_number(&self) -> u8 {
        self.endpoint_address & 0x0F
    }

    pub fn is_in(&self) -> bool {
        (self.endpoint_address & 0x80) != 0
    }

    pub fn transfer_type(&self) -> TransferType {
        match self.attributes & 0x03 {
            0 => TransferType::Control,
            1 => TransferType::Isochronous,
            2 => TransferType::Bulk,
            3 => TransferType::Interrupt,
            _ => TransferType::Control,
        }
    }
}

/// Requête de contrôle USB (Setup Packet)
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct SetupPacket {
    pub request_type: u8,           // Type de requête
    pub request: u8,                // Requête
    pub value: u16,                 // Valeur
    pub index: u16,                 // Index
    pub length: u16,                // Longueur des données
}

/// Requêtes USB standard
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsbRequest {
    GetStatus = 0x00,
    ClearFeature = 0x01,
    SetFeature = 0x03,
    SetAddress = 0x05,
    GetDescriptor = 0x06,
    SetDescriptor = 0x07,
    GetConfiguration = 0x08,
    SetConfiguration = 0x09,
    GetInterface = 0x0A,
    SetInterface = 0x0B,
    SynchFrame = 0x0C,
}

impl SetupPacket {
    /// Crée une requête GET_DESCRIPTOR
    pub fn get_descriptor(descriptor_type: DescriptorType, index: u8, length: u16) -> Self {
        Self {
            request_type: 0x80,  // Device to Host, Standard, Device
            request: UsbRequest::GetDescriptor as u8,
            value: ((descriptor_type as u16) << 8) | (index as u16),
            index: 0,
            length,
        }
    }

    /// Crée une requête SET_ADDRESS
    pub fn set_address(address: u8) -> Self {
        Self {
            request_type: 0x00,  // Host to Device, Standard, Device
            request: UsbRequest::SetAddress as u8,
            value: address as u16,
            index: 0,
            length: 0,
        }
    }

    /// Crée une requête SET_CONFIGURATION
    pub fn set_configuration(config: u8) -> Self {
        Self {
            request_type: 0x00,  // Host to Device, Standard, Device
            request: UsbRequest::SetConfiguration as u8,
            value: config as u16,
            index: 0,
            length: 0,
        }
    }

    /// Crée une requête GET_STATUS
    pub fn get_status() -> Self {
        Self {
            request_type: 0x80,  // Device to Host, Standard, Device
            request: UsbRequest::GetStatus as u8,
            value: 0,
            index: 0,
            length: 2,
        }
    }
}

/// Transfert USB
pub struct UsbTransfer {
    pub endpoint: u8,
    pub transfer_type: TransferType,
    pub direction: TransferDirection,
    pub data: Vec<u8>,
    pub max_packet_size: u16,
}

impl UsbTransfer {
    /// Crée un nouveau transfert
    pub fn new(
        endpoint: u8,
        transfer_type: TransferType,
        direction: TransferDirection,
        max_packet_size: u16,
    ) -> Self {
        Self {
            endpoint,
            transfer_type,
            direction,
            data: Vec::new(),
            max_packet_size,
        }
    }

    /// Ajoute des données au transfert
    pub fn add_data(&mut self, data: &[u8]) {
        self.data.extend_from_slice(data);
    }

    /// Obtient les données du transfert
    pub fn get_data(&self) -> &[u8] {
        &self.data
    }

    /// Efface les données du transfert
    pub fn clear_data(&mut self) {
        self.data.clear();
    }

    /// Nombre de paquets nécessaires
    pub fn packet_count(&self) -> usize {
        if self.data.is_empty() {
            return 1; // Paquet de longueur zéro
        }
        (self.data.len() + self.max_packet_size as usize - 1) / self.max_packet_size as usize
    }
}

/// Énumération USB
pub struct UsbEnumerator;

impl UsbEnumerator {
    /// Énumère un périphérique USB
    pub fn enumerate_device(port: u8) -> Result<DeviceDescriptor, UsbError> {
        // TODO: Implémenter l'énumération complète
        // 1. Réinitialiser le port
        // 2. Attendre que le périphérique soit prêt
        // 3. Obtenir les 8 premiers octets du descripteur de périphérique
        // 4. Réinitialiser à nouveau
        // 5. Assigner une adresse
        // 6. Obtenir le descripteur de périphérique complet
        // 7. Obtenir le descripteur de configuration
        // 8. Configurer le périphérique

        Ok(DeviceDescriptor::new())
    }

    /// Lit un descripteur
    pub fn read_descriptor(
        descriptor_type: DescriptorType,
        index: u8,
        length: u16,
    ) -> Result<Vec<u8>, UsbError> {
        // TODO: Implémenter la lecture de descripteur
        // 1. Créer un Setup Packet
        // 2. Effectuer un transfert de contrôle
        // 3. Retourner les données

        Ok(Vec::new())
    }

    /// Lit une chaîne de caractères
    pub fn read_string(index: u8) -> Result<String, UsbError> {
        if index == 0 {
            return Ok(String::new());
        }

        // TODO: Implémenter la lecture de string descriptor
        // 1. Lire le descripteur de chaîne
        // 2. Décoder l'UTF-16LE
        // 3. Retourner la chaîne

        Ok(String::from("USB Device"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_device_descriptor() {
        let desc = DeviceDescriptor::new();
        assert_eq!(desc.length, 18);
        assert_eq!(desc.descriptor_type, 0x01);
    }

    #[test_case]
    fn test_setup_packet_get_descriptor() {
        let packet = SetupPacket::get_descriptor(DescriptorType::Device, 0, 18);
        assert_eq!(packet.request, UsbRequest::GetDescriptor as u8);
        assert_eq!(packet.length, 18);
    }

    #[test_case]
    fn test_setup_packet_set_address() {
        let packet = SetupPacket::set_address(5);
        assert_eq!(packet.request, UsbRequest::SetAddress as u8);
        assert_eq!(packet.value, 5);
    }

    #[test_case]
    fn test_endpoint_descriptor() {
        let mut desc = EndpointDescriptor {
            length: 7,
            descriptor_type: 0x05,
            endpoint_address: 0x81,  // EP1 IN
            attributes: 0x02,        // Bulk
            max_packet_size: 512,
            interval: 0,
        };
        
        assert_eq!(desc.endpoint_number(), 1);
        assert!(desc.is_in());
        assert_eq!(desc.transfer_type(), TransferType::Bulk);
    }

    #[test_case]
    fn test_usb_transfer() {
        let mut transfer = UsbTransfer::new(
            1,
            TransferType::Bulk,
            TransferDirection::DeviceToHost,
            512,
        );
        
        transfer.add_data(&[1, 2, 3, 4, 5]);
        assert_eq!(transfer.get_data().len(), 5);
        assert_eq!(transfer.packet_count(), 1);
    }
}
