/// Module de mémoire partagée (Shared Memory - SHM)
/// 
/// Implémente la mémoire partagée POSIX pour la communication inter-processus (IPC).
/// Permet à plusieurs processus de partager des segments de mémoire.

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use spin::Mutex;
use x86_64::{PhysAddr, VirtAddr};

/// Clé spéciale pour créer un segment privé
pub const IPC_PRIVATE: i32 = 0;

/// Flags pour shmget
pub const IPC_CREAT: i32 = 0o1000;  // Créer si n'existe pas
pub const IPC_EXCL: i32 = 0o2000;   // Échouer si existe déjà

/// Commandes pour shmctl
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShmCmd {
    /// Obtenir les informations du segment
    IpcStat = 1,
    /// Modifier les informations du segment
    IpcSet = 2,
    /// Supprimer le segment
    IpcRmid = 3,
}

/// Erreurs de mémoire partagée
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShmError {
    /// Segment introuvable
    NotFound,
    /// Segment existe déjà (avec IPC_EXCL)
    AlreadyExists,
    /// Permission refusée
    PermissionDenied,
    /// Mémoire insuffisante
    OutOfMemory,
    /// Argument invalide
    InvalidArgument,
    /// Trop de segments
    TooManySegments,
}

/// Segment de mémoire partagée
#[derive(Debug, Clone)]
pub struct SharedMemorySegment {
    /// Identifiant unique du segment
    pub id: i32,
    /// Clé IPC
    pub key: i32,
    /// Taille en bytes
    pub size: usize,
    /// Adresse physique du segment
    pub phys_addr: PhysAddr,
    /// Propriétaire (UID)
    pub owner_uid: u32,
    /// Groupe (GID)
    pub owner_gid: u32,
    /// Permissions (rwxrwxrwx)
    pub permissions: u16,
    /// Nombre de processus attachés
    pub attached_count: usize,
    /// Timestamp de création
    pub created_at: u64,
    /// Timestamp de dernière attache
    pub last_attach: u64,
    /// Timestamp de dernière détache
    pub last_detach: u64,
}

impl SharedMemorySegment {
    /// Crée un nouveau segment
    fn new(id: i32, key: i32, size: usize, phys_addr: PhysAddr, uid: u32, gid: u32, permissions: u16) -> Self {
        Self {
            id,
            key,
            size,
            phys_addr,
            owner_uid: uid,
            owner_gid: gid,
            permissions,
            attached_count: 0,
            created_at: 0, // TODO: obtenir timestamp réel
            last_attach: 0,
            last_detach: 0,
        }
    }
    
    /// Vérifie si un processus a la permission d'accéder au segment
    pub fn check_permission(&self, uid: u32, gid: u32, write: bool) -> bool {
        // Propriétaire a tous les droits
        if uid == self.owner_uid {
            return true;
        }
        
        // Vérifier les permissions du groupe
        if gid == self.owner_gid {
            let group_perms = (self.permissions >> 3) & 0o7;
            if write {
                return (group_perms & 0o2) != 0; // Write permission
            } else {
                return (group_perms & 0o4) != 0; // Read permission
            }
        }
        
        // Vérifier les permissions des autres
        let other_perms = self.permissions & 0o7;
        if write {
            (other_perms & 0o2) != 0
        } else {
            (other_perms & 0o4) != 0
        }
    }
}

/// Gestionnaire de mémoire partagée
pub struct ShmManager {
    /// Segments indexés par ID
    segments: BTreeMap<i32, SharedMemorySegment>,
    /// Mapping clé → ID
    key_to_id: BTreeMap<i32, i32>,
    /// Prochain ID disponible
    next_id: i32,
    /// Nombre maximum de segments
    max_segments: usize,
}

impl ShmManager {
    /// Crée un nouveau gestionnaire
    pub const fn new() -> Self {
        Self {
            segments: BTreeMap::new(),
            key_to_id: BTreeMap::new(),
            next_id: 1,
            max_segments: 128,
        }
    }
    
    /// Crée ou récupère un segment de mémoire partagée
    /// 
    /// # Arguments
    /// * `key` - Clé IPC (IPC_PRIVATE pour segment privé)
    /// * `size` - Taille en bytes
    /// * `flags` - Flags (IPC_CREAT, IPC_EXCL, permissions)
    /// * `uid` - UID du processus appelant
    /// * `gid` - GID du processus appelant
    pub fn shmget(&mut self, key: i32, size: usize, flags: i32, uid: u32, gid: u32) -> Result<i32, ShmError> {
        // Vérifier le nombre de segments
        if self.segments.len() >= self.max_segments {
            return Err(ShmError::TooManySegments);
        }
        
        // Si clé privée, toujours créer un nouveau segment
        if key == IPC_PRIVATE {
            return self.create_segment(key, size, flags, uid, gid);
        }
        
        // Chercher un segment existant avec cette clé
        if let Some(&existing_id) = self.key_to_id.get(&key) {
            // Segment existe
            if (flags & IPC_EXCL) != 0 {
                return Err(ShmError::AlreadyExists);
            }
            return Ok(existing_id);
        }
        
        // Segment n'existe pas
        if (flags & IPC_CREAT) != 0 {
            self.create_segment(key, size, flags, uid, gid)
        } else {
            Err(ShmError::NotFound)
        }
    }
    
    /// Crée un nouveau segment
    fn create_segment(&mut self, key: i32, size: usize, flags: i32, uid: u32, gid: u32) -> Result<i32, ShmError> {
        // Allouer de la mémoire physique pour le segment
        // TODO: utiliser un vrai allocateur de pages
        let phys_addr = self.allocate_physical_memory(size)?;
        
        let id = self.next_id;
        self.next_id += 1;
        
        // Extraire les permissions des flags (9 bits de poids faible)
        let permissions = (flags & 0o777) as u16;
        
        let segment = SharedMemorySegment::new(id, key, size, phys_addr, uid, gid, permissions);
        
        self.segments.insert(id, segment);
        if key != IPC_PRIVATE {
            self.key_to_id.insert(key, id);
        }
        
        Ok(id)
    }
    
