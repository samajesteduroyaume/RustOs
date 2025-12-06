# Guide d'installation

## Prérequis

- Rust 1.70 ou supérieur
- `cargo` (gestionnaire de paquets Rust)
- `make` (pour les scripts de construction)
- Un compilateur C (pour certaines dépendances)

## Installation

1. Cloner le dépôt :
   ```bash
   git clone https://github.com/votre-utilisateur/ufat.git
   cd ufat
   ```

2. Compiler le projet :
   ```bash
   cargo build --release
   ```

3. Tester l'installation :
   ```bash
   cargo test
   ```

## Utilisation basique

Pour formater un périphérique avec UFAT :

```rust
use ufat::{UFAT, Disk};

let disk = /* votre implémentation de Disk */;
UFAT::format(disk, "MON_VOLUME")?;
```

Pour monter un système de fichiers existant :

```rust
let fs = UFAT::new(disk)?;
```

## Intégration avec un système d'exploitation

Voir le fichier [integration.md](integration.md) pour les instructions détaillées sur l'intégration avec un noyau de système d'exploitation.
