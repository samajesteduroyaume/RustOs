use core::fmt;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::collections::BTreeMap;
use crate::filesystem::FsError;
use crate::disk::Disk;

// Constantes pour EXT2
const EXT2_SIGNATURE: u16 = 0xEF53; // Signature EXT2
const EXT2_ROOT_INO: u32 = 2;      // Inode de la racine
const EXT2_S_IFREG: u16 = 0x8000;  // Fichier régulier
const EXT2_S_IFDIR: u16 = 0x4000;  // Répertoire
const EXT2_S_IFLNK: u16 = 0xA000;  // Lien symbolique

// Taille des blocs (peut être 1024, 2048 ou 4096 octets)
const BLOCK_SIZE: usize = 4096;

// Structure du Superbloc EXT2
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct SuperBlock {
    pub inodes_count: u32,          // Nombre total d'inodes
    pub blocks_count: u32,          // Nombre total de blocs
    pub r_blocks_count: u32,        // Nombre de blocs réservés
    pub free_blocks_count: u32,     // Nombre de blocs libres
    pub free_inodes_count: u32,     // Nombre d'inodes libres
    pub first_data_block: u32,      // Premier bloc de données
    pub log_block_size: u32,        // log2(block size) - 10
    pub log_frag_size: u32,         // log2(fragment size) - 10
    pub blocks_per_group: u32,      // Blocs par groupe
    pub frags_per_group: u32,       // Fragments par groupe
    pub inodes_per_group: u32,      // Inodes par groupe
    pub mtime: u32,                 // Dernier montage
    pub wtime: u32,                 // Dernière écriture
    pub mnt_count: u16,             // Nombre de montages depuis le dernier fsck
    pub max_mnt_count: u16,         // Nombre max de montages avant fsck
    pub magic: u16,                 // Signature (0xEF53)
    pub state: u16,                 // État du système de fichiers
    pub errors: u16,                // Comportement en cas d'erreur
    pub minor_rev_level: u16,       // Version mineure
    pub lastcheck: u32,             // Dernière vérification
    pub checkinterval: u32,         // Intervalle maximal entre vérifications
    pub creator_os: u32,            // OS ayant créé le système de fichiers
    pub rev_level: u32,             // Niveau de révision
    pub def_resuid: u16,            // UID par défaut pour les blocs réservés
    pub def_resgid: u16,            // GID par défaut pour les blocs réservés
    // ... autres champs omis pour la brièveté
}

// En-tête de groupe de blocs
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct BlockGroupDescriptor {
    pub block_bitmap: u32,          // Bloc contenant le bitmap de blocs
    pub inode_bitmap: u32,          // Bloc contenant le bitmap d'inodes
    pub inode_table: u32,           // Premier bloc de la table d'inodes
    pub free_blocks_count: u16,     // Nombre de blocs libres dans le groupe
    pub free_inodes_count: u16,     // Nombre d'inodes libres dans le groupe
    pub used_dirs_count: u16,       // Nombre de répertoires dans le groupe
    pub pad: u16,                   // Remplissage
    pub reserved: [u32; 3],         // Réservé pour l'avenir
}

// Structure d'un inode EXT2
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct Inode {
    pub mode: u16,                  // Type et permissions
    pub uid: u16,                   // UID du propriétaire
    pub size: u32,                  // Taille en octets
    pub atime: u32,                 // Dernier accès
    pub ctime: u32,                 // Création
    pub mtime: u32,                 // Dernière modification
    pub dtime: u32,                 // Heure de suppression
    pub gid: u16,                   // GID du propriétaire
    pub links_count: u16,           // Nombre de liens
    pub blocks: u32,                // Nombre de blocs de 512 octets
    pub flags: u32,                 // Drapeaux
    pub osd1: u32,                  // OS dépendant 1
    pub block: [u32; 15],           // Pointeurs vers les blocs de données
    pub generation: u32,            // Numéro de génération (NFS)
    pub file_acl: u32,              // Bloc ACL
    pub dir_acl: u32,               // Taille du fichier (si taille > 4 Go)
    pub faddr: u32,                 // Fragment address
    pub osd2: [u8; 12],             // OS dépendant 2
}

// Entrée de répertoire
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct DirEntry {
    pub inode: u32,                 // Numéro d'inode
    pub rec_len: u16,               // Longueur de l'entrée
    pub name_len: u8,               // Longueur du nom
    pub file_type: u8,              // Type de fichier
    pub name: [u8; 255],            // Nom du fichier (taille maximale)
}

