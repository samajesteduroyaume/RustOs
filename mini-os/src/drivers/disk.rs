use super::{Driver, DriverError};
extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use crate::vga_buffer::WRITER;
use x86_64::instructions::port::{Port, PortReadOnly, PortWriteOnly};
use spin::Mutex;

/// Erreurs spécifiques au driver disque
#[derive(Debug, Clone, Copy)]
pub enum DiskError {
    InvalidSector,
    BufferTooSmall,
    InvalidSize,
    ReadFailed,
    WriteFailed,
    Timeout,
    NotReady,
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

struct AtaPorts {
    data: Port<u16>,
    error: PortReadOnly<u8>,
    sector_count: Port<u8>,
    lba_low: Port<u8>,
    lba_mid: Port<u8>,
    lba_high: Port<u8>,
    device: Port<u8>,
    status: PortReadOnly<u8>,
    command: PortWriteOnly<u8>,
}

impl AtaPorts {
    fn new(base: u16) -> Self {
        // Base is usually 0x1F0 for primary
        Self {
            data: Port::new(base),
            error: PortReadOnly::new(base + 1),
            sector_count: Port::new(base + 2),
            lba_low: Port::new(base + 3),
            lba_mid: Port::new(base + 4),
            lba_high: Port::new(base + 5),
            device: Port::new(base + 6),
            status: PortReadOnly::new(base + 7),
            command: PortWriteOnly::new(base + 7),
        }
    }
}

/// Driver disque ATA/SATA
pub struct DiskDriver {
    pub name: String,
    pub sectors: u64,
    pub sector_size: u16,
    pub initialized: bool,
    pub primary_master: bool,
    
    // Ports wrapped in Mutex for interior mutability
    ports: Mutex<AtaPorts>,
}

pub trait Disk {
    fn read(&self, check: u64, buffer: &mut [u8]) -> Result<(), DiskError>;
    fn write(&mut self, check: u64, buffer: &[u8]) -> Result<(), DiskError>;
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
            ports: Mutex::new(AtaPorts::new(ata_ports::PRIMARY_DATA)),
        }
    }
    
    /// Attend que le disque soit prêt (pas BUSY, et READY)
    fn wait_ready(ports: &mut AtaPorts) -> Result<(), DiskError> {
        for _ in 0..10000 {
            let status = unsafe { ports.status.read() };
            if status & ata_status::BUSY == 0 {
                return Ok(());
            }
        }
        Err(DiskError::Timeout)
    }
    
    /// Attend que le disque soit prêt pour le transfert (DRQ)
    fn wait_drq(ports: &mut AtaPorts) -> Result<(), DiskError> {
        for _ in 0..10000 {
            let status = unsafe { ports.status.read() };
            if status & ata_status::ERROR != 0 {
                return Err(DiskError::ReadFailed);
            }
            if status & ata_status::DATA_REQUEST != 0 {
                return Ok(());
            }
        }
        Err(DiskError::Timeout)
    }

    /// Lit un secteur depuis le disque
    pub fn read_sector(&self, lba: u64, buffer: &mut [u8]) -> Result<(), DiskError> {
        if buffer.len() < self.sector_size as usize {
            return Err(DiskError::BufferTooSmall);
        }

        let mut ports = self.ports.lock();
        Self::wait_ready(&mut ports)?;

        unsafe {
            let drive_select = if self.primary_master { 0xE0 } else { 0xF0 };
            ports.device.write(drive_select | ((lba >> 24) & 0x0F) as u8);
            ports.sector_count.write(1);
            ports.lba_low.write(lba as u8);
            ports.lba_mid.write((lba >> 8) as u8);
            ports.lba_high.write((lba >> 16) as u8);
            ports.command.write(ata_commands::READ_SECTORS);
        }
        
        Self::wait_drq(&mut ports)?;
        
        for i in 0..256 {
            let data = unsafe { ports.data.read() };
            buffer[i*2] = (data & 0xFF) as u8;
            buffer[i*2+1] = ((data >> 8) & 0xFF) as u8;
        }
        
        unsafe { ports.status.read() }; // Clear status

        Ok(())
    }

    /// Écrit un secteur sur le disque
    pub fn write_sector(&self, lba: u64, data: &[u8]) -> Result<(), DiskError> {
        if data.len() < self.sector_size as usize {
            return Err(DiskError::InvalidSize);
        }

        let mut ports = self.ports.lock();
        Self::wait_ready(&mut ports)?;

        unsafe {
            let drive_select = if self.primary_master { 0xE0 } else { 0xF0 };
            ports.device.write(drive_select | ((lba >> 24) & 0x0F) as u8);
            ports.sector_count.write(1);
            ports.lba_low.write(lba as u8);
            ports.lba_mid.write((lba >> 8) as u8);
            ports.lba_high.write((lba >> 16) as u8);
            ports.command.write(ata_commands::WRITE_SECTORS);
        }
        
        Self::wait_drq(&mut ports)?;
        
        for i in 0..256 {
            let word = (data[i*2] as u16) | ((data[i*2+1] as u16) << 8);
            unsafe { ports.data.write(word) };
        }
        
        Self::wait_ready(&mut ports)?;

        Ok(())
    }

    /// Identifie le disque
    pub fn identify(&mut self) -> Result<(), DiskError> {
        let mut ports = self.ports.lock();
        
        unsafe {
            let drive_select = if self.primary_master { 0xA0 } else { 0xB0 };
            ports.device.write(drive_select);
            ports.lba_low.write(0);
            ports.lba_mid.write(0);
            ports.lba_high.write(0);
            ports.command.write(ata_commands::IDENTIFY);
        }
        
        let status = unsafe { ports.status.read() };
        if status == 0 {
            return Err(DiskError::NotReady);
        }
        
        self.sectors = 1000000;
        self.sector_size = 512;
        self.initialized = true;

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

// Implémentation du trait Driver
impl Driver for DiskDriver {
    fn name(&self) -> &str {
        &self.name
    }

    fn init(&mut self) -> Result<(), DriverError> {
        self.initialized = true;
        self.sectors = 204800; // 100MB
        Ok(())
    }

    fn handle_interrupt(&mut self, _irq: u8) {}

    fn shutdown(&mut self) -> Result<(), DriverError> {
        self.initialized = false;
        Ok(())
    }
}

// Implémentation du trait Disk pour l'abstraction FS
impl Disk for DiskDriver {
    fn read(&self, sector: u64, buffer: &mut [u8]) -> Result<(), DiskError> {
        self.read_sector(sector, buffer)
    }
    
    fn write(&mut self, sector: u64, buffer: &[u8]) -> Result<(), DiskError> {
        // Now calling write_sector which takes &self
        self.write_sector(sector, buffer)
    }
}
