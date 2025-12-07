use core::fmt;
use core::mem::size_of;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;
use spin::Mutex;
use crate::fs::{VfsError as FsError}; // Alias VfsError to FsError to match usage
use crate::drivers::disk::Disk; // Use correct path for Disk trait
use core::convert::TryInto;

// Constantes pour FAT32
pub const BYTES_PER_SECTOR: u32 = 512;
pub const DIR_ENTRY_SIZE: usize = 32;  // Taille d'une entrée de répertoire en octets
const DIR_ENTRY_DELETED: u8 = 0xE5;    // Marqueur d'entrée supprimée
const DIR_ENTRY_LAST: u8 = 0x00;       // Dernière entrée valide
const ATTR_READ_ONLY: u8 = 0x01;
const ATTR_HIDDEN: u8 = 0x02;
const ATTR_SYSTEM: u8 = 0x04;
const ATTR_VOLUME_ID: u8 = 0x08;
const ATTR_DIRECTORY: u8 = 0x10;
const ATTR_ARCHIVE: u8 = 0x20;
const ATTR_LONG_NAME: u8 = ATTR_READ_ONLY | ATTR_HIDDEN | ATTR_SYSTEM | ATTR_VOLUME_ID;

const FAT32_EOC: u32 = 0x0FFFFFF8;      // End Of Clusterchain marker
const FAT32_FREE: u32 = 0x00000000;     // Cluster libre
const FAT32_BAD: u32 = 0x0FFFFFF7;      // Cluster défectueux

// Constantes pour les noms de fichiers longs (LFN)
const LFN_LAST: u8 = 0x40;
const LFN_DELETED: u8 = 0x80;

// Valeurs spéciales pour les clusters
const CLUSTER_ROOT: u32 = 2;  // Premier cluster utilisable (0 et 1 sont réservés)

// Constantes pour FAT32
const SECTORS_PER_CLUSTER: u32 = 8;
const RESERVED_SECTORS: u32 = 32;   // Secteurs réservés (typique pour FAT32)
const NUM_FATS: u8 = 2;             // Nombre de copies de la FAT
const ROOT_ENTRIES: u16 = 0;        // 0 pour FAT32 (le répertoire racine est un cluster)
const TOTAL_SECTORS_16: u16 = 0;    // Non utilisé en FAT32
const MEDIA_DESCRIPTOR: u8 = 0xF8;  // Disque dur
const SECTORS_PER_FAT: u32 = 0;     // À déterminer à partir du BPB
const SECTORS_PER_TRACK: u16 = 63;  // Valeur typique
const NUM_HEADS: u16 = 255;         // Valeur typique
const HIDDEN_SECTORS: u32 = 0;      // Ajuster selon la partition
const TOTAL_SECTORS_32: u32 = 0;    // À déterminer

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct BiosParameterBlock {
    pub jmp_boot: [u8; 3],          // Code de démarrage
    pub oem_name: [u8; 8],          // Nom du formatage
    pub bytes_per_sector: u16,      // Octets par secteur (généralement 512)
    pub sectors_per_cluster: u8,     // Secteurs par cluster
    pub reserved_sectors: u16,       // Secteurs réservés
    pub num_fats: u8,                // Nombre de tables FAT
    pub root_entries: u16,           // Entrées dans le répertoire racine (0 pour FAT32)
    pub total_sectors_16: u16,       // Nombre total de secteurs (si < 65536)
    pub media_descriptor: u8,        // Descripteur de média
    pub sectors_per_fat_16: u16,     // Secteurs par table FAT (FAT12/16)
    pub sectors_per_track: u16,      // Secteurs par piste
    pub num_heads: u16,              // Nombre de têtes
    pub hidden_sectors: u32,         // Secteurs cachés avant la partition
    pub total_sectors_32: u32,       // Nombre total de secteurs (si >= 65536)
    
    // Extended Boot Record (FAT32)
    pub sectors_per_fat_32: u32,     // Secteurs par table FAT (FAT32)
    pub flags: u16,                  // Flags
    pub fat_version: u16,            // Version FAT (0.0)
    pub root_cluster: u32,           // Cluster de départ du répertoire racine
    pub fs_info_sector: u16,         // Secteur d'information du FS
    pub backup_boot_sector: u16,     // Secteur de sauvegarde du secteur de boot
    pub reserved: [u8; 12],          // Réservé
    pub drive_number: u8,            // Numéro de lecteur
    pub nt_flags: u8,                // Flags Windows NT
    pub signature: u8,               // Signature (0x28 ou 0x29)
    pub volume_id: u32,              // ID de volume
    pub volume_label: [u8; 11],      // Étiquette de volume
    pub fs_type: [u8; 8],            // Type de système de fichiers ("FAT32   ")
    pub boot_code: [u8; 420],        // Code de démarrage
    pub boot_signature: u16,         // Signature de démarrage (0xAA55)
}