// Structure principale du système de fichiers EXT2
pub struct Ext2<D: Disk> {
    disk: D,
    block_size: usize,
    inodes_per_group: u32,
    blocks_per_group: u32,
    inode_size: u16,
    block_group_count: u32,
    first_data_block: u32,
    superblock: SuperBlock,
    block_groups: Vec<BlockGroupDescriptor>,
}

// Erreurs spécifiques à EXT2
#[derive(Debug)]
pub enum Ext2Error {
    InvalidSignature,
    InvalidSuperblock,
    DiskError,
    InodeNotFound,
    BlockGroupNotFound,
    InvalidPath,
    NotADirectory,
    NotAFile,
    AlreadyExists,
    NoSpaceLeft,
    IoError,
}

impl From<Ext2Error> for FsError {
    fn from(err: Ext2Error) -> Self {
        match err {
            Ext2Error::InvalidSignature => FsError::IOError,
            Ext2Error::InvalidSuperblock => FsError::IOError,
            Ext2Error::DiskError => FsError::IOError,
            Ext2Error::InodeNotFound => FsError::NotFound,
            Ext2Error::BlockGroupNotFound => FsError::IOError,
            Ext2Error::InvalidPath => FsError::InvalidPath,
            Ext2Error::NotADirectory => FsError::NotADirectory,
            Ext2Error::NotAFile => FsError::IOError,
            Ext2Error::AlreadyExists => FsError::AlreadyExists,
            Ext2Error::NoSpaceLeft => FsError::IOError,
            Ext2Error::IoError => FsError::IOError,
        }
    }
}

impl<D: Disk> Ext2<D> {
    // Crée une nouvelle instance de EXT2 à partir d'un périphérique de disque
    pub fn new(disk: D) -> Result<Self, Ext2Error> {
        let mut superblock_buf = [0u8; 1024]; // Le superbloc fait 1024 octets
        disk.read(1024, &mut superblock_buf)?; // Le superbloc est à 1024 octets de décalage
        
        let superblock = unsafe { &*(superblock_buf.as_ptr() as *const SuperBlock) };
        
        // Vérifier la signature magique
        if superblock.magic != EXT2_SIGNATURE {
            return Err(Ext2Error::InvalidSignature);
        }
        
        // Calculer la taille des blocs
        let block_size = 1024 << superblock.log_block_size as usize;
        
        // Lire les descripteurs de groupe de blocs
        let block_group_count = (superblock.blocks_count + superblock.blocks_per_group - 1) / superblock.blocks_per_group;
        let bgdt_blocks = ((block_group_count as usize * 32) + block_size - 1) / block_size;
        let mut bgdt = Vec::with_capacity(block_group_count as usize);
        
        let bgdt_start = if block_size > 1024 { 2 } else { 1 };
        
        for i in 0..block_group_count {
            let mut bgd_buf = [0u8; 32]; // Chaque descripteur fait 32 octets
            let offset = (bgdt_start + (i as usize * 32) / block_size) * block_size;
            let bgd_offset = (i as usize * 32) % block_size;
            
            disk.read(offset as u64, &mut bgd_buf)?;
            let bgd = unsafe { &*(bgd_buf.as_ptr() as *const BlockGroupDescriptor) };
            bgdt.push(*bgd);
        }
        
        Ok(Self {
            disk,
            block_size,
            inodes_per_group: superblock.inodes_per_group,
            blocks_per_group: superblock.blocks_per_group,
            inode_size: if superblock.rev_level >= 1 { superblock.inode_size } else { 128 },
            block_group_count,
            first_data_block: superblock.first_data_block,
            superblock: *superblock,
            block_groups: bgdt,
        })
    }
    
    // Lit un bloc du disque
    fn read_block(&self, block_num: u32, buf: &mut [u8]) -> Result<(), Ext2Error> {
        let offset = (block_num as u64) * (self.block_size as u64);
        self.disk.read(offset, buf).map_err(|_| Ext2Error::DiskError)
    }
    
    // Écrit un bloc sur le disque
    fn write_block(&mut self, block_num: u32, buf: &[u8]) -> Result<(), Ext2Error> {
        let offset = (block_num as u64) * (self.block_size as u64);
        self.disk.write(offset, buf).map_err(|_| Ext2Error::DiskError)
    }
    
