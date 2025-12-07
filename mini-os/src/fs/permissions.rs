/// Module de gestion des permissions Unix
/// 
/// Implémente le modèle de permissions Unix (rwxrwxrwx) avec UID/GID

use alloc::collections::BTreeMap;
use spin::Mutex;

/// Permissions Unix (mode)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Permissions {
    /// Mode (rwxrwxrwx)
    mode: u16,
    /// UID propriétaire
    uid: u32,
    /// GID groupe
    gid: u32,
}

impl Permissions {
    /// Permissions utilisateur
    pub const USER_READ: u16 = 0o400;
    pub const USER_WRITE: u16 = 0o200;
    pub const USER_EXEC: u16 = 0o100;
    
    /// Permissions groupe
    pub const GROUP_READ: u16 = 0o040;
    pub const GROUP_WRITE: u16 = 0o020;
    pub const GROUP_EXEC: u16 = 0o010;
    
    /// Permissions autres
    pub const OTHER_READ: u16 = 0o004;
    pub const OTHER_WRITE: u16 = 0o002;
    pub const OTHER_EXEC: u16 = 0o001;
    
    /// Bits spéciaux
    pub const SUID: u16 = 0o4000;  // Set UID
    pub const SGID: u16 = 0o2000;  // Set GID
    pub const STICKY: u16 = 0o1000; // Sticky bit
    
    /// Crée de nouvelles permissions
    pub fn new(mode: u16, uid: u32, gid: u32) -> Self {
        Self { mode, uid, gid }
    }
    
    /// Permissions par défaut (0644, root:root)
    pub fn default() -> Self {
        Self::new(0o644, 0, 0)
    }
    
    /// Retourne le mode
    pub fn mode(&self) -> u16 {
        self.mode
    }
    
    /// Retourne l'UID
    pub fn uid(&self) -> u32 {
        self.uid
    }
    
    /// Retourne le GID
    pub fn gid(&self) -> u32 {
        self.gid
    }
    
    /// Change le mode
    pub fn set_mode(&mut self, mode: u16) {
        self.mode = mode;
    }
    
    /// Change l'UID
    pub fn set_uid(&mut self, uid: u32) {
        self.uid = uid;
    }
    
    /// Change le GID
    pub fn set_gid(&mut self, gid: u32) {
        self.gid = gid;
    }
    
    /// Vérifie si l'utilisateur a la permission de lire
    pub fn can_read(&self, uid: u32, gid: u32) -> bool {
        if uid == 0 {
            return true; // Root peut tout faire
        }
        
        if uid == self.uid {
            // Propriétaire
            (self.mode & Self::USER_READ) != 0
        } else if gid == self.gid {
            // Groupe
            (self.mode & Self::GROUP_READ) != 0
        } else {
            // Autres
            (self.mode & Self::OTHER_READ) != 0
        }
    }
    
    /// Vérifie si l'utilisateur a la permission d'écrire
    pub fn can_write(&self, uid: u32, gid: u32) -> bool {
        if uid == 0 {
            return true; // Root peut tout faire
        }
        
        if uid == self.uid {
            (self.mode & Self::USER_WRITE) != 0
        } else if gid == self.gid {
            (self.mode & Self::GROUP_WRITE) != 0
        } else {
            (self.mode & Self::OTHER_WRITE) != 0
        }
    }
    
    /// Vérifie si l'utilisateur a la permission d'exécuter
    pub fn can_execute(&self, uid: u32, gid: u32) -> bool {
        if uid == 0 {
            return true; // Root peut tout faire
        }
        
        if uid == self.uid {
            (self.mode & Self::USER_EXEC) != 0
        } else if gid == self.gid {
            (self.mode & Self::GROUP_EXEC) != 0
        } else {
            (self.mode & Self::OTHER_EXEC) != 0
        }
    }
    
    /// Vérifie si le SUID est activé
    pub fn has_suid(&self) -> bool {
        (self.mode & Self::SUID) != 0
    }
    
    /// Vérifie si le SGID est activé
    pub fn has_sgid(&self) -> bool {
        (self.mode & Self::SGID) != 0
    }
    
    /// Vérifie si le sticky bit est activé
    pub fn has_sticky(&self) -> bool {
        (self.mode & Self::STICKY) != 0
    }
}

/// Erreurs de permissions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionError {
    /// Permission refusée
    PermissionDenied,
    /// Fichier non trouvé
    NotFound,
    /// Opération non permise
    NotPermitted,
    /// Argument invalide
    InvalidArgument,
}

/// Gestionnaire de permissions
pub struct PermissionManager {
    /// Permissions par inode
    permissions: BTreeMap<u64, Permissions>,
}

impl PermissionManager {
    /// Crée un nouveau gestionnaire
    pub const fn new() -> Self {
        Self {
            permissions: BTreeMap::new(),
        }
    }
    
    /// Définit les permissions pour un inode
    pub fn set_permissions(&mut self, inode: u64, perms: Permissions) {
        self.permissions.insert(inode, perms);
    }
    
    /// Récupère les permissions d'un inode
    pub fn get_permissions(&self, inode: u64) -> Option<Permissions> {
        self.permissions.get(&inode).copied()
    }
    