// Entrée de répertoire FAT32
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct DirEntry {
    pub name: [u8; 8],              // Nom du fichier (8 caractères)
    pub ext: [u8; 3],               // Extension (3 caractères)
    pub attr: u8,                   // Attributs
    pub nt_reserved: u8,            // Réservé pour Windows NT
    pub creation_time_tenth: u8,    // Dizaines de secondes de création
    pub creation_time: u16,         // Heure de création
    pub creation_date: u16,         // Date de création
    pub last_access_date: u16,      // Dernier accès
    pub first_cluster_hi: u16,      // Cluster haut (bits 16-31)
    pub write_time: u16,            // Heure de dernière modification
    pub write_date: u16,            // Date de dernière modification
    pub first_cluster_lo: u16,      // Cluster bas (bits 0-15)
    pub file_size: u32,             // Taille du fichier en octets
}

// Structure pour gérer une entrée de répertoire longue (LFN)
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct LfnEntry {
    pub order: u8,                   // Numéro d'ordre
    pub name1: [u16; 5],            // Première partie du nom (caractères 1-5)
    pub attr: u8,                    // Attributs (doit être ATTR_LONG_NAME)
    pub type_: u8,                   // Type d'entrée (0 pour LFN)
    pub checksum: u8,                // Checksum du nom court
    pub name2: [u16; 6],            // Deuxième partie du nom (caractères 6-11)
    pub first_cluster: u16,          // Premier cluster (toujours 0 pour LFN)
    pub name3: [u16; 2],            // Troisième partie du nom (caractères 12-13)
}

// Structure pour gérer le système de fichiers FAT32
pub struct FAT32<D: Disk> {
    disk: D,                        // Disque sous-jacent
    bpb: BiosParameterBlock,        // Boot Parameter Block
    fat_start: u64,                 // Début de la première FAT (en secteurs)
    data_start: u64,                // Début de la zone de données (en secteurs)
    current_dir_cluster: u32,       // Cluster du répertoire courant
    next_free_cluster: u32,         // Prochain cluster libre (pour l'allocation)
    free_cluster_count: u32,        // Nombre de clusters libres
    initialized: bool,              // Si le système de fichiers est initialisé
}

impl<D: Disk> FAT32<D> {
    /// Crée une nouvelle instance de FAT32
    pub fn new(disk: D, disk_offset: u64) -> Result<Self, FsError> {
        // Lire le secteur de démarrage (secteur 0)
        let mut bpb = BiosParameterBlock {
            jmp_boot: [0; 3],
            oem_name: [0; 8],
            bytes_per_sector: 0,
            sectors_per_cluster: 0,
            reserved_sectors: 0,
            num_fats: 0,
            root_entries: 0,
            total_sectors_16: 0,
            media_descriptor: 0,
            sectors_per_fat_16: 0,
            sectors_per_track: 0,
            num_heads: 0,
            hidden_sectors: 0,
            total_sectors_32: 0,
            sectors_per_fat_32: 0,
            flags: 0,
            fat_version: 0,
            root_cluster: 0,
            fs_info_sector: 0,
            backup_boot_sector: 0,
            reserved: [0; 12],
            drive_number: 0,
            nt_flags: 0,
            signature: 0,
            volume_id: 0,
            volume_label: [0; 11],
            fs_type: [0; 8],
            boot_code: [0; 420],
            boot_signature: 0,
        };

        // Lire le secteur de démarrage depuis le disque
        // disk_read(disk_offset, &mut bpb as *const _ as *mut u8, 512); -- COMMENTAIRE ORIGINAL, ON REMPLACE PAR:
        
        // Lecture réelle du BPB
        unsafe {
            // Hack pour lire structure packed dans buffer
            let mut buffer = [0u8; 512];
            disk.read(disk_offset, &mut buffer).map_err(|_| FsError::IoError)?;
            core::ptr::copy_nonoverlapping(buffer.as_ptr(), &mut bpb as *mut _ as *mut u8, 512);
        }

        // Vérifier la signature de démarrage
        if bpb.boot_signature != 0xAA55 {
            return Err(FsError::IoError);
        }

        // Vérifier que c'est bien un système de fichiers FAT32
        if &bpb.fs_type[0..4] != b"FAT32" {
            return Err(FsError::IoError);
        }

        // Calculer les positions importantes
        let fat_start = disk_offset + (bpb.hidden_sectors + bpb.reserved_sectors as u32) as u64 * 512;
        let root_dir_sectors = ((bpb.root_entries as u32 * 32) + (bpb.bytes_per_sector as u32 - 1)) / bpb.bytes_per_sector as u32;
        let data_start = fat_start + (bpb.sectors_per_fat_32 as u64 * bpb.num_fats as u64 * 512) + (root_dir_sectors as u64 * 512);

        Ok(FAT32 {
            disk, // Initialisation du champ disk
            bpb,
            fat_start,
            data_start,
            current_dir_cluster: bpb.root_cluster,
            next_free_cluster: bpb.root_cluster + 1, // Valeur initiale simple
            free_cluster_count: 0, // À calculer
            initialized: true,
        })
    }