    // Alloue un nouveau bloc
    fn allocate_block(&mut self) -> Result<u32, Ext2Error> {
        // Parcourir les groupes de blocs pour trouver un bloc libre
        for (group_idx, bg) in self.block_groups.iter_mut().enumerate() {
            if bg.free_blocks_count == 0 {
                continue;
            }
            
            // Lire le bitmap de blocs
            let mut bitmap = vec![0u8; self.block_size];
            self.read_block(bg.block_bitmap, &mut bitmap)?;
            
            // Trouver un bit à 0 dans le bitmap
            for byte_idx in 0..bitmap.len() {
                if bitmap[byte_idx] != 0xFF { // Pas tous les bits sont à 1
                    for bit in 0..8 {
                        if (bitmap[byte_idx] & (1 << bit)) == 0 {
                            // Marquer le bloc comme utilisé
                            bitmap[byte_idx] |= 1 << bit;
                            
                            // Calculer le numéro de bloc global
                            let block_in_group = (byte_idx * 8 + bit) as u32;
                            let block_num = (group_idx as u32 * self.blocks_per_group) + block_in_group;
                            
                            // Mettre à jour le bitmap sur le disque
                            self.write_block(bg.block_bitmap, &bitmap)?;
                            
                            // Mettre à jour les compteurs
                            bg.free_blocks_count -= 1;
                            self.superblock.free_blocks_count -= 1;
                            
                            // Mettre à jour le superbloc sur le disque
                            let superblock_buf = unsafe {
                                core::slice::from_raw_parts(
                                    &self.superblock as *const _ as *const u8,
                                    core::mem::size_of::<SuperBlock>()
                                )
                            };
                            self.write_block(1, superblock_buf)?; // Le superbloc est à l'offset 1 si block_size > 1024
                            
                            return Ok(block_num);
                        }
                    }
                }
            }
        }
        
        Err(Ext2Error::NoSpaceLeft)
    }
    
    // Met à jour un inode sur le disque
    fn update_inode(&mut self, inode_num: u32, inode: &Inode) -> Result<(), Ext2Error> {
        if inode_num == 0 || inode_num > self.superblock.inodes_count {
            return Err(Ext2Error::InodeNotFound);
        }
        
        let inode_idx = inode_num - 1;
        let group_idx = inode_idx / self.inodes_per_group;
        let inode_in_group = inode_idx % self.inodes_per_group;
        
        if group_idx >= self.block_group_count {
            return Err(Ext2Error::BlockGroupNotFound);
        }
        
        let bg = &self.block_groups[group_idx as usize];
        let inode_table_block = bg.inode_table;
        let inode_offset = (inode_in_group as usize) * (self.inode_size as usize);
        let block_offset = inode_offset / self.block_size;
        let in_block_offset = inode_offset % self.block_size;
        
        // Lire le bloc contenant l'inode
        let mut block_buf = vec![0u8; self.block_size];
        self.read_block(inode_table_block + block_offset as u32, &mut block_buf)?;
        
        // Mettre à jour l'inode dans le buffer
        let inode_ptr = &mut block_buf[in_block_offset..in_block_offset + 128];
        unsafe {
            core::ptr::copy_nonoverlapping(
                inode as *const _ as *const u8,
                inode_ptr.as_mut_ptr(),
                core::mem::size_of::<Inode>(),
            );
        }
        
        // Écrire le bloc mis à jour
        self.write_block(inode_table_block + block_offset as u32, &block_buf)?;
        
        Ok(())
    }
    
    // Écrit des données dans un inode, allouant des blocs si nécessaire
    fn write_inode_data(&mut self, inode: &mut Inode, offset: usize, data: &[u8]) -> Result<usize, Ext2Error> {
        let mut remaining = data.len();
        let mut total_written = 0;
        
        while remaining > 0 {
            let block_idx = (offset + total_written) / self.block_size;
            let block_offset = (offset + total_written) % self.block_size;
            let to_write = remaining.min(self.block_size - block_offset);
            
            // Obtenir le numéro de bloc, en allouant un nouveau bloc si nécessaire
            let block_num = if block_idx < 12 {
                // Bloc direct
                if inode.block[block_idx] == 0 {
                    let new_block = self.allocate_block()?;
                    inode.block[block_idx] = new_block;
                    new_block
                } else {
                    inode.block[block_idx]
                }
            } else {
                // Pour simplifier, on ne gère pas les blocs indirects ici
                return Ok(total_written);
            };
            
            // Lire le bloc existant ou créer un nouveau
            let mut block_buf = vec![0u8; self.block_size];
            if block_offset > 0 || to_write < self.block_size {
                self.read_block(block_num, &mut block_buf)?;
            }
            
            // Copier les données dans le buffer du bloc
            let start = total_written;
            let end = start + to_write;
            block_buf[block_offset..block_offset + to_write].copy_from_slice(&data[start..end]);
            
            // Écrire le bloc mis à jour
            self.write_block(block_num, &block_buf)?;
            
            total_written += to_write;
            remaining -= to_write;
        }
        
        // Mettre à jour la taille du fichier si nécessaire
        let new_size = (offset + total_written) as u32;
        if new_size > inode.size {
            inode.size = new_size;
        }
        
        Ok(total_written)
    }
    
