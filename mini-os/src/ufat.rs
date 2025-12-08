use core::fmt;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::BTreeMap;
use spin::Mutex;
use crate::filesystem::{FsError, NodeType, Metadata};
use crate::disk::Disk;

// Constantes pour UFAT
const UFAT_MAGIC: u32 = 0x55464154; // 'UFAT' en ASCII
const UFAT_VERSION: u32 = 1;
const DEFAULT_BLOCK_SIZE: u32 = 4096; // 4KB par défaut
const INODES_PER_GROUP: u32 = 1024;
const BLOCKS_PER_GROUP: u32 = 32768; // 128MB par groupe avec des blocs de 4KB
const MAX_FILENAME_LENGTH: usize = 255;

// Types de fichiers
const UFAT_FT_UNKNOWN: u8 = 0;
const UFAT_FT_REG_FILE: u8 = 1;
const UFAT_FT_DIR: u8 = 2;
const UFAT_FT_SYMLINK: u8 = 3;

// Drapeaux d'inode
const UFAT_IFLAG_SYNC: u32 = 0x0001;      // Les écritures sont synchronisées
const UFAT_IFLAG_IMMUTABLE: u32 = 0x0002; // Fichier immuable
const UFAT_IFLAG_APPEND: u32 = 0x0004;    // Écritures en mode ajout uniquement
const UFAT_IFLAG_NODUMP: u32 = 0x0008;    // Ne pas sauvegarder avec dump
const UFAT_IFLAG_ENCRYPT: u32 = 0x0010;   // Fichier chiffré

// En-tête principal UFAT
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct UfatSuperBlock {
    pub magic: u32,                 // Signature UFAT (0x55464154)
    pub version: u32,               // Version du format
    pub block_size: u32,            // Taille des blocs en octets
    pub block_count: u64,           // Nombre total de blocs
    pub free_blocks: u64,           // Nombre de blocs libres
    pub inode_count: u64,           // Nombre total d'inodes
    pub free_inodes: u64,           // Nombre d'inodes libres
    pub first_data_block: u32,      // Premier bloc de données
    pub inodes_per_group: u32,      // Nombre d'inodes par groupe
    pub blocks_per_group: u32,      // Nombre de blocs par groupe
    pub volume_name: [u8; 32],      // Nom du volume (UTF-8, non terminé par zéro)
    pub last_mount: u64,            // Dernier montage (timestamp Unix)
    pub last_write: u64,            // Dernière écriture (timestamp Unix)
    pub mount_count: u32,           // Nombre de montages depuis le dernier fsck
    pub max_mounts: u32,            // Nombre maximal de montages avant fsck
    pub checksum: u32,              // Checksum de l'en-tête
    pub reserved: [u8; 448],        // Réservé pour extensions futures
}

// Descripteur de groupe de blocs
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct BlockGroupDescriptor {
    pub block_bitmap: u32,          // Bloc contenant le bitmap de blocs
    pub inode_bitmap: u32,          // Bloc contenant le bitmap d'inodes
    pub inode_table: u32,           // Premier bloc de la table d'inodes
    pub free_blocks: u16,           // Nombre de blocs libres dans le groupe
    pub free_inodes: u16,           // Nombre d'inodes libres dans le groupe
    pub used_dirs: u16,             // Nombre de répertoires dans le groupe
    pub flags: u16,                 // Drapeaux du groupe
    pub checksum: u32,              // Checksum du groupe
    pub reserved: [u8; 12],         // Réservé pour extensions futures
}

// Inode UFAT
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct UfatInode {
    pub mode: u16,                  // Type et permissions
    pub uid: u16,                   // ID du propriétaire
    pub size: u64,                  // Taille en octets
    pub atime: u64,                 // Dernier accès (timestamp Unix)
    pub ctime: u64,                 // Création (timestamp Unix)
    pub mtime: u64,                 // Dernière modification (timestamp Unix)
    pub blocks: u64,                // Nombre de blocs alloués
    pub flags: u32,                 // Drapeaux (voir UFAT_IFLAG_*)
    // Pointeurs de blocs : 12 directs, 1 simple, 1 double, 1 triple
    pub block: [u32; 15],
    pub checksum: u32,              // Checksum des métadonnées
    pub reserved: [u8; 16],         // Réservé pour extensions futures
}

