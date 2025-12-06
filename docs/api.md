# Référence d'API

## UFAT

### `UFAT::new(disk: D) -> Result<Self, FsError>`
Crée une nouvelle instance du système de fichiers UFAT sur un périphérique de disque.

### `UFAT::format(disk: D, volume_name: &str) -> Result<(), FsError>`
Formate le périphérique avec le système de fichiers UFAT.

### `read_file(&self, path: &str) -> Result<Vec<u8>, FsError>`
Lit le contenu d'un fichier.

### `write_file(&mut self, path: &str, content: &[u8]) -> Result<(), FsError>`
Écrit du contenu dans un fichier, le créant s'il n'existe pas.

### `create_dir(&mut self, path: &str) -> Result<(), FsError>`
Crée un nouveau répertoire.

### `read_dir(&self, path: &str) -> Result<Vec<String>, FsError>`
Liste le contenu d'un répertoire.

### `remove_file(&mut self, path: &str) -> Result<(), FsError>`
Supprime un fichier.

### `remove_dir(&mut self, path: &str) -> Result<(), FsError>`
Supprime un répertoire vide.

## Structures

### `UfatSuperBlock`
Contient les métadonnées du système de fichiers.

### `BlockGroupDescriptor`
Décrit un groupe de blocs.

### `UfatInode`
Représente un inode dans le système de fichiers.

### `DirEntry`
Entrée de répertoire.

## Exemples

### Création d'un système de fichiers

```rust
let disk = /* votre implémentation de Disk */;
UFAT::format(disk, "MON_VOLUME")?;
```

### Manipulation de fichiers

```rust
let mut fs = UFAT::new(disk)?;

// Créer un fichier
fs.write_file("/test.txt", b"Hello, UFAT!")?;

// Lire un fichier
let content = fs.read_file("/test.txt")?;

// Créer un répertoire
fs.create_dir("/documents")?;

// Lister un répertoire
let entries = fs.read_dir("/")?;
```