    // Ajoute une entrée à un répertoire
    fn add_dir_entry(&mut self, dir_inode: &mut Inode, name: &str, inode_num: u32, file_type: u8) -> Result<(), Ext2Error> {
        let name_bytes = name.as_bytes();
        let name_len = name_bytes.len();
        
        if name_len > 255 {
            return Err(Ext2Error::InvalidPath);
        }
        
        // Calculer la taille nécessaire pour la nouvelle entrée
        let entry_len = 8 + ((name_len + 3) & !3); // Alignement sur 4 octets
        
        // Parcourir le répertoire pour trouver un emplacement libre
        let mut offset = 0;
        let mut buf = vec![0u8; self.block_size];
        let mut found_space = false;
        let mut space_pos = 0;
        let mut space_size = 0;
        
        loop {
            let read = self.read_inode_data(dir_inode, offset, &mut buf)?;
            if read == 0 {
                break;
            }
            
            let mut pos = 0;
            while pos + 8 <= read {
                let entry = unsafe { &*(&buf[pos..] as *const _ as *const DirEntry) };
                let entry_len = entry.rec_len as usize;
                
                if entry.inode == 0 && entry.rec_len as usize >= entry_len {
                    // Espace libre trouvé
                    found_space = true;
                    space_pos = offset + pos;
                    space_size = entry.rec_len as usize;
                    break;
                }
                
                if entry_len == 0 || pos + entry_len > read {
                    break;
                }
                
                pos += entry_len;
            }
            
            if found_space {
                break;
            }
            
            offset += read;
        }
        
        // Si aucun espace libre n'a été trouvé, agrandir le répertoire
        if !found_space {
            // Pour simplifier, on alloue un nouveau bloc
            let block_num = self.allocate_block()?;
            let block_idx = (offset + self.block_size - 1) / self.block_size;
            
            if block_idx < 12 {
                inode.block[block_idx] = block_num;
            } else {
                // Gérer les blocs indirects (non implémenté ici)
                return Err(Ext2Error::NoSpaceLeft);
            }
            
            // Initialiser le nouveau bloc avec une entrée vide
            let mut new_block = vec![0u8; self.block_size];
            let new_entry = DirEntry {
                inode: 0,
                rec_len: self.block_size as u16,
                name_len: 0,
                file_type: 0,
                name: [0; 255],
            };
            
            // Copier la structure dans le buffer
            let new_entry_slice = unsafe {
                core::slice::from_raw_parts(
                    &new_entry as *const _ as *const u8,
                    core::mem::size_of::<DirEntry>()
                )
            };
            new_block[..new_entry_slice.len()].copy_from_slice(new_entry_slice);
            
            // Écrire le nouveau bloc
            self.write_block(block_num, &new_block)?;
            
            space_pos = offset;
            space_size = self.block_size;
        }
        
        // Lire le bloc contenant l'espace libre
        let block_offset = space_pos % self.block_size;
        let block_num = {
            let block_idx = space_pos / self.block_size;
            if block_idx < 12 {
                inode.block[block_idx]
            } else {
                // Gérer les blocs indirects (non implémenté ici)
                return Err(Ext2Error::NoSpaceLeft);
            }
        };
        
        let mut block_buf = vec![0u8; self.block_size];
        self.read_block(block_num, &mut block_buf)?;
        
        // Créer la nouvelle entrée
        let mut new_entry = DirEntry {
            inode: inode_num,
            rec_len: entry_len as u16,
            name_len: name_len as u8,
            file_type,
            name: [0; 255],
        };
        
        // Copier le nom
        new_entry.name[..name_len].copy_from_slice(name_bytes);
        
        // Mettre à jour la longueur de l'entrée précédente si nécessaire
        if space_size > entry_len {
            let remaining_space = space_size - entry_len;
            let prev_entry = unsafe { &mut *(&mut block_buf[block_offset] as *mut _ as *mut DirEntry) };
            prev_entry.rec_len = remaining_space as u16;
            
            // Déplacer le pointeur vers la nouvelle entrée
            let new_entry_pos = block_offset + remaining_space;
            
            // Copier la nouvelle entrée
            let new_entry_slice = unsafe {
                core::slice::from_raw_parts(
                    &new_entry as *const _ as *const u8,
                    core::mem::size_of::<DirEntry>()
                )
            };
            
            block_buf[new_entry_pos..new_entry_pos + entry_len].copy_from_slice(&new_entry_slice[..entry_len]);
        } else {
            // Pas assez d'espace pour diviser, écraser l'entrée vide
            let new_entry_slice = unsafe {
                core::slice::from_raw_parts(
                    &new_entry as *const _ as *const u8,
                    core::mem::size_of::<DirEntry>()
                )
            };
            
            block_buf[block_offset..block_offset + entry_len].copy_from_slice(&new_entry_slice[..entry_len]);
        }
        
        // Écrire le bloc mis à jour
        self.write_block(block_num, &block_buf)?;
        
        // Mettre à jour le nombre de liens du répertoire parent
        if inode_num != EXT2_ROOT_INO {
            dir_inode.links_count += 1;
        }
        
        Ok(())
    }
    
