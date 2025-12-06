use super::{Driver, DriverError};
use crate::vga_buffer::WRITER;

/// Erreurs spécifiques au driver disque
#[derive(Debug, Clone, Copy)]
pub enum DiskError {
    InvalidSector,
    BufferTooSmall,
    InvalidSize,
    ReadFailed,
    WriteFailed,
    Timeout,
}

/// Ports ATA/SATA
pub mod ata_ports {
    pub const PRIMARY_DATA: u16 = 0x1F0;
    pub const PRIMARY_ERROR: u16 = 0x1F1;
    pub const PRIMARY_SECTOR_COUNT: u16 = 0x1F2;
    pub const PRIMARY_LBA_LOW: u16 = 0x1F3;
    pub const PRIMARY_LBA_MID: u16 = 0x1F4;
    pub const PRIMARY_LBA_HIGH: u16 = 0x1F5;
    pub const PRIMARY_DEVICE: u16 = 0x1F6;
    pub const PRIMARY_STATUS: u16 = 0x1F7;
    pub const PRIMARY_COMMAND: u16 = 0x1F7;
}

/// Commandes ATA
pub mod ata_commands {
    pub const READ_SECTORS: u8 = 0x20;
    pub const WRITE_SECTORS: u8 = 0x30;
    pub const IDENTIFY: u8 = 0xEC;
}

/// Bits de statut ATA
pub mod ata_status {
    pub const BUSY: u8 = 0x80;
    pub const READY: u8 = 0x40;
    pub const WRITE_FAULT: u8 = 0x20;
    pub const SEEK_COMPLETE: u8 = 0x10;
    pub const DATA_REQUEST: u8 = 0x08;
    pub const CORRECTED_DATA: u8 = 0x04;
    pub const INDEX: u8 = 0x02;
    pub const ERROR: u8 = 0x01;
}

/// Driver disque ATA/SATA
pub struct DiskDriver {
    pub name: String,
    pub sectors: u64,
    pub sector_size: u16,
    pub initialized: bool,
    pub primary_master: bool,
}

impl DiskDriver {
    /// Crée un nouveau driver disque
    pub fn new(name: &str, primary_master: bool) -> Self {
        Self {
            name: name.into(),
            sectors: 0,
            sector_size: 512,
            initialized: false,
            primary_master,
        }
    }

    /// Lit un secteur depuis le disque
    pub fn read_sector(&self, sector: u64, buffer: &mut [u8]) -> Result<(), DiskError> {
        if buffer.len() < self.sector_size as usize {
            return Err(DiskError::BufferTooSmall);
        }

        if sector >= self.sectors {
            return Err(DiskError::InvalidSector);
        }

        // TODO: Implémenter la lecture ATA
        // 1. Vérifier que le disque est prêt
        // 2. Envoyer la commande READ_SECTORS
        // 3. Attendre que les données soient disponibles
        // 4. Lire les données depuis le port DATA

        WRITER.lock().write_string(&format!(
            "Lecture secteur {} (512 octets)\n",
            sector
        ));

        Ok(())
    }

    /// Écrit un secteur sur le disque
    pub fn write_sector(&mut self, sector: u64, data: &[u8]) -> Result<(), DiskError> {
        if data.len() != self.sector_size as usize {
            return Err(DiskError::InvalidSize);
        }

        if sector >= self.sectors {
            return Err(DiskError::InvalidSector);
        }

        // TODO: Implémenter l'écriture ATA
        // 1. Vérifier que le disque est prêt
        // 2. Envoyer la commande WRITE_SECTORS
        // 3. Écrire les données vers le port DATA
        // 4. Attendre la fin de l'écriture

        WRITER.lock().write_string(&format!(
            "Écriture secteur {} (512 octets)\n",
            sector
        ));

        Ok(())
    }