// Entrée de répertoire
#[repr(C, packed)]
#[derive(Debug, Clone)]
pub struct DirEntry {
    pub inode: u32,                 // Numéro d'inode
    pub name_len: u8,               // Longueur du nom
    pub file_type: u8,              // Type de fichier (UFAT_FT_*)
    pub name: [u8; MAX_FILENAME_LENGTH], // Nom du fichier (UTF-8)
}

// Structure principale du système de fichiers UFAT
pub struct UFAT<D: Disk> {
    disk: Mutex<D>,
    block_size: u32,
    block_count: u64,
    inode_count: u64,
    free_blocks: u64,
    free_inodes: u64,
    blocks_per_group: u32,
    inodes_per_group: u32,
}

impl<D: Disk> UFAT<D> {
    /// Crée une nouvelle instance de UFAT sur un périphérique de disque
    pub fn new(disk: D) -> Result<Self, FsError> {
        // Lire le superbloc (offset 0)
        let mut sb_buf = [0u8; 1024]; // Taille min
        // Note: Disk trait read prend u64 offset.
        // On suppose qu'on peut lire juste le début
        let mut locked_disk = disk.lock(); // On a besoin de lock pour lire
        // lock() returns MutexGuard.Disk? No, disk is D. UFAT has Mutex<D>.
        // Wait, `disk` param is `D`. `self.disk` is `Mutex<D>`.
        // So here `disk` is owned directly.
        
        // Read directly from disk
        let mut buf = vec![0u8; 4096]; // Read first block (assuming 4k or less)
        disk.read(0, &mut buf).map_err(|_| FsError::IOError)?;
        
        let sb_ptr = buf.as_ptr() as *const UfatSuperBlock;
        let sb = unsafe { sb_ptr.read_unaligned() };
        
        if sb.magic != UFAT_MAGIC {
            return Err(FsError::InvalidFilesystem);
        }
        
        Ok(Self {
            disk: Mutex::new(disk),
            block_size: sb.block_size,
            block_count: sb.block_count,
            inode_count: sb.inode_count,
            free_blocks: sb.free_blocks,
            free_inodes: sb.free_inodes,
            blocks_per_group: sb.blocks_per_group,
            inodes_per_group: sb.inodes_per_group,
        })
    }
    