    // Trouve un inode par son numéro
    fn get_inode(&self, inode_num: u32) -> Result<Inode, Ext2Error> {
        if inode_num == 0 || inode_num > self.superblock.inodes_count {
            return Err(Ext2Error::InodeNotFound);
        }
        
        let inode_idx = inode_num - 1;
        let group_idx = inode_idx / self.inodes_per_group;
        let inode_in_group = inode_idx % self.inodes_per_group;
        
        if group_idx >= self.block_group_count {
            return Err(Ext2Error::BlockGroupNotFound);
        }
        
        let bg = &self.block_groups[group_idx as usize];
        let inode_table_block = bg.inode_table;
        let inode_offset = (inode_in_group as usize) * (self.inode_size as usize);
        let block_offset = inode_offset / self.block_size;
        let in_block_offset = inode_offset % self.block_size;
        
        let mut block_buf = vec![0u8; self.block_size];
        self.read_block(inode_table_block + block_offset as u32, &mut block_buf)?;
        
        // Extraire l'inode du bloc
        let inode_ptr = &block_buf[in_block_offset..in_block_offset + 128]; // Taille minimale d'un inode
        let inode = unsafe { &*(inode_ptr.as_ptr() as *const Inode) };
        
        Ok(*inode)
    }
    
    // Lit les données d'un inode
    fn read_inode_data(&self, inode: &Inode, offset: usize, buf: &mut [u8]) -> Result<usize, Ext2Error> {
        let file_size = inode.size as usize;
        let mut remaining = buf.len().min(file_size.saturating_sub(offset));
        let mut total_read = 0;
        
        while remaining > 0 && offset + total_read < file_size {
            let block_idx = (offset + total_read) / self.block_size;
            let block_offset = (offset + total_read) % self.block_size;
            let to_read = remaining.min(self.block_size - block_offset);
            
            // Obtenir le numéro de bloc physique
            let block_num = self.get_block_number(inode, block_idx as u32)?;
            
            // Lire le bloc
            let mut block_buf = vec![0u8; self.block_size];
            self.read_block(block_num, &mut block_buf)?;
            
            // Copier les données dans le buffer de sortie
            let start = block_offset;
            let end = start + to_read;
            buf[total_read..total_read + to_read].copy_from_slice(&block_buf[start..end]);
            
            total_read += to_read;
            remaining -= to_read;
        }
        
        Ok(total_read)
    }
    
    // Obtient le numéro de bloc physique pour un inode et un index de bloc logique
    fn get_block_number(&self, inode: &Inode, logical_block: u32) -> Result<u32, Ext2Error> {
        // Implémentation simplifiée - ne gère que les blocs directs
        if logical_block < 12 {
            // Blocs directs
            Ok(inode.block[logical_block as usize])
        } else {
            // Pour une implémentation complète, il faudrait gérer les blocs indirects
            // simples, doubles et triples. Ici, on retourne une erreur.
            Err(Ext2Error::IoError)
        }
    }
    