    /// Lit un cluster depuis le disque
    fn read_cluster(&self, cluster: u32, buffer: &mut [u8]) -> Result<(), FsError> {
        if cluster < 2 || cluster >= 0x0FFFFFF0 {
            return Err(FsError::NotFound);
        }

        let sector = self.data_start + 
                    ((cluster - 2) as u64 * self.bpb.sectors_per_cluster as u64);
        
        // Lire chaque secteur du cluster
        let sectors_per_cluster = self.bpb.sectors_per_cluster as usize;
        let bytes_per_sector = self.bpb.bytes_per_sector as usize;
        
        for i in 0..sectors_per_cluster {
            let offset = i * bytes_per_sector;
            let sector_addr = sector + i as u64;
            
            if let Err(_) = self.disk.read(sector_addr, &mut buffer[offset..offset + bytes_per_sector]) {
                return Err(FsError::IoError);
            }
        }
        
        Ok(())
    }
    
    /// Écrit un cluster sur le disque
    fn write_cluster(&mut self, cluster: u32, data: &[u8]) -> Result<(), FsError> {
        if cluster < 2 || cluster >= 0x0FFFFFF0 {
            return Err(FsError::InvalidArgument);
        }

        let sector = self.data_start + 
                    ((cluster - 2) as u64 * self.bpb.sectors_per_cluster as u64);
        
        // Écrire chaque secteur du cluster
        let sectors_per_cluster = self.bpb.sectors_per_cluster as usize;
        let bytes_per_sector = self.bpb.bytes_per_sector as usize;
        
        for i in 0..sectors_per_cluster {
            let offset = i * bytes_per_sector;
            let sector_addr = sector + i as u64;
            
            if let Err(_) = self.disk.write(sector_addr, &data[offset..offset + bytes_per_sector]) {
                return Err(FsError::IoError);
            }
        }
        
        Ok(())
    }
    
    /// Lit une entrée FAT
    fn read_fat_entry(&self, cluster: u32) -> Result<u32, FsError> {
        let fat_offset = cluster * 4;  // 4 octets par entrée en FAT32
        let fat_sector = self.fat_start + (fat_offset / self.bpb.bytes_per_sector as u32) as u64;
        let entry_offset = (fat_offset % self.bpb.bytes_per_sector as u32) as usize;
        
        // Lire le secteur FAT
        let mut sector = vec![0u8; self.bpb.bytes_per_sector as usize];
        if let Err(_) = self.disk.read(fat_sector, &mut sector) {
            return Err(FsError::IoError);
        }
        
        // Extraire la valeur du cluster
        let next_cluster = u32::from_le_bytes([
            sector[entry_offset],
            sector[entry_offset + 1],
            sector[entry_offset + 2],
            sector[entry_offset + 3],
        ]) & 0x0FFFFFFF;  // Masquer les 4 bits supérieurs
        
        Ok(next_cluster)
    }
    