    /// Formate un périphérique avec le système de fichiers UFAT
    pub fn format(mut disk: D, volume_name: &str) -> Result<(), FsError> {
        // 1. Vérifier les paramètres
        let disk_size = disk.size();
        let block_size = DEFAULT_BLOCK_SIZE as u64;
        let blocks_count = disk_size / block_size;
        
        if blocks_count < 16 {
            return Err(FsError::IOError); // Disque trop petit
        }
        
        // 2. Calculer la géométrie du système de fichiers
        let blocks_per_group = BLOCKS_PER_GROUP as u64;
        let inodes_per_group = INODES_PER_GROUP as u64;
        let group_count = (blocks_count + blocks_per_group - 1) / blocks_per_group;
        
        // 3. Initialiser le superbloc
        let mut superblock = UfatSuperBlock {
            magic: UFAT_MAGIC,
            version: UFAT_VERSION,
            block_size: block_size as u32,
            block_count: blocks_count,
            free_blocks: blocks_count - 1, // Moins le superbloc
            inode_count: group_count * inodes_per_group,
            free_inodes: group_count * inodes_per_group - 1, // Moins la racine
            first_data_block: 1, // Le superbloc est au bloc 0
            inodes_per_group: inodes_per_group as u32,
            blocks_per_group: blocks_per_group as u32,
            volume_name: [0; 32],
            last_mount: 0,
            last_write: 0, // Sera mis à jour plus tard
            mount_count: 0,
            max_mounts: 65535,
            checksum: 0,
            reserved: [0; 448],
        };
        
        // Copier le nom du volume
        let name_bytes = volume_name.as_bytes();
        let name_len = name_bytes.len().min(31);
        superblock.volume_name[..name_len].copy_from_slice(&name_bytes[..name_len]);
        
        // 4. Écrire le superbloc
        let superblock_bytes = unsafe {
            core::slice::from_raw_parts(
                &superblock as *const _ as *const u8,
                core::mem::size_of::<UfatSuperBlock>(),
            )
        };
        disk.write(0, superblock_bytes)?;
        
        // 5. Initialiser les groupes de blocs
        let mut current_block = 1; // Après le superbloc
        let bgdt_blocks = (group_count as usize * core::mem::size_of::<BlockGroupDescriptor>() + block_size as usize - 1) / block_size as usize;
        
        // Allouer de l'espace pour la table des groupes de blocs
        current_block += bgdt_blocks as u64;
        
        // 6. Initialiser chaque groupe
        for group in 0..group_count {
            let group_start = group * blocks_per_group;
            let inode_bitmap_block = current_block;
            let block_bitmap_block = inode_bitmap_block + 1;
            let inode_table_block = block_bitmap_block + 1;
            
            // Écrire le bitmap des inodes (tous libres sauf inode 0)
            let mut inode_bitmap = vec![0u8; block_size as usize];
            if group == 0 {
                inode_bitmap[0] = 0x01; // Marquer l'inode 0 comme utilisé (mauvais bloc)
                inode_bitmap[1] = 0x01; // Marquer l'inode 1 comme utilisé (répertoire racine)
            }
            disk.write(inode_bitmap_block * block_size, &inode_bitmap)?;
            
            // Écrire le bitmap des blocs
            let mut block_bitmap = vec![0u8; block_size as usize];
            // Marquer les blocs déjà utilisés (superbloc, GDT, etc.)
            for i in 0..current_block {
                block_bitmap[(i / 8) as usize] |= 1 << (i % 8);
            }
            disk.write(block_bitmap_block * block_size, &block_bitmap)?;
            
            // Initialiser la table d'inodes
            let inode_size = core::mem::size_of::<UfatInode>();
            let inodes_per_block = block_size as usize / inode_size;
            
            for i in 0..inodes_per_group as usize {
                let inode_block = inode_table_block + (i / inodes_per_block) as u64;
                let inode_offset = (i % inodes_per_block) * inode_size;
                
                let mut inode = UfatInode {
                    mode: 0,
                    uid: 0,
                    size: 0,
                    atime: 0,
                    ctime: 0,
                    mtime: 0,
                    blocks: 0,
                    flags: 0,
                    block: [0; 15],
                    checksum: 0,
                    reserved: [0; 16],
                };
                
                // Inode du répertoire racine (inode 1)
                if group == 0 && i == 1 {
                    inode.mode = 0o755 | ((UFAT_FT_DIR as u16) << 12);
                    inode.uid = 0; // root
                    inode.size = block_size as u64;
                    inode.ctime = 0; // TODO: Mettre à jour avec l'heure actuelle
                    inode.mtime = inode.ctime;
                    inode.atime = inode.ctime;
                    
                    // Allouer un bloc pour le répertoire racine
                    let root_block = current_block;
                    current_block += 1;
                    inode.block[0] = root_block as u32;
                    inode.blocks = 1;
                    
                    // Initialiser le bloc du répertoire racine
                    let root_dir = DirEntry {
                        inode: 1, // Self
                        name_len: 1,
                        file_type: UFAT_FT_DIR,
                        name: {
                            let mut name = [0; MAX_FILENAME_LENGTH];
                            name[0] = b'.';
                            name
                        },
                    };
                    
                    let mut root_block_data = vec![0u8; block_size as usize];
                    let root_dir_bytes = unsafe {
                        core::slice::from_raw_parts(
                            &root_dir as *const _ as *const u8,
                            core::mem::size_of::<DirEntry>(),
                        )
                    };
                    root_block_data[..root_dir_bytes.len()].copy_from_slice(root_dir_bytes);
                    
                    // Marquer le bloc comme utilisé
                    block_bitmap[(root_block / 8) as usize] |= 1 << (root_block % 8);
                    disk.write(block_bitmap_block * block_size, &block_bitmap)?;
                    
                    // Écrire le bloc du répertoire racine
                    disk.write(root_block * block_size, &root_block_data)?;
                }
                
                // Écrire l'inode
                let inode_bytes = unsafe {
                    core::slice::from_raw_parts(
                        &inode as *const _ as *const u8,
                        inode_size,
                    )
                };
                
                let mut inode_block_data = vec![0u8; block_size as usize];
                inode_block_data[inode_offset..inode_offset + inode_size].copy_from_slice(inode_bytes);
                disk.write(inode_block * block_size, &inode_block_data)?;
            }
            
            // Mettre à jour le groupe de blocs
            let bgd = BlockGroupDescriptor {
                block_bitmap: block_bitmap_block as u32,
                inode_bitmap: inode_bitmap_block as u32,
                inode_table: inode_table_block as u32,
                free_blocks: (blocks_per_group - (current_block - group_start)) as u16,
                free_inodes: (inodes_per_group - if group == 0 { 2 } else { 0 }) as u16,
                used_dirs: if group == 0 { 1 } else { 0 },
                flags: 0,
                checksum: 0, // TODO: Calculer le checksum
                reserved: [0; 12],
            };
            
            // Écrire le descripteur de groupe
            let bgd_bytes = unsafe {
                core::slice::from_raw_parts(
                    &bgd as *const _ as *const u8,
                    core::mem::size_of::<BlockGroupDescriptor>(),
                )
            };
            
            let bgdt_offset = (1 + (group as usize * core::mem::size_of::<BlockGroupDescriptor>()) / block_size as usize) as u64 * block_size;
            let bgdt_offset_in_block = (group as usize * core::mem::size_of::<BlockGroupDescriptor>()) % block_size as usize;
            
            let mut bgdt_block = vec![0u8; block_size as usize];
            if bgdt_offset_in_block + bgd_bytes.len() <= block_size as usize {
                bgdt_block[bgdt_offset_in_block..bgdt_offset_in_block + bgd_bytes.len()].copy_from_slice(bgd_bytes);
                disk.write(bgdt_offset, &bgdt_block)?;
            }
        }
        
        // Mettre à jour le superbloc avec les informations finales
        superblock.free_blocks = blocks_count - current_block;
        superblock.free_inodes = (group_count * inodes_per_group) - 2; // Moins inode 0 et 1
        superblock.last_write = 0; // TODO: Mettre à jour avec l'heure actuelle
        
        // Re-écrire le superbloc mis à jour
        let superblock_bytes = unsafe {
            core::slice::from_raw_parts(
                &superblock as *const _ as *const u8,
                core::mem::size_of::<UfatSuperBlock>(),
            )
        };
        disk.write(0, superblock_bytes)?;
        
        Ok(())
    }
    