    /// Attache un segment à l'espace d'adressage du processus
    /// 
    /// # Arguments
    /// * `id` - ID du segment
    /// * `addr` - Adresse virtuelle souhaitée (None = auto)
    /// * `uid` - UID du processus
    /// * `gid` - GID du processus
    pub fn shmat(&mut self, id: i32, addr: Option<VirtAddr>, uid: u32, gid: u32) -> Result<VirtAddr, ShmError> {
        let segment = self.segments.get_mut(&id).ok_or(ShmError::NotFound)?;
        
        // Vérifier les permissions
        if !segment.check_permission(uid, gid, false) {
            return Err(ShmError::PermissionDenied);
        }
        
        // Déterminer l'adresse virtuelle
        let virt_addr = addr.unwrap_or_else(|| {
            // TODO: trouver une adresse libre dans l'espace d'adressage
            VirtAddr::new(0x7000_0000_0000) // Placeholder
        });
        
        // TODO: mapper les pages physiques dans l'espace d'adressage du processus
        
        segment.attached_count += 1;
        segment.last_attach = 0; // TODO: timestamp réel
        
        Ok(virt_addr)
    }
    
    /// Détache un segment de l'espace d'adressage du processus
    /// 
    /// # Arguments
    /// * `addr` - Adresse virtuelle du segment
    pub fn shmdt(&mut self, addr: VirtAddr) -> Result<(), ShmError> {
        // TODO: trouver le segment correspondant à cette adresse
        // TODO: unmapper les pages
        // TODO: décrémenter attached_count
        
        // Pour l'instant, juste un placeholder
        Ok(())
    }
    
    /// Contrôle un segment (stats, delete, etc.)
    /// 
    /// # Arguments
    /// * `id` - ID du segment
    /// * `cmd` - Commande à exécuter
    /// * `uid` - UID du processus
    pub fn shmctl(&mut self, id: i32, cmd: ShmCmd, uid: u32) -> Result<Option<SharedMemorySegment>, ShmError> {
        let segment = self.segments.get(&id).ok_or(ShmError::NotFound)?;
        
        match cmd {
            ShmCmd::IpcStat => {
                // Retourner les informations du segment
                Ok(Some(segment.clone()))
            }
            
            ShmCmd::IpcSet => {
                // Modifier les informations (permissions, etc.)
                // Seul le propriétaire peut modifier
                if uid != segment.owner_uid {
                    return Err(ShmError::PermissionDenied);
                }
                // TODO: implémenter la modification
                Ok(None)
            }
            
            ShmCmd::IpcRmid => {
                // Supprimer le segment
                // Seul le propriétaire peut supprimer
                if uid != segment.owner_uid {
                    return Err(ShmError::PermissionDenied);
                }
                
                // Vérifier qu'aucun processus n'est attaché
                if segment.attached_count > 0 {
                    return Err(ShmError::InvalidArgument);
                }
                
                // Cloner les données nécessaires avant de supprimer
                let key = segment.key;
                
                // Supprimer le segment
                self.segments.remove(&id);
                if key != IPC_PRIVATE {
                    self.key_to_id.remove(&key);
                }
                
                // TODO: libérer la mémoire physique
                
                Ok(None)
            }
        }
    }
    
    /// Alloue de la mémoire physique pour un segment
    fn allocate_physical_memory(&self, size: usize) -> Result<PhysAddr, ShmError> {
        // TODO: utiliser le Buddy Allocator ou un allocateur de pages
        // Pour l'instant, retourner une adresse placeholder
        Ok(PhysAddr::new(0x1000_0000))
    }
    
    /// Retourne les statistiques
    pub fn get_stats(&self) -> ShmStats {
        ShmStats {
            total_segments: self.segments.len(),
            total_attached: self.segments.values().map(|s| s.attached_count).sum(),
        }
    }
}

/// Statistiques de mémoire partagée
#[derive(Debug, Clone, Copy)]
pub struct ShmStats {
    pub total_segments: usize,
    pub total_attached: usize,
}

/// Instance globale du gestionnaire SHM
pub static SHM_MANAGER: Mutex<ShmManager> = Mutex::new(ShmManager::new());

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test_case]
    fn test_shmget_create() {
        let mut manager = ShmManager::new();
        let id = manager.shmget(1234, 4096, IPC_CREAT | 0o666, 1000, 1000);
        assert!(id.is_ok());
    }
    
    #[test_case]
    fn test_shmget_exclusive() {
        let mut manager = ShmManager::new();
        let _ = manager.shmget(1234, 4096, IPC_CREAT | 0o666, 1000, 1000);
        let result = manager.shmget(1234, 4096, IPC_CREAT | IPC_EXCL | 0o666, 1000, 1000);
        assert_eq!(result, Err(ShmError::AlreadyExists));
    }
    
    #[test_case]
    fn test_permissions() {
        let segment = SharedMemorySegment::new(1, 1234, 4096, PhysAddr::new(0), 1000, 1000, 0o644);
        
        // Propriétaire peut tout faire
        assert!(segment.check_permission(1000, 1000, true));
        
        // Groupe peut lire
        assert!(segment.check_permission(1001, 1000, false));
        
        // Groupe ne peut pas écrire
        assert!(!segment.check_permission(1001, 1000, true));
        
        // Autres peuvent lire
        assert!(segment.check_permission(1001, 1001, false));
    }
}