    /// Lit plusieurs secteurs
    pub fn read_sectors(&self, start: u64, count: u64, buffer: &mut [u8]) -> Result<(), DiskError> {
        let required_size = (count as usize) * (self.sector_size as usize);
        
        if buffer.len() < required_size {
            return Err(DiskError::BufferTooSmall);
        }

        if start + count > self.sectors {
            return Err(DiskError::InvalidSector);
        }

        // TODO: Implémenter la lecture de plusieurs secteurs
        WRITER.lock().write_string(&format!(
            "Lecture {} secteurs à partir du secteur {}\n",
            count, start
        ));

        Ok(())
    }

    /// Écrit plusieurs secteurs
    pub fn write_sectors(&mut self, start: u64, data: &[u8]) -> Result<(), DiskError> {
        if data.len() % (self.sector_size as usize) != 0 {
            return Err(DiskError::InvalidSize);
        }

        let count = data.len() / (self.sector_size as usize);
        
        if start + (count as u64) > self.sectors {
            return Err(DiskError::InvalidSector);
        }

        // TODO: Implémenter l'écriture de plusieurs secteurs
        WRITER.lock().write_string(&format!(
            "Écriture {} secteurs à partir du secteur {}\n",
            count, start
        ));

        Ok(())
    }

    /// Identifie le disque
    pub fn identify(&mut self) -> Result<(), DiskError> {
        // TODO: Implémenter la commande IDENTIFY
        // Cette commande retourne les informations du disque
        // (modèle, numéro de série, nombre de secteurs, etc.)

        WRITER.lock().write_string("Identification du disque...\n");

        // Valeurs par défaut pour la simulation
        self.sectors = 1000000; // 500 MB avec secteurs de 512 octets
        self.sector_size = 512;

        Ok(())
    }

    /// Obtient la taille totale du disque en octets
    pub fn get_size(&self) -> u64 {
        self.sectors * (self.sector_size as u64)
    }

    /// Obtient le nombre de secteurs
    pub fn get_sector_count(&self) -> u64 {
        self.sectors
    }

    /// Obtient la taille d'un secteur
    pub fn get_sector_size(&self) -> u16 {
        self.sector_size
    }
}

impl Driver for DiskDriver {
    fn name(&self) -> &str {
        &self.name
    }

    fn init(&mut self) -> Result<(), DriverError> {
        WRITER.lock().write_string(&format!("Initialisation du driver disque: {}\n", self.name));

        // Identifier le disque
        self.identify().map_err(|_| DriverError::InitializationFailed)?;

        self.initialized = true;

        WRITER.lock().write_string(&format!(
            "Disque {} initialisé: {} secteurs ({} MB)\n",
            self.name,
            self.sectors,
            self.get_size() / (1024 * 1024)
        ));

        Ok(())
    }

    fn handle_interrupt(&mut self, irq: u8) {
        WRITER.lock().write_string(&format!(
            "Interruption disque (IRQ {})\n",
            irq
        ));

        // TODO: Gérer les interruptions disque
        // - Vérifier le statut du disque
        // - Traiter les erreurs
        // - Réveiller les processus en attente
    }

    fn shutdown(&mut self) -> Result<(), DriverError> {
        WRITER.lock().write_string(&format!("Arrêt du driver disque: {}\n", self.name));
        self.initialized = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_disk_driver_creation() {
        let driver = DiskDriver::new("sda", true);
        assert_eq!(driver.name, "sda");
        assert_eq!(driver.sector_size, 512);
        assert!(!driver.initialized);
    }

    #[test_case]
    fn test_disk_driver_identify() {
        let mut driver = DiskDriver::new("sda", true);
        assert!(driver.identify().is_ok());
        assert!(driver.sectors > 0);
    }

    #[test_case]
    fn test_disk_driver_size() {
        let mut driver = DiskDriver::new("sda", true);
        driver.identify().unwrap();
        assert!(driver.get_size() > 0);
        assert_eq!(driver.get_sector_size(), 512);
    }
}