    /// Alloue un bloc libre
    fn allocate_block(&mut self) -> Result<u64, FsError> {
        let groups_count = (self.block_count + self.blocks_per_group as u64 - 1) / self.blocks_per_group as u64;
        let blocks_per_group = self.blocks_per_group as u64;

        for group in 0..groups_count {
            let bgd = self.read_bgd(group)?;
            if bgd.free_blocks == 0 { continue; }

            let bitmap_block = bgd.block_bitmap as u64;
            let mut bitmap = vec![0u8; self.block_size as usize];
            self.read_block(bitmap_block, &mut bitmap)?;
            
            for (byte_idx, &byte) in bitmap.iter().enumerate() {
                if byte != 0xFF {
                    for bit_idx in 0..8 {
                        if (byte & (1 << bit_idx)) == 0 {
                            let relative_block = (byte_idx * 8 + bit_idx) as u64;
                            if relative_block >= blocks_per_group { break; }
                            
                            let absolute_block = group * blocks_per_group + relative_block;
                            if absolute_block >= self.block_count { return Err(FsError::NoSpace); }

                            let mut new_bitmap = bitmap.clone();
                            new_bitmap[byte_idx] |= 1 << bit_idx;
                            self.write_block(bitmap_block, &new_bitmap)?;
                            
                            // TODO: Mettre à jour free_blocks dans GDT et SB
                            
                            return Ok(absolute_block);
                        }
                    }
                }
            }
        }
        Err(FsError::NoSpace)
    }

    fn free_block(&mut self, block_num: u64) -> Result<(), FsError> {
        let blocks_per_group = self.blocks_per_group as u64;
        let group = block_num / blocks_per_group;
        let relative_block = block_num % blocks_per_group;
        
        let bgd = self.read_bgd(group)?;
        let bitmap_block = bgd.block_bitmap as u64;
        
        let mut bitmap = vec![0u8; self.block_size as usize];
        self.read_block(bitmap_block, &mut bitmap)?;
        
        let byte_idx = (relative_block / 8) as usize;
        let bit_idx = (relative_block % 8) as usize;
        
        bitmap[byte_idx] &= !(1 << bit_idx);
        self.write_block(bitmap_block, &bitmap)?;
        Ok(())
    }

