/// USB Mass Storage Driver - Bulk-Only Transport (BOT)
/// 
/// Implémente le protocole USB Mass Storage Class avec SCSI

use alloc::vec::Vec;
use alloc::string::String;
use super::usb_protocol::*;
use crate::vga_buffer::WRITER;

/// Command Block Wrapper (CBW)
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct CommandBlockWrapper {
    pub signature: u32,         // 0x43425355 "USBC"
    pub tag: u32,               // Tag unique
    pub data_transfer_length: u32, // Longueur des données
    pub flags: u8,              // Direction (bit 7: 0=OUT, 1=IN)
    pub lun: u8,                // Logical Unit Number
    pub cb_length: u8,          // Longueur de la commande (1-16)
    pub command_block: [u8; 16], // Commande SCSI
}

impl CommandBlockWrapper {
    const SIGNATURE: u32 = 0x43425355;

    pub fn new(tag: u32, data_length: u32, direction: TransferDirection, lun: u8, command: &[u8]) -> Self {
        let mut cb = [0u8; 16];
        let len = command.len().min(16);
        cb[..len].copy_from_slice(&command[..len]);

        let flags = match direction {
            TransferDirection::DeviceToHost => 0x80,
            TransferDirection::HostToDevice => 0x00,
        };

        Self {
            signature: Self::SIGNATURE,
            tag,
            data_transfer_length: data_length,
            flags,
            lun,
            cb_length: len as u8,
            command_block: cb,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                self as *const Self as *const u8,
                core::mem::size_of::<Self>(),
            )
        }
    }
}

/// Command Status Wrapper (CSW)
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct CommandStatusWrapper {
    pub signature: u32,         // 0x53425355 "USBS"
    pub tag: u32,               // Tag du CBW correspondant
    pub data_residue: u32,      // Différence entre données attendues et transférées
    pub status: u8,             // Statut (0=OK, 1=Fail, 2=Phase Error)
}

impl CommandStatusWrapper {
    const SIGNATURE: u32 = 0x53425355;

    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < core::mem::size_of::<Self>() {
            return None;
        }

        unsafe {
            let csw = core::ptr::read_unaligned(data.as_ptr() as *const Self);
            if csw.signature == Self::SIGNATURE {
                Some(csw)
            } else {
                None
            }
        }
    }

    pub fn is_success(&self) -> bool {
        self.status == 0
    }
}

/// Commandes SCSI
#[derive(Debug, Clone, Copy)]
pub enum ScsiCommand {
    TestUnitReady = 0x00,
    RequestSense = 0x03,
    Inquiry = 0x12,
    ModeSense6 = 0x1A,
    ReadCapacity10 = 0x25,
    Read10 = 0x28,
    Write10 = 0x2A,
    ModeSense10 = 0x5A,
    ReadCapacity16 = 0x9E,
}

/// Driver USB Mass Storage
pub struct UsbMassStorageDriver {
    /// Endpoint IN (lecture)
    pub endpoint_in: u8,
    
    /// Endpoint OUT (écriture)
    pub endpoint_out: u8,
    
    /// Taille maximale de paquet
    pub max_packet_size: u16,
    
    /// Tag de commande (incrémenté à chaque commande)
    tag: u32,
    
    /// Capacité du disque (en blocs)
    pub capacity: u64,
    
    /// Taille de bloc
    pub block_size: u32,
}

impl UsbMassStorageDriver {
    /// Crée un nouveau driver mass storage
    pub fn new(endpoint_in: u8, endpoint_out: u8, max_packet_size: u16) -> Self {
        Self {
            endpoint_in,
            endpoint_out,
            max_packet_size,
            tag: 1,
            capacity: 0,
            block_size: 512,
        }
    }

    /// Obtient le prochain tag
    fn next_tag(&mut self) -> u32 {
        let tag = self.tag;
        self.tag = self.tag.wrapping_add(1);
        tag
    }

    /// Envoie une commande SCSI
    fn send_command(&mut self, command: &[u8], data_length: u32, direction: TransferDirection) -> Result<u32, UsbError> {
        let tag = self.next_tag();
        let cbw = CommandBlockWrapper::new(tag, data_length, direction, 0, command);

        // TODO: Envoyer le CBW via transfert bulk OUT
        WRITER.lock().write_string("Envoi commande SCSI...\n");

        Ok(tag)
    }

    /// Reçoit le statut de la commande
    fn receive_status(&self, expected_tag: u32) -> Result<CommandStatusWrapper, UsbError> {
        // TODO: Recevoir le CSW via transfert bulk IN
        let csw_data = [0u8; 13]; // Taille du CSW
        
        if let Some(csw) = CommandStatusWrapper::from_bytes(&csw_data) {
            if csw.tag == expected_tag {
                Ok(csw)
            } else {
                Err(UsbError::TransferFailed)
            }
        } else {
            Err(UsbError::TransferFailed)
        }
    }

    /// Test Unit Ready
    pub fn test_unit_ready(&mut self) -> Result<(), UsbError> {
        let command = [ScsiCommand::TestUnitReady as u8, 0, 0, 0, 0, 0];
        let tag = self.send_command(&command, 0, TransferDirection::DeviceToHost)?;
        let csw = self.receive_status(tag)?;

        if csw.is_success() {
            Ok(())
        } else {
            Err(UsbError::TransferFailed)
        }
    }