    /// Écrit une entrée FAT
    fn write_fat_entry(&mut self, cluster: u32, value: u32) -> Result<(), FsError> {
        let fat_offset = cluster * 4;  // 4 octets par entrée en FAT32
        let fat_sector = self.fat_start + (fat_offset / self.bpb.bytes_per_sector as u32) as u64;
        let entry_offset = (fat_offset % self.bpb.bytes_per_sector as u32) as usize;
        
        // Lire le secteur FAT existant
        let mut sector = vec![0u8; self.bpb.bytes_per_sector as usize];
        if let Err(_) = self.disk.read(fat_sector, &mut sector) {
            return Err(FsError::IoError);
        }
        
        // Mettre à jour la valeur du cluster
        let value_bytes = value.to_le_bytes();
        for i in 0..4 {
            sector[entry_offset + i] = value_bytes[i];
        }
        
        // Écrire le secteur FAT mis à jour
        if let Err(_) = self.disk.write(fat_sector, &sector) {
            return Err(FsError::IoError);
        }
        
        // Mettre à jour la copie de la FAT si nécessaire
        if self.bpb.num_fats > 1 {
            let fat_size = self.bpb.sectors_per_fat_32 as u64 * self.bpb.bytes_per_sector as u64;
            for i in 1..self.bpb.num_fats as u64 {
                let backup_fat_sector = fat_sector + (i * fat_size);
                if let Err(_) = self.disk.write(backup_fat_sector, &sector) {
                    return Err(FsError::IoError);
                }
            }
        }
        
        Ok(())
    }

    /// Trouve le prochain cluster dans la chaîne FAT
    fn get_next_cluster(&self, current_cluster: u32) -> Result<u32, FsError> {
        let next_cluster = self.read_fat_entry(current_cluster)?;
        
        // Vérifier si c'est la fin de la chaîne
        if next_cluster >= 0x0FFFFFF8 {
            return Err(FsError::NotFound);
        }
        
        // Vérifier si le cluster est valide
        if next_cluster == FAT32_FREE || next_cluster == FAT32_BAD || next_cluster == 1 {
            return Err(FsError::IoError);
        }

        Ok(next_cluster)
    }
    
    /// Trouve un cluster libre
    fn find_free_cluster(&mut self, start_from: Option<u32>) -> Result<u32, FsError> {
        let start_cluster = start_from.unwrap_or(2); // Commencer après le cluster réservé
        
        // Si on a un indice de cluster libre, on vérifie d'abord celui-là
        if let Some(next_free) = self.next_free_cluster.checked_sub(2) {
            if next_free >= start_cluster {
                // Vérifier si le cluster est toujours libre
                match self.read_fat_entry(next_free) {
                    Ok(FAT32_FREE) => {
                        self.next_free_cluster = next_free + 1;
                        return Ok(next_free);
                    },
                    _ => {}
                }
            }
        }
        
        // Parcourir la FAT pour trouver un cluster libre
        let total_clusters = (self.bpb.total_sectors_32 / self.bpb.sectors_per_cluster as u32) as u32;
        
        for cluster in start_cluster..total_clusters {
            match self.read_fat_entry(cluster) {
                Ok(FAT32_FREE) => {
                    self.next_free_cluster = cluster + 1;
                    return Ok(cluster);
                },
                Ok(_) => {},
                Err(_) => break,
            }
        }
        
        Err(FsError::NoSpace)
    }
    
    /// Alloue une nouvelle chaîne de clusters
    fn allocate_cluster_chain(&mut self, count: u32) -> Result<u32, FsError> {
        if count == 0 {
            return Err(FsError::InvalidArgument);
        }
        
        // Trouver le premier cluster libre
        let first_cluster = self.find_free_cluster(None)?;
        let mut current_cluster = first_cluster;
        
        // Allouer les clusters supplémentaires si nécessaire
        for _ in 1..count {
            let next_cluster = self.find_free_cluster(Some(current_cluster + 1))?;
            
            // Mettre à jour la FAT pour pointer vers le prochain cluster
            self.write_fat_entry(current_cluster, next_cluster)?;
            current_cluster = next_cluster;
        }
        
        // Marquer la fin de la chaîne
        self.write_fat_entry(current_cluster, FAT32_EOC)?;
        
        Ok(first_cluster)
    }
    
    /// Libère une chaîne de clusters
    fn free_cluster_chain(&mut self, start_cluster: u32) -> Result<(), FsError> {
        let mut current_cluster = start_cluster;
        
        while current_cluster < 0x0FFFFFF8 {
            // Lire le prochain cluster avant d'effacer l'entrée actuelle
            let next_cluster = self.read_fat_entry(current_cluster)?;
            
            // Marquer le cluster comme libre
            self.write_fat_entry(current_cluster, FAT32_FREE)?;
            
            // Mettre à jour le compteur de clusters libres
            self.free_cluster_count = self.free_cluster_count.saturating_add(1);
            
            // Mettre à jour le prochain cluster libre si nécessaire
            if current_cluster < self.next_free_cluster {
                self.next_free_cluster = current_cluster;
            }
            
            // Passer au cluster suivant
            if next_cluster >= 0x0FFFFFF8 || next_cluster == FAT32_FREE {
                break;
            }
            current_cluster = next_cluster;
        }
        
        Ok(())
    }