    fn read_bgd(&self, group: u64) -> Result<BlockGroupDescriptor, FsError> {
        let block_size = self.block_size as u64;
        let bgd_size = core::mem::size_of::<BlockGroupDescriptor>() as u64;
        
        let gdt_start_block = 1; 
        let entries_per_block = block_size / bgd_size;
        let block_offset = group / entries_per_block;
        let entry_offset = group % entries_per_block;
        
        let block_num = gdt_start_block + block_offset;
        let mut buf = vec![0u8; block_size as usize];
        self.read_block(block_num, &mut buf)?;
        
        let start = (entry_offset * bgd_size) as usize;
        let end = start + bgd_size as usize;
        
        let ptr = buf[start..end].as_ptr() as *const BlockGroupDescriptor;
        let bgd = unsafe { ptr.read_unaligned() };
        Ok(bgd)
    }

    /// Lit un inode
    fn read_inode(&self, inode_num: u64) -> Result<UfatInode, FsError> {
        if inode_num == 0 || inode_num > self.inode_count {
            return Err(FsError::InvalidInode);
        }

        let group = (inode_num - 1) / self.inodes_per_group as u64;
        let index = (inode_num - 1) % self.inodes_per_group as u64;
        
        let bgd = self.read_bgd(group)?;
        let inode_table_block = bgd.inode_table as u64;
        let inode_size = core::mem::size_of::<UfatInode>() as u64;
        
        let block_offset = (index * inode_size) / self.block_size as u64;
        let byte_offset = (index * inode_size) % self.block_size as u64;
        
        let mut buf = vec![0u8; self.block_size as usize];
        self.read_block(inode_table_block + block_offset, &mut buf)?;
        
        let start = byte_offset as usize;
        let end = start + inode_size as usize;
        
        let ptr = buf[start..end].as_ptr() as *const UfatInode;
        let inode = unsafe { ptr.read_unaligned() };
        
        Ok(inode)
    }

    /// Écrit un inode
    fn write_inode(&self, inode_num: u64, inode: &UfatInode) -> Result<(), FsError> {
        if inode_num == 0 || inode_num > self.inode_count {
            return Err(FsError::InvalidInode);
        }

        let group = (inode_num - 1) / self.inodes_per_group as u64;
        let index = (inode_num - 1) % self.inodes_per_group as u64;
        
        let bgd = self.read_bgd(group)?;
        let inode_table_block = bgd.inode_table as u64;
        let inode_size = core::mem::size_of::<UfatInode>() as u64;
        
        let block_offset = (index * inode_size) / self.block_size as u64;
        let byte_offset = (index * inode_size) % self.block_size as u64;
        
        let block_num = inode_table_block + block_offset;
        let mut buf = vec![0u8; self.block_size as usize];
        self.read_block(block_num, &mut buf)?;
        
        let inode_bytes = unsafe {
            core::slice::from_raw_parts(
                inode as *const _ as *const u8,
                inode_size as usize,
            )
        };
        
        // Copier les octets de l'inode dans le buffer au bon offset
        let start = byte_offset as usize;
        buf[start..start + inode_size as usize].copy_from_slice(inode_bytes);
        
        self.write_block(block_num, &buf)?;
        
        Ok(())
    }

    /// Alloue un nouvel inode
    fn allocate_inode(&mut self) -> Result<u64, FsError> {
        let groups_count = (self.block_count + self.blocks_per_group as u64 - 1) / self.blocks_per_group as u64;

        for group in 0..groups_count {
            let bgd = self.read_bgd(group)?;
            if bgd.free_inodes == 0 { continue; }
            
            let bitmap_block = bgd.inode_bitmap as u64;
            let mut bitmap = vec![0u8; self.block_size as usize];
            self.read_block(bitmap_block, &mut bitmap)?;
            
            for (byte_idx, &byte) in bitmap.iter().enumerate() {
                if byte != 0xFF {
                    for bit_idx in 0..8 {
                        if (byte & (1 << bit_idx)) == 0 {
                            let relative_inode = (byte_idx * 8 + bit_idx) as u64;
                            if relative_inode >= self.inodes_per_group as u64 { break; }
                            
                            let absolute_inode = group * self.inodes_per_group as u64 + relative_inode + 1; // Inodes start at 1
                            if absolute_inode > self.inode_count { return Err(FsError::NoSpace); }
                            
                            let mut new_bitmap = bitmap.clone();
                            new_bitmap[byte_idx] |= 1 << bit_idx;
                            self.write_block(bitmap_block, &new_bitmap)?;
                            
                            // TODO: Update GDT/SB free counts
                            
                            return Ok(absolute_inode);
                        }
                    }
                }
            }
        }
        Err(FsError::NoSpace)
    }