    // Trouve une entrée dans un répertoire
    fn find_entry_in_dir(&self, dir_inode: &Inode, name: &str) -> Result<DirEntry, Ext2Error> {
        let mut offset = 0;
        let mut buf = vec![0u8; self.block_size];
        
        loop {
            let read = self.read_inode_data(dir_inode, offset, &mut buf)?;
            if read == 0 {
                break;
            }
            
            let mut pos = 0;
            while pos + 8 <= read {
                let entry = unsafe { &*(&buf[pos..] as *const _ as *const DirEntry) };
                let entry_len = entry.rec_len as usize;
                
                if entry.inode != 0 {
                    let entry_name = String::from_utf8_lossy(&entry.name[..entry.name_len as usize]);
                    if entry_name == name {
                        return Ok(*entry);
                    }
                }
                
                if entry_len == 0 || pos + entry_len > read {
                    break;
                }
                
                pos += entry_len;
                offset += entry_len;
            }
        }
        
        Err(Ext2Error::InodeNotFound)
    }
}

// Implémentation du trait FileSystem pour EXT2
impl<D: Disk> crate::filesystem::FileSystem for Ext2<D> {
    fn read_dir(&self, path: &str) -> Result<Vec<String>, FsError> {
        let inode = if path.is_empty() || path == "/" {
            self.get_inode(EXT2_ROOT_INO)
        } else {
            let dir_inode = self.get_inode(EXT2_ROOT_INO)?;
            let entry = self.find_entry_in_dir(&dir_inode, path.trim_start_matches('/'))?;
            self.get_inode(entry.inode)
        }.map_err(Ext2Error::into)?;
        
        if (inode.mode & EXT2_S_IFDIR) == 0 {
            return Err(FsError::NotADirectory);
        }
        
        let mut entries = Vec::new();
        let mut offset = 0;
        let mut buf = vec![0u8; self.block_size];
        
        loop {
            let read = self.read_inode_data(&inode, offset, &mut buf)
                .map_err(Ext2Error::into)?;
                
            if read == 0 {
                break;
            }
            
            let mut pos = 0;
            while pos + 8 <= read {
                let entry = unsafe { &*(&buf[pos..] as *const _ as *const DirEntry) };
                let entry_len = entry.rec_len as usize;
                
                if entry.inode != 0 && entry.name_len > 0 {
                    let name = String::from_utf8_lossy(&entry.name[..entry.name_len as usize]);
                    if name != "." && name != ".." {
                        entries.push(name.into_owned());
                    }
                }
                
                if entry_len == 0 || pos + entry_len > read {
                    break;
                }
                
                pos += entry_len;
                offset += entry_len;
            }
        }
        
        Ok(entries)
    }
    
    fn read_file(&self, path: &str) -> Result<Vec<u8>, FsError> {
        let inode = if path.is_empty() || path == "/" {
            return Err(FsError::NotAFile);
        } else {
            let dir_inode = self.get_inode(EXT2_ROOT_INO).map_err(Ext2Error::into)?;
            let entry = self.find_entry_in_dir(&dir_inode, path.trim_start_matches('/'))
                .map_err(Ext2Error::into)?;
            self.get_inode(entry.inode).map_err(Ext2Error::into)?
        };
        
        if (inode.mode & EXT2_S_IFREG) == 0 {
            return Err(FsError::NotAFile);
        }
        
        let mut data = vec![0u8; inode.size as usize];
        self.read_inode_data(&inode, 0, &mut data)
            .map_err(Ext2Error::into)?;
            
        Ok(data)
    }
    
    // Les méthodes suivantes sont des implémentations de base qui retournent des erreurs
    // car l'écriture nécessite une implémentation plus complexe avec mise à jour des bitmaps, etc.
    