    /// Trouve un fichier dans le répertoire courant
    pub fn find_file(&self, name: &str) -> Result<DirEntry, FsError> {
        let mut current_cluster = self.current_dir_cluster;
        let mut buffer = vec![0u8; self.bpb.bytes_per_sector as usize * self.bpb.sectors_per_cluster as usize];
        let mut lfn_entries = Vec::new();
        let short_name = Self::to_short_name(name);
        
        loop {
            // Lire le cluster actuel
            self.read_cluster(current_cluster, &mut buffer)?;
            
            // Parcourir chaque entrée de répertoire dans le cluster
            for entry_pos in (0..buffer.len()).step_by(DIR_ENTRY_SIZE) {
                let entry_slice = &buffer[entry_pos..entry_pos + DIR_ENTRY_SIZE];
                let first_byte = entry_slice[0];
                
                // Vérifier si c'est la fin du répertoire
                if first_byte == DIR_ENTRY_LAST {
                    return Err(FsError::NotFound);
                }
                
                // Ignorer les entrées supprimées
                if first_byte == DIR_ENTRY_DELETED {
                    lfn_entries.clear();
                    continue;
                }
                
                // Vérifier si c'est une entrée LFN (Long File Name)
                if (entry_slice[11] & ATTR_LONG_NAME) == ATTR_LONG_NAME {
                    // C'est une entrée LFN, la stocker pour plus tard
                    let lfn_entry = unsafe { &*(entry_slice.as_ptr() as *const LfnEntry) };
                    lfn_entries.push(lfn_entry.clone());
                    continue;
                }
                
                // C'est une entrée de répertoire standard
                let dir_entry = unsafe { &*(entry_slice.as_ptr() as *const DirEntry) };
                
                // Vérifier si c'est un fichier ou un répertoire valide
                if dir_entry.name[0] != 0x00 && dir_entry.name[0] != 0xE5 {
                    // Vérifier si le nom correspond
                    let entry_short_name = Self::format_short_name(&dir_entry.name, &dir_entry.ext);
                    
                    if entry_short_name == short_name {
                        return Ok(*dir_entry);
                    }
                    
                    // Vérifier si on a des entrées LFN pour ce fichier
                    if !lfn_entries.is_empty() {
                        let lfn = Self::decode_lfn_entries(&lfn_entries);
                        if lfn.eq_ignore_ascii_case(name) {
                            return Ok(*dir_entry);
                        }
                        lfn_entries.clear();
                    }
                } else {
                    lfn_entries.clear();
                }
            }
            
            // Passer au cluster suivant dans la chaîne
            match self.get_next_cluster(current_cluster) {
                Ok(next_cluster) => current_cluster = next_cluster,
                Err(_) => break, // Fin de la chaîne
            }
        }
        
        Err(FsError::NotFound)
    }
    
    /// Lit un fichier dans le système de fichiers
    pub fn read_file(&self, path: &str) -> Result<Vec<u8>, FsError> {
        // Trouver le fichier
        let entry = self.find_file(path)?;
        
        // Vérifier que c'est bien un fichier
        if (entry.attr & ATTR_DIRECTORY) != 0 {
            return Err(FsError::IsDirectory);
        }
        
        // Calculer le nombre de clusters nécessaires
        let cluster_size = self.bpb.bytes_per_sector as u64 * self.bpb.sectors_per_cluster as u64;
        let mut remaining_size = entry.file_size as u64;
        let mut data = Vec::with_capacity(entry.file_size as usize);
        let mut buffer = vec![0u8; cluster_size as usize];
        
        // Lire le premier cluster
        let mut current_cluster = ((entry.first_cluster_hi as u32) << 16) | (entry.first_cluster_lo as u32);
        
        while remaining_size > 0 {
            // Lire le cluster actuel
            self.read_cluster(current_cluster, &mut buffer)?;
            
            // Déterminer la quantité de données à copier
            let to_copy = core::cmp::min(remaining_size, cluster_size) as usize;
            data.extend_from_slice(&buffer[..to_copy]);
            remaining_size -= to_copy as u64;
            
            // Passer au cluster suivant
            match self.get_next_cluster(current_cluster) {
                Ok(next_cluster) => current_cluster = next_cluster,
                Err(_) if remaining_size > 0 => return Err(FsError::IoError),
                Err(_) => break,
            }
        }
        
        Ok(data)
    }
    