    /// Libère un inode
    fn free_inode(&mut self, inode_num: u64) -> Result<(), FsError> {
        if inode_num == 0 || inode_num > self.inode_count { return Err(FsError::InvalidInode); }
        
        let group = (inode_num - 1) / self.inodes_per_group as u64;
        let index = (inode_num - 1) % self.inodes_per_group as u64;
        
        let bgd = self.read_bgd(group)?;
        let inode_bitmap_block = bgd.inode_bitmap as u64;
        
        let mut bitmap = vec![0u8; self.block_size as usize];
        self.read_block(inode_bitmap_block, &mut bitmap)?;
        
        let byte_idx = (index / 8) as usize;
        let bit_idx = (index % 8) as usize;
        
        bitmap[byte_idx] &= !(1 << bit_idx);
        self.write_block(inode_bitmap_block, &bitmap)?;
        
        Ok(())
    }

    /// Ajoute une entrée de répertoire
    fn add_directory_entry(&mut self, parent_inode: u64, child_inode: u64, name: &str, file_type: u8) -> Result<(), FsError> {
        let mut inode = self.read_inode(parent_inode)?;
        let entry_size = core::mem::size_of::<DirEntry>();
        
        // Trouver un slot libre
        for (i, &block_num) in inode.block.iter().enumerate().take(12) {
             let block_num = if block_num == 0 {
                 let new = self.allocate_block()?;
                 inode.block[i] = new as u32;
                 inode.blocks += 1;
                 // Write updated inode immediately to save block alloc
                 self.write_inode(parent_inode, &inode)?;
                 new
             } else {
                 block_num as u64
             };
             
             let mut buf = vec![0u8; self.block_size as usize];
             self.read_block(block_num, &mut buf)?;
             
             let entries_in_block = self.block_size as usize / entry_size;
             for j in 0..entries_in_block {
                 let start = j * entry_size;
                 let end = start + entry_size;
                 
                 // Check if empty (inode 0)
                 let ptr = buf[start..end].as_ptr() as *const DirEntry;
                 let entry = unsafe { ptr.read_unaligned() };
                 
                 if entry.inode == 0 {
                     // Found slot
                     let mut new_entry = DirEntry {
                         inode: child_inode as u32,
                         name_len: name.len() as u8,
                         file_type,
                         name: [0; 255],
                     };
                     new_entry.name[..name.len()].copy_from_slice(name.as_bytes());
                     
                     let entry_bytes = unsafe {
                        core::slice::from_raw_parts(
                            &new_entry as *const _ as *const u8,
                            entry_size,
                        )
                     };
                     buf[start..end].copy_from_slice(entry_bytes);
                     self.write_block(block_num, &buf)?;
                     return Ok(());
                 }
             }
        }
        
        Err(FsError::NoSpace)
    }

    /// Lit les entrées d'un répertoire
    fn read_directory_entries(&self, inode_num: u64) -> Result<Vec<DirEntry>, FsError> {
        let inode = self.read_inode(inode_num)?;
        let file_type = (inode.mode >> 12) as u8;
        if file_type != UFAT_FT_DIR {
             return Err(FsError::NotADirectory);
        }
        
        let mut entries = Vec::new();
        let entry_size = core::mem::size_of::<DirEntry>();
        
        for &block_num in inode.block.iter().take(12) {
             if block_num == 0 { break; }
             
             let mut buf = vec![0u8; self.block_size as usize];
             self.read_block(block_num as u64, &mut buf)?;
             
             let entries_in_block = self.block_size as usize / entry_size;
             for i in 0..entries_in_block {
                 let start = i * entry_size;
                 let end = start + entry_size;
                 let ptr = buf[start..end].as_ptr() as *const DirEntry;
                 let entry = unsafe { ptr.read_unaligned() };
                 
                 if entry.inode != 0 {
                     entries.push(entry);
                 }
             }
        }
        Ok(entries)
    }