    fn write_file(&mut self, path: &str, content: &[u8]) -> Result<(), FsError> {
        if path.is_empty() || path == "/" {
            return Err(FsError::IOError);
        }
        
        let path = path.trim_start_matches('/');
        let (dir_path, file_name) = match path.rfind('/') {
            Some(pos) => path.split_at(pos),
            None => ("", path),
        };
        
        // Trouver le répertoire parent
        let dir_inode_num = if dir_path.is_empty() {
            EXT2_ROOT_INO
        } else {
            let mut current_inode = self.get_inode(EXT2_ROOT_INO)?;
            for component in dir_path.split('/') {
                if component.is_empty() {
                    continue;
                }
                
                let entry = self.find_entry_in_dir(&current_inode, component)?;
                current_inode = self.get_inode(entry.inode)?;
                
                if (current_inode.mode & EXT2_S_IFDIR) == 0 {
                    return Err(FsError::NotADirectory);
                }
            }
            
            // Obtenir le numéro d'inode du répertoire parent
            let entry = self.find_entry_in_dir(&current_inode, ".")?;
            entry.inode
        };
        
        let mut dir_inode = self.get_inode(dir_inode_num)?;
        
        // Vérifier si le fichier existe déjà
        let existing_entry = self.find_entry_in_dir(&dir_inode, file_name);
        let mut inode_num = match existing_entry {
            Ok(entry) => {
                // Le fichier existe, on le tronque
                let inode = self.get_inode(entry.inode)?;
                // Libérer les blocs existants (simplifié)
                for &block in &inode.block {
                    if block != 0 {
                        // Marquer le bloc comme libre dans le bitmap
                        // (implémentation simplifiée)
                    }
                }
                entry.inode
            },
            Err(Ext2Error::InodeNotFound) => {
                // Créer un nouvel inode
                let inode_num = self.allocate_inode()?;
                
                // Créer une entrée de répertoire
                let file_type = EXT2_S_IFREG >> 12; // Type de fichier régulier
                self.add_dir_entry(&mut dir_inode, file_name, inode_num, file_type as u8)?;
                
                inode_num
            },
            Err(e) => return Err(e.into()),
        };
        
        // Mettre à jour l'inode du fichier
        let mut inode = self.get_inode(inode_num)?;
        inode.size = content.len() as u32;
        inode.mtime = 0; // Devrait être le timestamp actuel
        inode.ctime = 0;
        inode.atime = 0;
        inode.mode = 0o644 | EXT2_S_IFREG as u16; // Permissions 644 pour les fichiers réguliers
        
        // Écrire les données du fichier
        self.write_inode_data(&mut inode, 0, content)?;
        
        // Mettre à jour l'inode sur le disque
        self.update_inode(inode_num, &inode)?;
        
        // Mettre à jour l'inode du répertoire parent
        self.update_inode(dir_inode_num, &dir_inode)?;
        
        Ok(())
    }
    
    fn create_file(&mut self, path: &str, content: &[u8]) -> Result<(), FsError> {
        if self.exists(path) {
            return Err(FsError::AlreadyExists);
        }
        
        self.write_file(path, content)
    }
    