    /// Écrit un fichier dans le système de fichiers
    pub fn write_file(&mut self, path: &str, data: &[u8]) -> Result<(), FsError> {
        // Vérifier si le fichier existe déjà
        let entry = self.find_file(path);
        
        // Si le fichier existe, le supprimer d'abord
        if let Ok(entry) = entry {
            self.remove_file(path)?;
        }
        
        // Calculer le nombre de clusters nécessaires
        let cluster_size = self.bpb.bytes_per_sector as usize * self.bpb.sectors_per_cluster as usize;
        let clusters_needed = (data.len() + cluster_size - 1) / cluster_size;
        
        // Allouer les clusters nécessaires
        let first_cluster = self.allocate_cluster_chain(clusters_needed as u32)?;
        
        // Créer une nouvelle entrée de répertoire
        let mut dir_entry = DirEntry {
            name: [b' '; 8],
            ext: [b' '; 3],
            attr: ATTR_ARCHIVE,
            nt_reserved: 0,
            creation_time_tenth: 0, // TODO: Implémenter la gestion de l'horodatage
            creation_time: 0,
            creation_date: 0,
            last_access_date: 0,
            first_cluster_hi: (first_cluster >> 16) as u16,
            write_time: 0,
            write_date: 0,
            first_cluster_lo: (first_cluster & 0xFFFF) as u16,
            file_size: data.len() as u32,
        };
        
        // Définir le nom court du fichier
        let (name, ext) = Self::split_filename(path);
        dir_entry.name[..name.len()].copy_from_slice(&name.as_bytes()[..name.len().min(8)]);
        if !ext.is_empty() {
            dir_entry.ext[..ext.len()].copy_from_slice(&ext.as_bytes()[..ext.len().min(3)]);
        }
        
        // Écrire les données dans les clusters alloués
        let mut remaining_data = data;
        let mut current_cluster = first_cluster;
        
        while !remaining_data.is_empty() {
            // Déterminer la taille des données à écrire dans ce cluster
            let chunk_size = core::cmp::min(remaining_data.len(), cluster_size);
            let chunk = &remaining_data[..chunk_size];
            
            // Préparer le buffer avec les données
            let mut buffer = vec![0u8; cluster_size];
            buffer[..chunk_size].copy_from_slice(chunk);
            
            // Écrire le cluster
            self.write_cluster(current_cluster, &buffer)?;
            
            // Passer au cluster suivant
            remaining_data = &remaining_data[chunk_size..];
            if !remaining_data.is_empty() {
                current_cluster = self.get_next_cluster(current_cluster)?;
            }
        }
        
        // Ajouter l'entrée de répertoire
        self.add_directory_entry(&dir_entry)
    }
    
    /// Ajoute une entrée au répertoire courant
    fn add_directory_entry(&mut self, entry: &DirEntry) -> Result<(), FsError> {
        let mut current_cluster = self.current_dir_cluster;
        let mut buffer = vec![0u8; self.bpb.bytes_per_sector as usize * self.bpb.sectors_per_cluster as usize];
        
        loop {
            // Lire le cluster actuel
            self.read_cluster(current_cluster, &mut buffer)?;
            
            // Chercher une entrée libre
            for entry_pos in (0..buffer.len()).step_by(DIR_ENTRY_SIZE) {
                let entry_byte = buffer[entry_pos];
                
                // Vérifier si c'est une entrée libre ou supprimée
                if entry_byte == DIR_ENTRY_LAST || entry_byte == DIR_ENTRY_DELETED {
                    // Écrire la nouvelle entrée
                    let entry_slice = unsafe {
                        core::slice::from_raw_parts(
                            entry as *const _ as *const u8,
                            core::mem::size_of::<DirEntry>()
                        )
                    };
                    
                    buffer[entry_pos..entry_pos + DIR_ENTRY_SIZE].copy_from_slice(entry_slice);
                    
                    // Écrire le cluster mis à jour
                    self.write_cluster(current_cluster, &buffer)?;
                    return Ok(());
                }
            }
            
            // Passer au cluster suivant ou en allouer un nouveau
            match self.get_next_cluster(current_cluster) {
                Ok(next_cluster) => current_cluster = next_cluster,
                Err(_) => {
                    // Fin de la chaîne, allouer un nouveau cluster
                    let new_cluster = self.allocate_cluster_chain(1)?;
                    self.write_fat_entry(current_cluster, new_cluster)?;
                    
                    // Initialiser le nouveau cluster avec des zéros
                    let cluster_size = self.bpb.bytes_per_sector as usize * self.bpb.sectors_per_cluster as usize;
                    let zero_buffer = vec![0u8; cluster_size];
                    self.write_cluster(new_cluster, &zero_buffer)?;
                    
                    // Écrire la nouvelle entrée au début du nouveau cluster
                    let entry_slice = unsafe {
                        core::slice::from_raw_parts(
                            entry as *const _ as *const u8,
                            core::mem::size_of::<DirEntry>()
                        )
                    };
                    
                    buffer[..DIR_ENTRY_SIZE].copy_from_slice(entry_slice);
                    self.write_cluster(new_cluster, &buffer)?;
                    
                    return Ok(());
                }
            }
        }
    }
    