    /// Inquiry - Obtient les informations sur le périphérique
    pub fn inquiry(&mut self) -> Result<Vec<u8>, UsbError> {
        let command = [
            ScsiCommand::Inquiry as u8,
            0,      // EVPD=0
            0,      // Page Code
            0,      // Reserved
            36,     // Allocation Length
            0,      // Control
        ];

        let tag = self.send_command(&command, 36, TransferDirection::DeviceToHost)?;
        
        // TODO: Recevoir les données d'inquiry
        let mut data = vec![0u8; 36];
        
        let csw = self.receive_status(tag)?;
        
        if csw.is_success() {
            Ok(data)
        } else {
            Err(UsbError::TransferFailed)
        }
    }

    /// Read Capacity - Obtient la capacité du disque
    pub fn read_capacity(&mut self) -> Result<(u64, u32), UsbError> {
        let command = [
            ScsiCommand::ReadCapacity10 as u8,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];

        let tag = self.send_command(&command, 8, TransferDirection::DeviceToHost)?;
        
        // TODO: Recevoir les données de capacité
        let data = [0u8; 8];
        
        // Extraire last_block et block_size (big-endian)
        let last_block = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
        let block_size = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
        
        let csw = self.receive_status(tag)?;
        
        if csw.is_success() {
            self.capacity = (last_block as u64) + 1;
            self.block_size = block_size;
            Ok((self.capacity, self.block_size))
        } else {
            Err(UsbError::TransferFailed)
        }
    }

    /// Read - Lit des blocs
    pub fn read(&mut self, lba: u32, num_blocks: u16, buffer: &mut [u8]) -> Result<usize, UsbError> {
        let transfer_length = num_blocks as u32 * self.block_size;
        
        if buffer.len() < transfer_length as usize {
            return Err(UsbError::InvalidArgument);
        }

        let command = [
            ScsiCommand::Read10 as u8,
            0,                              // Flags
            (lba >> 24) as u8,              // LBA (big-endian)
            (lba >> 16) as u8,
            (lba >> 8) as u8,
            lba as u8,
            0,                              // Reserved
            (num_blocks >> 8) as u8,        // Transfer Length
            num_blocks as u8,
            0,                              // Control
        ];

        let tag = self.send_command(&command, transfer_length, TransferDirection::DeviceToHost)?;
        
        // TODO: Recevoir les données via transfert bulk IN
        WRITER.lock().write_string(&format!(
            "Lecture {} blocs à LBA {}\n",
            num_blocks, lba
        ));
        
        let csw = self.receive_status(tag)?;
        
        if csw.is_success() {
            Ok(transfer_length as usize)
        } else {
            Err(UsbError::TransferFailed)
        }
    }

    /// Write - Écrit des blocs
    pub fn write(&mut self, lba: u32, num_blocks: u16, buffer: &[u8]) -> Result<usize, UsbError> {
        let transfer_length = num_blocks as u32 * self.block_size;
        
        if buffer.len() < transfer_length as usize {
            return Err(UsbError::InvalidArgument);
        }

        let command = [
            ScsiCommand::Write10 as u8,
            0,                              // Flags
            (lba >> 24) as u8,              // LBA (big-endian)
            (lba >> 16) as u8,
            (lba >> 8) as u8,
            lba as u8,
            0,                              // Reserved
            (num_blocks >> 8) as u8,        // Transfer Length
            num_blocks as u8,
            0,                              // Control
        ];

        let tag = self.send_command(&command, transfer_length, TransferDirection::HostToDevice)?;
        
        // TODO: Envoyer les données via transfert bulk OUT
        WRITER.lock().write_string(&format!(
            "Écriture {} blocs à LBA {}\n",
            num_blocks, lba
        ));
        
        let csw = self.receive_status(tag)?;
        
        if csw.is_success() {
            Ok(transfer_length as usize)
        } else {
            Err(UsbError::TransferFailed)
        }
    }

    /// Initialise le driver
    pub fn init(&mut self) -> Result<(), UsbError> {
        WRITER.lock().write_string("Initialisation USB Mass Storage...\n");

        // Test si le périphérique est prêt
        self.test_unit_ready()?;

        // Obtenir les informations du périphérique
        let inquiry_data = self.inquiry()?;
        
        // Obtenir la capacité
        let (capacity, block_size) = self.read_capacity()?;
        
        WRITER.lock().write_string(&format!(
            "Capacité: {} blocs de {} octets ({} MB)\n",
            capacity,
            block_size,
            (capacity * block_size as u64) / (1024 * 1024)
        ));

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_cbw_creation() {
        let command = [0x12, 0, 0, 0, 36, 0]; // INQUIRY
        let cbw = CommandBlockWrapper::new(1, 36, TransferDirection::DeviceToHost, 0, &command);
        
        assert_eq!(cbw.signature, 0x43425355);
        assert_eq!(cbw.tag, 1);
        assert_eq!(cbw.data_transfer_length, 36);
        assert_eq!(cbw.flags, 0x80);
        assert_eq!(cbw.cb_length, 6);
    }

    #[test_case]
    fn test_mass_storage_driver() {
        let mut driver = UsbMassStorageDriver::new(0x81, 0x02, 512);
        assert_eq!(driver.endpoint_in, 0x81);
        assert_eq!(driver.endpoint_out, 0x02);
        assert_eq!(driver.block_size, 512);
    }

    #[test_case]
    fn test_tag_increment() {
        let mut driver = UsbMassStorageDriver::new(0x81, 0x02, 512);
        assert_eq!(driver.next_tag(), 1);
        assert_eq!(driver.next_tag(), 2);
        assert_eq!(driver.next_tag(), 3);
    }
}