    fn create_dir(&mut self, path: &str) -> Result<(), FsError> {
        if path.is_empty() || path == "/" {
            return Err(FsError::AlreadyExists);
        }
        
        if self.exists(path) {
            return Err(FsError::AlreadyExists);
        }
        
        let path = path.trim_start_matches('/');
        let (dir_path, dir_name) = match path.rfind('/') {
            Some(pos) => path.split_at(pos),
            None => ("", path),
        };
        
        // Trouver le répertoire parent
        let parent_inode = if dir_path.is_empty() {
            self.get_inode(EXT2_ROOT_INO)?
        } else {
            let mut current_inode = self.get_inode(EXT2_ROOT_INO)?;
            for component in dir_path.split('/') {
                if component.is_empty() {
                    continue;
                }
                
                let entry = self.find_entry_in_dir(&current_inode, component)?;
                current_inode = self.get_inode(entry.inode)?;
                
                if (current_inode.mode & EXT2_S_IFDIR) == 0 {
                    return Err(FsError::NotADirectory);
                }
            }
            current_inode
        };
        
        // Allouer un nouvel inode pour le répertoire
        let new_inode_num = self.allocate_inode()?;
        let mut new_inode = Inode {
            mode: 0o755 | EXT2_S_IFDIR as u16, // Permissions 755 pour les répertoires
            uid: 0, // root
            size: 0,
            atime: 0,
            ctime: 0,
            mtime: 0,
            dtime: 0,
            gid: 0, // root
            links_count: 2, // . et ..
            blocks: 0,
            flags: 0,
            osd1: 0,
            block: [0; 15],
            generation: 0,
            file_acl: 0,
            dir_acl: 0,
            faddr: 0,
            osd2: [0; 12],
        };
        
        // Créer les entrées . et ..
        let dot_entry = DirEntry {
            inode: new_inode_num,
            rec_len: 12,
            name_len: 1,
            file_type: (EXT2_S_IFDIR >> 12) as u8,
            name: {
                let mut name = [0; 255];
                name[0] = b'.';
                name
            },
        };
        
        let dotdot_entry = DirEntry {
            inode: if dir_path.is_empty() { new_inode_num } else { 
                // Trouver le numéro d'inode du parent
                let entry = self.find_entry_in_dir(&parent_inode, ".")?;
                entry.inode
            },
            rec_len: (self.block_size - 12) as u16, // Le reste du bloc
            name_len: 2,
            file_type: (EXT2_S_IFDIR >> 12) as u8,
            name: {
                let mut name = [0; 255];
                name[0] = b'.';
                name[1] = b'.';
                name
            },
        };
        
        // Allouer un bloc pour le répertoire
        let block_num = self.allocate_block()?;
        new_inode.block[0] = block_num;
        
        // Écrire les entrées . et .. dans le bloc
        let mut block_buf = vec![0u8; self.block_size];
        
        // Copier l'entrée .
        let dot_slice = unsafe {
            core::slice::from_raw_parts(
                &dot_entry as *const _ as *const u8,
                12 // Taille fixe pour .
            )
        };
        block_buf[..12].copy_from_slice(dot_slice);
        
        // Copier l'entrée ..
        let dotdot_slice = unsafe {
            core::slice::from_raw_parts(
                &dotdot_entry as *const _ as *const u8,
                12 // Taille fixe pour ..
            )
        };
        block_buf[12..24].copy_from_slice(dotdot_slice);
        
        // Écrire le bloc
        self.write_block(block_num, &block_buf)?;
        
        // Mettre à jour la taille du répertoire
        new_inode.size = self.block_size as u32;
        
        // Écrire l'inode
        self.update_inode(new_inode_num, &new_inode)?;
        
        // Ajouter l'entrée dans le répertoire parent
        let file_type = (EXT2_S_IFDIR >> 12) as u8;
        self.add_dir_entry(&mut parent_inode.clone(), dir_name, new_inode_num, file_type)?;
        
        // Mettre à jour le nombre de liens du répertoire parent
        if !dir_path.is_empty() {
            let mut parent_inode = self.get_inode(parent_inode.block[0])?; // Simplification
            parent_inode.links_count += 1;
            self.update_inode(parent_inode.block[0], &parent_inode)?; // Simplification
        }
        
        Ok(())
    }
    
    fn remove_file(&mut self, _path: &str) -> Result<(), FsError> {
        // Implémentation simplifiée - retourne une erreur
        Err(FsError::IOError)
    }
    
    fn remove_dir(&mut self, _path: &str) -> Result<(), FsError> {
        // Implémentation simplifiée - retourne une erreur
        Err(FsError::IOError)
    }
    
    fn exists(&self, path: &str) -> bool {
        if path.is_empty() || path == "/" {
            return true;
        }
        
        match self.get_inode(EXT2_ROOT_INO) {
            Ok(dir_inode) => {
                match self.find_entry_in_dir(&dir_inode, path.trim_start_matches('/')) {
                    Ok(_) => true,
                    Err(_) => false,
                }
            },
            Err(_) => false,
        }
    }
    
    fn is_file(&self, path: &str) -> bool {
        if path.is_empty() || path == "/" {
            return false;
        }
        
        match self.get_inode(EXT2_ROOT_INO) {
            Ok(dir_inode) => {
                match self.find_entry_in_dir(&dir_inode, path.trim_start_matches('/')) {
                    Ok(entry) => {
                        match self.get_inode(entry.inode) {
                            Ok(inode) => (inode.mode & EXT2_S_IFREG) != 0,
                            Err(_) => false,
                        }
                    },
                    Err(_) => false,
                }
            },
            Err(_) => false,
        }
    }
    
    fn is_dir(&self, path: &str) -> bool {
        if path.is_empty() || path == "/" {
            return true;
        }
        
        match self.get_inode(EXT2_ROOT_INO) {
            Ok(dir_inode) => {
                match self.find_entry_in_dir(&dir_inode, path.trim_start_matches('/')) {
                    Ok(entry) => {
                        match self.get_inode(entry.inode) {
                            Ok(inode) => (inode.mode & EXT2_S_IFDIR) != 0,
                            Err(_) => false,
                        }
                    },
                    Err(_) => false,
                }
            },
            Err(_) => false,
        }
    }
}

// Fonction utilitaire pour monter une partition EXT2
pub fn mount_ext2<D: Disk>(disk: D) -> Result<Ext2<D>, FsError> {
    Ext2::new(disk).map_err(Ext2Error::into)
}