    /// Supprime un fichier du système de fichiers
    pub fn remove_file(&mut self, path: &str) -> Result<(), FsError> {
        // Trouver le fichier
        let entry = self.find_file(path)?;
        
        // Vérifier que c'est bien un fichier
        if (entry.attr & ATTR_DIRECTORY) != 0 {
            return Err(FsError::IsDirectory);
        }
        
        // Libérer les clusters alloués
        let first_cluster = ((entry.first_cluster_hi as u32) << 16) | (entry.first_cluster_lo as u32);
        self.free_cluster_chain(first_cluster)?;
        
        // Marquer l'entrée comme supprimée
        self.mark_entry_deleted(path)
    }
    
    /// Marque une entrée de répertoire comme supprimée
    fn mark_entry_deleted(&mut self, path: &str) -> Result<(), FsError> {
        let mut current_cluster = self.current_dir_cluster;
        let mut buffer = vec![0u8; self.bpb.bytes_per_sector as usize * self.bpb.sectors_per_cluster as usize];
        let short_name = Self::to_short_name(path);
        
        loop {
            // Lire le cluster actuel
            self.read_cluster(current_cluster, &mut buffer)?;
            
            // Parcourir chaque entrée de répertoire dans le cluster
            for entry_pos in (0..buffer.len()).step_by(DIR_ENTRY_SIZE) {
                let entry_slice = &mut buffer[entry_pos..entry_pos + DIR_ENTRY_SIZE];
                let first_byte = entry_slice[0];
                
                // Vérifier si c'est la fin du répertoire
                if first_byte == DIR_ENTRY_LAST {
                    return Err(FsError::NotFound);
                }
                
                // Ignorer les entrées supprimées
                if first_byte == DIR_ENTRY_DELETED {
                    continue;
                }
                
                // Vérifier si c'est le fichier qu'on cherche
                let dir_entry = unsafe { &*(entry_slice.as_ptr() as *const DirEntry) };
                let entry_short_name = Self::format_short_name(&dir_entry.name, &dir_entry.ext);
                
                if entry_short_name == short_name {
                    // Marquer l'entrée comme supprimée
                    entry_slice[0] = DIR_ENTRY_DELETED;
                    
                    // Écrire le cluster mis à jour
                    self.write_cluster(current_cluster, &buffer)?;
                    return Ok(());
                }
            }
            
            // Passer au cluster suivant
            match self.get_next_cluster(current_cluster) {
                Ok(next_cluster) => current_cluster = next_cluster,
                Err(_) => break, // Fin de la chaîne
            }
        }
        
        Err(FsError::NotFound)
    }

    /// Lit un fichier dans le système de fichiers
    
    /// Convertit un nom de fichier en format 8.3
    fn to_short_name(name: &str) -> String {
        let name = name.to_ascii_uppercase();
        let (name_part, ext_part) = match name.rfind('.') {
            Some(dot_pos) => (&name[..dot_pos], &name[dot_pos + 1..]),
            None => (name.as_str(), ""),
        };
        
        // Tronquer ou compléter avec des espaces
        let name_part = if name_part.len() > 8 { &name_part[..8] } else { name_part };
        let ext_part = if ext_part.len() > 3 { &ext_part[..3] } else { ext_part };
        
        alloc::format!("{:8.8}{:3.3}", name_part, ext_part)
    }
    
    /// Formate un nom de fichier 8.3
    fn format_short_name(name: &[u8; 8], ext: &[u8; 3]) -> String {
        let name_str = String::from_utf8_lossy(name).trim_end().into();
        let ext_str: String = String::from_utf8_lossy(ext).trim_end().into();
        
        if ext_str.is_empty() {
            name_str
        } else {
            alloc::format!("{}.{}", name_str, ext_str)
        }
    }
    