    /// Change le mode (chmod)
    /// 
    /// Seul le propriétaire ou root peut changer le mode
    pub fn chmod(&mut self, inode: u64, mode: u16, caller_uid: u32) -> Result<(), PermissionError> {
        if let Some(perms) = self.permissions.get_mut(&inode) {
            // Vérifier que l'appelant est le propriétaire ou root
            if caller_uid != 0 && caller_uid != perms.uid {
                return Err(PermissionError::PermissionDenied);
            }
            
            perms.set_mode(mode);
            Ok(())
        } else {
            Err(PermissionError::NotFound)
        }
    }
    
    /// Change le propriétaire (chown)
    /// 
    /// Seul root peut changer le propriétaire
    pub fn chown(&mut self, inode: u64, uid: u32, caller_uid: u32) -> Result<(), PermissionError> {
        if caller_uid != 0 {
            return Err(PermissionError::NotPermitted);
        }
        
        if let Some(perms) = self.permissions.get_mut(&inode) {
            perms.set_uid(uid);
            Ok(())
        } else {
            Err(PermissionError::NotFound)
        }
    }
    
    /// Change le groupe (chgrp)
    /// 
    /// Le propriétaire ou root peut changer le groupe
    pub fn chgrp(&mut self, inode: u64, gid: u32, caller_uid: u32) -> Result<(), PermissionError> {
        if let Some(perms) = self.permissions.get_mut(&inode) {
            // Vérifier que l'appelant est le propriétaire ou root
            if caller_uid != 0 && caller_uid != perms.uid {
                return Err(PermissionError::PermissionDenied);
            }
            
            perms.set_gid(gid);
            Ok(())
        } else {
            Err(PermissionError::NotFound)
        }
    }
    
    /// Vérifie l'accès (access)
    pub fn check_access(&self, inode: u64, uid: u32, gid: u32, mode: u8) -> bool {
        if let Some(perms) = self.permissions.get(&inode) {
            let read = (mode & 4) != 0;
            let write = (mode & 2) != 0;
            let exec = (mode & 1) != 0;
            
            if read && !perms.can_read(uid, gid) {
                return false;
            }
            if write && !perms.can_write(uid, gid) {
                return false;
            }
            if exec && !perms.can_execute(uid, gid) {
                return false;
            }
            
            true
        } else {
            false
        }
    }
    
    /// Retourne les statistiques
    pub fn get_stats(&self) -> PermissionStats {
        PermissionStats {
            total_inodes: self.permissions.len(),
        }
    }
}

/// Statistiques des permissions
#[derive(Debug, Clone, Copy)]
pub struct PermissionStats {
    pub total_inodes: usize,
}

/// Instance globale du gestionnaire de permissions
use lazy_static::lazy_static;

lazy_static! {
    pub static ref PERMISSION_MANAGER: Mutex<PermissionManager> = Mutex::new(PermissionManager::new());
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test_case]
    fn test_permissions_creation() {
        let perms = Permissions::new(0o755, 1000, 1000);
        assert_eq!(perms.mode(), 0o755);
        assert_eq!(perms.uid(), 1000);
        assert_eq!(perms.gid(), 1000);
    }
    
    #[test_case]
    fn test_can_read() {
        let perms = Permissions::new(0o644, 1000, 1000);
        
        // Propriétaire peut lire
        assert!(perms.can_read(1000, 1000));
        
        // Groupe peut lire
        assert!(perms.can_read(1001, 1000));
        
        // Autres peuvent lire
        assert!(perms.can_read(1001, 1001));
    }
    
    #[test_case]
    fn test_can_write() {
        let perms = Permissions::new(0o644, 1000, 1000);
        
        // Propriétaire peut écrire
        assert!(perms.can_write(1000, 1000));
        
        // Groupe ne peut pas écrire
        assert!(!perms.can_write(1001, 1000));
        
        // Autres ne peuvent pas écrire
        assert!(!perms.can_write(1001, 1001));
    }
    
    #[test_case]
    fn test_root_can_do_everything() {
        let perms = Permissions::new(0o000, 1000, 1000);
        
        // Root peut tout faire même avec mode 000
        assert!(perms.can_read(0, 0));
        assert!(perms.can_write(0, 0));
        assert!(perms.can_execute(0, 0));
    }
    
    #[test_case]
    fn test_chmod() {
        let mut manager = PermissionManager::new();
        manager.set_permissions(1, Permissions::new(0o644, 1000, 1000));
        
        // Propriétaire peut changer le mode
        assert!(manager.chmod(1, 0o755, 1000).is_ok());
        assert_eq!(manager.get_permissions(1).unwrap().mode(), 0o755);
        
        // Autre utilisateur ne peut pas
        assert!(manager.chmod(1, 0o777, 1001).is_err());
    }
    
    #[test_case]
    fn test_chown() {
        let mut manager = PermissionManager::new();
        manager.set_permissions(1, Permissions::new(0o644, 1000, 1000));
        
        // Seul root peut changer le propriétaire
        assert!(manager.chown(1, 2000, 0).is_ok());
        assert_eq!(manager.get_permissions(1).unwrap().uid(), 2000);
        
        // Utilisateur normal ne peut pas
        assert!(manager.chown(1, 3000, 1000).is_err());
    }
}
