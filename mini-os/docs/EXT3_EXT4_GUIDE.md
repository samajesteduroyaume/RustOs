# Guide d'Utilisation - EXT3/EXT4

## Installation

Les modules sont déjà intégrés dans RustOS :
```rust
use mini_os::ext3::Ext3;
use mini_os::ext4::Ext4;
use mini_os::fs::JournalMode;
```

## Exemple Complet

```rust
use mini_os::ext4::Ext4;
use mini_os::fs::JournalMode;
use mini_os::drivers::disk::DiskDriver;

fn main() {
    // 1. Initialiser le disque
    let mut disk = DiskDriver::new("sda", true);
    disk.init().unwrap();
    
    // 2. Créer filesystem EXT4
    let mut fs = Ext4::new(disk, JournalMode::Ordered).unwrap();
    
    // 3. Monter (recovery automatique)
    fs.mount().unwrap();
    
    // 4. Opérations fichiers
    fs.create_dir("/home").unwrap();
    fs.write_file("/home/README.txt", b"Welcome to RustOS!").unwrap();
    
    // 5. Lecture
    let content = fs.read_file("/home/README.txt").unwrap();
    println!("{}", String::from_utf8_lossy(&content));
    
    // 6. Preallocation (EXT4)
    fs.preallocate("/home/bigfile.dat", 100 * 1024 * 1024).unwrap();
    
    // 7. Statistiques
    let stats = fs.get_stats();
    println!("Journal commits: {}", stats.journal_commits);
    
    // 8. Sync
    fs.sync().unwrap();
}
```

## API Reference

### EXT3

```rust
impl<D: Disk> Ext3<D> {
    // Création
    pub fn new(disk: D, mode: JournalMode) -> Result<Self, Ext2Error>;
    
    // Montage
    pub fn mount(&mut self) -> Result<(), FsError>;
    
    // Lecture (non journalisée)
    pub fn read_dir(&self, path: &str) -> Result<Vec<String>, FsError>;
    pub fn read_file(&self, path: &str) -> Result<Vec<u8>, FsError>;
    
    // Écriture (journalisée)
    pub fn write_file(&mut self, path: &str, content: &[u8]) -> Result<(), FsError>;
    pub fn create_file(&mut self, path: &str, content: &[u8]) -> Result<(), FsError>;
    pub fn delete_file(&mut self, path: &str) -> Result<(), FsError>;
    pub fn create_dir(&mut self, path: &str) -> Result<(), FsError>;
    
    // Statistiques
    pub fn get_journal_stats(&self) -> JournalStats;
    pub fn sync(&mut self) -> Result<(), FsError>;
}
```

### EXT4

```rust
impl<D: Disk> Ext4<D> {
    // Toutes les méthodes EXT3 +
    
    // Preallocation
    pub fn preallocate(&mut self, path: &str, size: u64) -> Result<(), FsError>;
    
    // Statistiques extents
    pub fn get_extent_stats(&self, path: &str) -> Result<ExtentStats, FsError>;
    pub fn get_stats(&self) -> Ext4Stats;
}
```

## Modes de Journaling

```rust
// Ordered (recommandé)
let fs = Ext3::new(disk, JournalMode::Ordered)?;

// Writeback (plus rapide)
let fs = Ext3::new(disk, JournalMode::Writeback)?;

// Journal (plus sûr)
let fs = Ext3::new(disk, JournalMode::Journal)?;
```

## Gestion des Erreurs

```rust
match fs.write_file("/test.txt", b"data") {
    Ok(_) => println!("Succès"),
    Err(FsError::NoSpace) => println!("Disque plein"),
    Err(FsError::NotFound) => println!("Fichier non trouvé"),
    Err(e) => println!("Erreur: {:?}", e),
}
```