    /// Résout un chemin vers un inode
    fn resolve_path(&self, path: &str) -> Result<u64, FsError> {
        if path == "/" || path.is_empty() { return Ok(1); } // Root inode
        
        let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        let mut current_inode = 1;
        
        for part in parts {
            let entries = self.read_directory_entries(current_inode)?;
            let mut found = false;
            
            for entry in entries {
                let len = entry.name_len as usize;
                if len > MAX_FILENAME_LENGTH { continue; }
                
                let name = &entry.name[..len];
                if name == part.as_bytes() {
                    current_inode = entry.inode as u64;
                    found = true;
                    break;
                }
            }
            
            if !found { return Err(FsError::NotFound); }
        }
        
        Ok(current_inode)
    }

    // Méthodes internes d'aide
    fn read_block(&self, block_num: u64, buf: &mut [u8]) -> Result<(), FsError> {
        let mut disk = self.disk.lock();
        let offset = block_num * self.block_size as u64;
        disk.read(offset, buf)
    }
    
    fn write_block(&self, block_num: u64, buf: &[u8]) -> Result<(), FsError> {
        let mut disk = self.disk.lock();
        let offset = block_num * self.block_size as u64;
        disk.write(offset, buf)
    }
}

// Implémentation du trait FileSystem pour UFAT
impl<D: Disk> crate::filesystem::FileSystem for UFAT<D> {
    fn read_dir(&self, path: &str) -> Result<Vec<String>, FsError> {
        let inode_num = self.resolve_path(path)?;
        let entries = self.read_directory_entries(inode_num)?;
        
        let mut names = Vec::new();
        for entry in entries {
            let len = entry.name_len as usize;
            if len <= MAX_FILENAME_LENGTH {
                if let Ok(s) = core::str::from_utf8(&entry.name[..len]) {
                    names.push(String::from(s));
                }
            }
        }
        Ok(names)
    }
    
    fn read_file(&self, path: &str) -> Result<Vec<u8>, FsError> {
        let inode_num = self.resolve_path(path)?;
        let inode = self.read_inode(inode_num)?;
        
        let file_type = (inode.mode >> 12) as u8;
        if file_type == UFAT_FT_DIR {
            return Err(FsError::NotAFile);
        }
        
        let mut content = Vec::new();
        let mut remaining = inode.size as usize;
        
        // Read direct blocks
        for &block_num in inode.block.iter().take(12) {
            if block_num == 0 || remaining == 0 { break; }
            
            let mut buf = vec![0u8; self.block_size as usize];
            self.read_block(block_num as u64, &mut buf)?;
            
            let to_read = remaining.min(self.block_size as usize);
            content.extend_from_slice(&buf[..to_read]);
            remaining -= to_read;
        }
        
        Ok(content)
    }
    
    fn write_file(&mut self, path: &str, content: &[u8]) -> Result<(), FsError> {
        let inode_num = self.resolve_path(path)?;
        let mut inode = self.read_inode(inode_num)?;
        
        let file_type = (inode.mode >> 12) as u8;
        if file_type == UFAT_FT_DIR {
            return Err(FsError::NotAFile);
        }
        
        let mut remaining = content.len();
        let mut offset = 0;
        let mut block_idx = 0;
        
        while remaining > 0 {
            if block_idx >= 12 {
                // TODO: Support indirect blocks
                return Err(FsError::NoSpace); // Limit for now
            }
            
            let block_num = if inode.block[block_idx] == 0 {
                // Allouer un nouveau bloc
                let new_block = self.allocate_block()?;
                inode.block[block_idx] = new_block as u32;
                inode.blocks += 1;
                new_block
            } else {
                inode.block[block_idx] as u64
            };
            
            let to_write = remaining.min(self.block_size as usize);
            let mut buf = vec![0u8; self.block_size as usize];
            
            // Copier les données
            buf[..to_write].copy_from_slice(&content[offset..offset + to_write]);
            self.write_block(block_num, &buf)?;
            
            remaining -= to_write;
            offset += to_write;
            block_idx += 1;
        }
        
        inode.size = content.len() as u64;
        self.write_inode(inode_num, &inode)?;
        
        Ok(())
    }
    
