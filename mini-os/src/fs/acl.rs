/// Module ACL (Access Control Lists)
/// 
/// Implémente les ACLs POSIX pour permissions granulaires

use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use spin::Mutex;

/// Type d'entrée ACL
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AclEntryType {
    /// Propriétaire du fichier
    UserObj,
    /// Utilisateur spécifique
    User,
    /// Groupe propriétaire
    GroupObj,
    /// Groupe spécifique
    Group,
    /// Masque (limite permissions effectives)
    Mask,
    /// Autres utilisateurs
    Other,
}

/// Permissions ACL (similaire à Unix mais plus flexible)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AclPermissions {
    pub read: bool,
    pub write: bool,
    pub execute: bool,
}

impl AclPermissions {
    /// Crée des permissions vides
    pub fn none() -> Self {
        Self {
            read: false,
            write: false,
            execute: false,
        }
    }
    
    /// Crée des permissions depuis un mode octal (rwx)
    pub fn from_mode(mode: u8) -> Self {
        Self {
            read: (mode & 0o4) != 0,
            write: (mode & 0o2) != 0,
            execute: (mode & 0o1) != 0,
        }
    }
    
    /// Convertit en mode octal
    pub fn to_mode(&self) -> u8 {
        let mut mode = 0;
        if self.read { mode |= 0o4; }
        if self.write { mode |= 0o2; }
        if self.execute { mode |= 0o1; }
        mode
    }
    
    /// Applique un masque
    pub fn apply_mask(&self, mask: &AclPermissions) -> Self {
        Self {
            read: self.read && mask.read,
            write: self.write && mask.write,
            execute: self.execute && mask.execute,
        }
    }
}

/// Entrée ACL
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AclEntry {
    /// Type d'entrée
    pub entry_type: AclEntryType,
    /// ID (UID pour User, GID pour Group, ignoré pour autres)
    pub id: Option<u32>,
    /// Permissions
    pub permissions: AclPermissions,
}

impl AclEntry {
    /// Crée une nouvelle entrée
    pub fn new(entry_type: AclEntryType, id: Option<u32>, permissions: AclPermissions) -> Self {
        Self {
            entry_type,
            id,
            permissions,
        }
    }
    
    /// Crée une entrée USER
    pub fn user(uid: u32, permissions: AclPermissions) -> Self {
        Self::new(AclEntryType::User, Some(uid), permissions)
    }
    
    /// Crée une entrée GROUP
    pub fn group(gid: u32, permissions: AclPermissions) -> Self {
        Self::new(AclEntryType::Group, Some(gid), permissions)
    }
}

/// Liste de contrôle d'accès (ACL)
#[derive(Debug, Clone)]
pub struct Acl {
    /// Entrées ACL
    entries: Vec<AclEntry>,
}

impl Acl {
    /// Crée une ACL vide
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
    
    /// Crée une ACL minimale depuis permissions Unix
    pub fn from_unix_mode(mode: u16, uid: u32, gid: u32) -> Self {
        let mut acl = Self::new();
        
        // USER_OBJ (propriétaire)
        let user_perms = AclPermissions::from_mode(((mode >> 6) & 0o7) as u8);
        acl.add_entry(AclEntry::new(AclEntryType::UserObj, Some(uid), user_perms));
        
        // GROUP_OBJ (groupe)
        let group_perms = AclPermissions::from_mode(((mode >> 3) & 0o7) as u8);
        acl.add_entry(AclEntry::new(AclEntryType::GroupObj, Some(gid), group_perms));
        
        // OTHER
        let other_perms = AclPermissions::from_mode((mode & 0o7) as u8);
        acl.add_entry(AclEntry::new(AclEntryType::Other, None, other_perms));
        
        acl
    }
    
    /// Ajoute une entrée
    pub fn add_entry(&mut self, entry: AclEntry) {
        // Vérifier si une entrée similaire existe déjà
        if let Some(pos) = self.entries.iter().position(|e| {
            e.entry_type == entry.entry_type && e.id == entry.id
        }) {
            // Remplacer
            self.entries[pos] = entry;
        } else {
            // Ajouter
            self.entries.push(entry);
        }
        
        // Trier les entrées (ordre canonique POSIX)
        self.entries.sort_by(|a, b| {
            a.entry_type.cmp(&b.entry_type)
                .then_with(|| a.id.cmp(&b.id))
        });
    }
    
    /// Supprime une entrée
    pub fn remove_entry(&mut self, entry_type: AclEntryType, id: Option<u32>) -> bool {
        if let Some(pos) = self.entries.iter().position(|e| {
            e.entry_type == entry_type && e.id == id
        }) {
            self.entries.remove(pos);
            true
        } else {
            false
        }
    }
    
    /// Vérifie les permissions pour un utilisateur
    pub fn check_permission(&self, uid: u32, gid: u32, groups: &[u32], perm_type: PermissionType) -> bool {
        // 1. Vérifier USER_OBJ si c'est le propriétaire
        if let Some(entry) = self.entries.iter().find(|e| {
            e.entry_type == AclEntryType::UserObj && e.id == Some(uid)
        }) {
            return self.has_permission(&entry.permissions, perm_type);
        }
        
        // 2. Vérifier USER spécifique
        if let Some(entry) = self.entries.iter().find(|e| {
            e.entry_type == AclEntryType::User && e.id == Some(uid)
        }) {
            let effective = self.apply_mask(&entry.permissions);
            return self.has_permission(&effective, perm_type);
        }
        
        // 3. Vérifier GROUP_OBJ si dans le groupe
        if let Some(entry) = self.entries.iter().find(|e| {
            e.entry_type == AclEntryType::GroupObj && e.id == Some(gid)
        }) {
            let effective = self.apply_mask(&entry.permissions);
            if self.has_permission(&effective, perm_type) {
                return true;
            }
        }
        
        // 4. Vérifier GROUP spécifiques
        for group_id in groups {
            if let Some(entry) = self.entries.iter().find(|e| {
                e.entry_type == AclEntryType::Group && e.id == Some(*group_id)
            }) {
                let effective = self.apply_mask(&entry.permissions);
                if self.has_permission(&effective, perm_type) {
                    return true;
                }
            }
        }
        
        // 5. Vérifier OTHER
        if let Some(entry) = self.entries.iter().find(|e| {
            e.entry_type == AclEntryType::Other
        }) {
            return self.has_permission(&entry.permissions, perm_type);
        }
        
        false
    }
    