    /// Sépare un nom de fichier en nom et extension
    fn split_filename(path: &str) -> (String, String) {
        let path = path.trim_start_matches('/');
        let name = match path.rfind('/') {
            Some(pos) => &path[pos + 1..],
            None => path,
        };
        
        match name.rfind('.') {
            Some(dot_pos) => (name[..dot_pos].into(), name[dot_pos + 1..].into()),
            None => (name.into(), String::new()),
        }
    }
    
    /// Décode une entrée de nom de fichier long (LFN)
    fn decode_lfn_entries(entries: &[LfnEntry]) -> String {
        let mut result = String::new();
        
        // Trier les entrées par ordre décroissant (l'ordre est stocké dans les 6 bits de poids faible)
        let mut sorted_entries = entries.to_vec();
        sorted_entries.sort_by_key(|e| e.order & 0x3F);
        
        // Concaténer les parties du nom
        for entry in sorted_entries {
            // Ajouter les caractères de chaque partie du nom
            for c in entry.name1 {
                if c != 0xFFFF && c != 0x0000 {
                    if let Some(ch) = char::from_u32(c as u32) {
                        result.push(ch);
                    }
                }
            }
            
            for c in entry.name2 {
                if c != 0xFFFF && c != 0x0000 {
                    if let Some(ch) = char::from_u32(c as u32) {
                        result.push(ch);
                    }
                }
            }
            
            for c in entry.name3 {
                if c != 0xFFFF && c != 0x0000 {
                    if let Some(ch) = char::from_u32(c as u32) {
                        result.push(ch);
                    }
                }
            }
        }
        
        result
    }

    /// Lit les entrées racine ou d'un répertoire
    pub fn read_dir(&self, path: &str) -> Result<Vec<String>, FsError> {
        let mut entries = Vec::new();
        let mut current_cluster = if path.is_empty() || path == "/" {
            self.bpb.root_cluster
        } else {
            // Trouver le cluster du répertoire
            let entry = self.find_file(path)?;
            if (entry.attr & ATTR_DIRECTORY) == 0 {
                return Err(FsError::NotDirectory);
            }
            ((entry.first_cluster_hi as u32) << 16) | (entry.first_cluster_lo as u32)
        };
        
        loop {
            // Taille d'un cluster en octets
            let cluster_size = self.bpb.bytes_per_sector as usize * self.bpb.sectors_per_cluster as usize;
            let mut buffer = vec![0u8; cluster_size];
            
            // Lire le contenu du cluster
            self.read_cluster(current_cluster, &mut buffer)?;
            
            // Parser les entrées
            let mut lfn_entries: Vec<LfnEntry> = Vec::new();
            
            for entry_slice in buffer.chunks(DIR_ENTRY_SIZE) {
                if entry_slice[0] == 0x00 {
                    // Fin du répertoire
                    return Ok(entries);
                }
                
                if entry_slice[0] == 0xE5 {
                    // Entrée supprimée
                    continue;
                }
                
                // Vérifier si c'est une entrée LFN
                if (entry_slice[11] & ATTR_LONG_NAME) == ATTR_LONG_NAME {
                    // C'est une entrée LFN, la stocker pour plus tard
                    let lfn_entry = unsafe { &*(entry_slice.as_ptr() as *const LfnEntry) };
                    lfn_entries.push(lfn_entry.clone());
                    continue;
                }
                
                // C'est une entrée de répertoire standard
                let dir_entry = unsafe { &*(entry_slice.as_ptr() as *const DirEntry) };
                
                // Vérifier si c'est un fichier ou un répertoire valide
                if dir_entry.name[0] != 0x00 && dir_entry.name[0] != 0xE5 {
                    let entry_name = if !lfn_entries.is_empty() {
                        // Utiliser le nom long si disponible
                        let lfn = Self::decode_lfn_entries(&lfn_entries);
                        lfn_entries.clear();
                        lfn
                    } else {
                        // Sinon utiliser le nom court
                        Self::format_short_name(&dir_entry.name, &dir_entry.ext)
                    };
                    
                    // Ignorer les entrées spéciales . et ..
                    if entry_name != "." && entry_name != ".." {
                        entries.push(entry_name);
                    }
                } else {
                    lfn_entries.clear();
                }
            }
            
            // Passer au cluster suivant dans la chaîne
            match self.get_next_cluster(current_cluster) {
                Ok(next_cluster) => current_cluster = next_cluster,
                Err(_) => break, // Fin de la chaîne
            }
        }
        
        Ok(entries)
    }
}