    fn create_file(&mut self, path: &str, content: &[u8]) -> Result<(), FsError> {
        if self.exists(path) { return Err(FsError::AlreadyExists); }
        
        let path_string = String::from(path);
        let parts: Vec<&str> = path_string.rsplitn(2, '/').collect();
        let (filename, parent_path) = if parts.len() == 2 {
            (parts[0], parts[1])
        } else {
            (parts[0], if path.starts_with('/') { "/" } else { "." })
        };
        let parent_path = if parent_path.is_empty() { "/" } else { parent_path };
        
        let parent_inode_num = self.resolve_path(parent_path)?;
        let new_inode_num = self.allocate_inode()?;
        
        let inode = UfatInode {
            mode: 0o644 | ((UFAT_FT_REG_FILE as u16) << 12),
            uid: 0,
            size: 0,
            atime: 0, ctime: 0, mtime: 0,
            blocks: 0, flags: 0,
            block: [0; 15], checksum: 0, reserved: [0; 16],
        };
        self.write_inode(new_inode_num, &inode)?;
        
        self.add_directory_entry(parent_inode_num, new_inode_num, filename, UFAT_FT_REG_FILE)?;
        
        self.write_file(path, content)
    }
    
    fn create_dir(&mut self, path: &str) -> Result<(), FsError> {
        if self.exists(path) { return Err(FsError::AlreadyExists); }
        
        let path_string = String::from(path);
        let parts: Vec<&str> = path_string.rsplitn(2, '/').collect();
        let (filename, parent_path) = if parts.len() == 2 {
            (parts[0], parts[1])
        } else {
            (parts[0], if path.starts_with('/') { "/" } else { "." })
        };
        let parent_path = if parent_path.is_empty() { "/" } else { parent_path };
        
        let parent_inode_num = self.resolve_path(parent_path)?;
        let new_inode_num = self.allocate_inode()?;
        
        let mut inode = UfatInode {
            mode: 0o755 | ((UFAT_FT_DIR as u16) << 12),
            uid: 0,
            size: self.block_size as u64,
            atime: 0, ctime: 0, mtime: 0,
            blocks: 1, flags: 0,
            block: [0; 15], checksum: 0, reserved: [0; 16],
        };
        
        let block_num = self.allocate_block()?;
        inode.block[0] = block_num as u32;
        self.write_inode(new_inode_num, &inode)?;
        
        // Init directory with . and ..
        let dot = DirEntry {
             inode: new_inode_num as u32,
             name_len: 1, file_type: UFAT_FT_DIR,
             name: { let mut n = [0; 255]; n[0] = b'.'; n },
        };
        let dotdot = DirEntry {
             inode: parent_inode_num as u32,
             name_len: 2, file_type: UFAT_FT_DIR,
             name: { let mut n = [0; 255]; n[0] = b'.'; n[1] = b'.'; n },
        };
        
        let mut buf = vec![0u8; self.block_size as usize];
        
        // Manual copy to avoid unsafe ptr arithmetic if possible, or just be careful
        let entry_size = core::mem::size_of::<DirEntry>();
         unsafe {
            let ptr = buf.as_mut_ptr() as *mut DirEntry;
            *ptr.add(0) = dot;
            *ptr.add(1) = dotdot;
        }
        self.write_block(block_num, &buf)?;
        
        self.add_directory_entry(parent_inode_num, new_inode_num, filename, UFAT_FT_DIR)?;
        
        Ok(())
    }
    
    fn remove_file(&mut self, path: &str) -> Result<(), FsError> {
        // TODO: Implémenter la suppression de fichier
        Err(FsError::IOError)
    }
    
    fn remove_dir(&mut self, path: &str) -> Result<(), FsError> {
        // TODO: Implémenter la suppression de répertoire
        Err(FsError::IOError)
    }
    
    fn exists(&self, path: &str) -> bool {
        self.resolve_path(path).is_ok()
    }
    
    fn is_file(&self, path: &str) -> bool {
        if let Ok(inode_num) = self.resolve_path(path) {
            if let Ok(inode) = self.read_inode(inode_num) {
                return ((inode.mode >> 12) as u8) == UFAT_FT_REG_FILE;
            }
        }
        false
    }
    
    fn is_dir(&self, path: &str) -> bool {
        if let Ok(inode_num) = self.resolve_path(path) {
            if let Ok(inode) = self.read_inode(inode_num) {
                return ((inode.mode >> 12) as u8) == UFAT_FT_DIR;
            }
        }
        false
    }
}

// Fonction utilitaire pour monter une partition UFAT
pub fn mount_ufat<D: Disk>(disk: D) -> Result<UFAT<D>, FsError> {
    UFAT::new(disk)
}

// Fonction utilitaire pour formater une partition en UFAT
pub fn format_ufat<D: Disk>(disk: D, volume_name: &str) -> Result<(), FsError> {
    UFAT::format(disk, volume_name)
}