    /// Applique le masque si présent
    fn apply_mask(&self, perms: &AclPermissions) -> AclPermissions {
        if let Some(mask_entry) = self.entries.iter().find(|e| {
            e.entry_type == AclEntryType::Mask
        }) {
            perms.apply_mask(&mask_entry.permissions)
        } else {
            *perms
        }
    }
    
    /// Vérifie si une permission est accordée
    fn has_permission(&self, perms: &AclPermissions, perm_type: PermissionType) -> bool {
        match perm_type {
            PermissionType::Read => perms.read,
            PermissionType::Write => perms.write,
            PermissionType::Execute => perms.execute,
        }
    }
    
    /// Retourne toutes les entrées
    pub fn get_entries(&self) -> &[AclEntry] {
        &self.entries
    }
    
    /// Nombre d'entrées
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    
    /// Vérifie si l'ACL est vide
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Type de permission à vérifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionType {
    Read,
    Write,
    Execute,
}

/// Gestionnaire d'ACLs
pub struct AclManager {
    /// ACLs par inode
    acls: BTreeMap<u64, Acl>,
}

impl AclManager {
    /// Crée un nouveau gestionnaire
    pub const fn new() -> Self {
        Self {
            acls: BTreeMap::new(),
        }
    }
    
    /// Définit l'ACL pour un inode
    pub fn set_acl(&mut self, inode: u64, acl: Acl) {
        self.acls.insert(inode, acl);
    }
    
    /// Récupère l'ACL d'un inode
    pub fn get_acl(&self, inode: u64) -> Option<&Acl> {
        self.acls.get(&inode)
    }
    
    /// Récupère l'ACL mutable d'un inode
    pub fn get_acl_mut(&mut self, inode: u64) -> Option<&mut Acl> {
        self.acls.get_mut(&inode)
    }
    
    /// Supprime l'ACL d'un inode
    pub fn remove_acl(&mut self, inode: u64) -> Option<Acl> {
        self.acls.remove(&inode)
    }
    
    /// Vérifie une permission
    pub fn check_permission(&self, inode: u64, uid: u32, gid: u32, groups: &[u32], perm_type: PermissionType) -> bool {
        if let Some(acl) = self.acls.get(&inode) {
            acl.check_permission(uid, gid, groups, perm_type)
        } else {
            false
        }
    }
    
    /// Retourne les statistiques
    pub fn get_stats(&self) -> AclStats {
        AclStats {
            total_acls: self.acls.len(),
            total_entries: self.acls.values().map(|a| a.len()).sum(),
        }
    }
}

/// Statistiques ACL
#[derive(Debug, Clone, Copy)]
pub struct AclStats {
    pub total_acls: usize,
    pub total_entries: usize,
}

/// Instance globale du gestionnaire ACL
use lazy_static::lazy_static;

lazy_static! {
    pub static ref ACL_MANAGER: Mutex<AclManager> = Mutex::new(AclManager::new());
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test_case]
    fn test_acl_permissions() {
        let perms = AclPermissions::from_mode(0o7);
        assert!(perms.read);
        assert!(perms.write);
        assert!(perms.execute);
        assert_eq!(perms.to_mode(), 0o7);
    }
    
    #[test_case]
    fn test_acl_entry() {
        let entry = AclEntry::user(1000, AclPermissions::from_mode(0o6));
        assert_eq!(entry.entry_type, AclEntryType::User);
        assert_eq!(entry.id, Some(1000));
        assert!(entry.permissions.read);
        assert!(entry.permissions.write);
        assert!(!entry.permissions.execute);
    }
    
    #[test_case]
    fn test_acl_from_unix() {
        let acl = Acl::from_unix_mode(0o755, 1000, 1000);
        assert_eq!(acl.len(), 3);
    }
    
    #[test_case]
    fn test_acl_check_permission() {
        let mut acl = Acl::new();
        acl.add_entry(AclEntry::new(AclEntryType::UserObj, Some(1000), AclPermissions::from_mode(0o7)));
        acl.add_entry(AclEntry::new(AclEntryType::Other, None, AclPermissions::from_mode(0o4)));
        
        // Propriétaire peut tout faire
        assert!(acl.check_permission(1000, 1000, &[], PermissionType::Read));
        assert!(acl.check_permission(1000, 1000, &[], PermissionType::Write));
        
        // Autres peuvent seulement lire
        assert!(acl.check_permission(2000, 2000, &[], PermissionType::Read));
        assert!(!acl.check_permission(2000, 2000, &[], PermissionType::Write));
    }
    
    #[test_case]
    fn test_acl_mask() {
        let perms = AclPermissions::from_mode(0o7);
        let mask = AclPermissions::from_mode(0o4);
        let effective = perms.apply_mask(&mask);
        
        assert!(effective.read);
        assert!(!effective.write);
        assert!(!effective.execute);
    }
}
