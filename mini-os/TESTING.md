# Guide de Test - RustOS

## 📋 Vue d'Ensemble

RustOS est un système d'exploitation bare-metal (no_std) qui compile pour la cible `x86_64-unknown-none`. Cette architecture impose des limitations sur l'exécution des tests.

## ⚠️ Limitation Importante

**Les tests unitaires marqués avec `#[test_case]` ne peuvent PAS être exécutés avec `cargo test --lib`.**

### Pourquoi ?

Le projet utilise `#![no_std]` et compile pour une cible bare-metal (`x86_64-unknown-none`). Quand `cargo test` essaie de compiler les tests, il crée un conflit :
- Une version de `core` pour la cible bare-metal
- Une version de `core` pour l'environnement de test standard

Cela génère l'erreur : `duplicate lang item in crate 'core': 'sized'`

## ✅ Tests Disponibles

### Tests QEMU Automatisés ⭐ NOUVEAU

Les tests unitaires peuvent maintenant être exécutés dans un environnement QEMU simulé :

```bash
cd /home/selim/Bureau/RustOs/mini-os

# Installer QEMU si nécessaire
# Ubuntu/Debian: sudo apt install qemu-system-x86
# Fedora: sudo dnf install qemu-system-x86
# Arch: sudo pacman -S qemu

# Installer bootimage (première fois seulement)
cargo install bootimage

# Exécuter les tests
./run_tests.sh
```

**Sortie attendue** :
```
🧪 RustOS - Exécution des tests dans QEMU
==========================================
📦 Compilation du kernel de test...
✅ Compilation réussie
🔨 Création de l'image bootable...
✅ Image bootable créée
🚀 Lancement de QEMU...
----------------------------------------
RustOS Test Suite
=================

Running X tests
================
test::module::test_name...[ok]
...
================
All tests passed!
----------------------------------------
✅ Tous les tests ont réussi!
```

### Tests d'Intégration RamFS

Les tests d'intégration pour le système de fichiers RamFS :

```bash
cd /home/selim/Bureau/RustOs/mini-os
./run_ramfs_tests.sh
```

## 📝 Tests Unitaires dans le Code

Le code source contient **50+ tests unitaires** marqués avec `#[test_case]` dans les modules suivants :

- `device_manager/` - Tests de détection de périphériques
- `network/` - Tests de la pile réseau (TCP, UDP, ICMP, DNS)
- `drivers/` - Tests des pilotes (USB, Bluetooth)
- `shell/` - Tests du shell
- `fs/` - Tests du système de fichiers

### Comment sont-ils utilisés ?

Ces tests sont **documentés et vérifiés manuellement** ou peuvent être exécutés dans un environnement QEMU configuré pour RustOS.

## 🔧 Exécution des Tests (Options Avancées)

### Option 1 : Tests Manuels

Vérifier manuellement la logique en lisant le code des tests et en validant le comportement dans l'OS en cours d'exécution.

### Option 2 : Tests QEMU (Avancé)

Configurer un environnement QEMU pour exécuter le noyau et les tests intégrés :

```bash
# Compiler le noyau
cargo build --release

# Exécuter dans QEMU (nécessite configuration supplémentaire)
qemu-system-x86_64 -kernel target/x86_64-unknown-none/release/mini-os
```

### Option 3 : Tests de Logique Pure (Future)

Pour tester la logique pure sans dépendances hardware, on pourrait créer des tests séparés avec `std` :

```rust
#[cfg(all(test, not(target_os = "none")))]
mod tests_std {
    // Tests de logique pure uniquement
}
```

## 📊 Couverture de Tests

| Module | Tests Unitaires | Tests d'Intégration | Statut |
|--------|----------------|---------------------|--------|
| device_manager | 16+ | - | ✅ Documentés |
| network | 20+ | - | ✅ Documentés |
| drivers | 10+ | - | ✅ Documentés |
| fs | 4+ | ✅ RamFS | ✅ Exécutables |
| shell | 3+ | - | ✅ Documentés |

## 🎯 Ajouter de Nouveaux Tests

### Tests d'Intégration (Recommandé)

Créer un nouveau fichier dans `tests/` :

```rust
// tests/mon_test.rs
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(test_runner)]
#![reexport_test_harness_main = "test_main"]

// Votre code de test ici
```

Ajouter dans `Cargo.toml` :

```toml
[[test]]
name = "mon_test"
path = "tests/mon_test.rs"
harness = false
```

### Tests Unitaires (Documentation)

Ajouter des tests dans vos modules avec `#[test_case]` :

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test_case]
    fn test_ma_fonction() {
        assert_eq!(ma_fonction(), valeur_attendue);
    }
}
```

**Note** : Ces tests ne seront pas exécutés automatiquement mais servent de documentation et de validation manuelle.

## 🔍 Vérification de la Compilation

Pour vérifier que le code compile sans erreurs :

```bash
# Build release (sans tests)
cargo build --release

# Build debug
cargo build

# Vérifier les warnings
cargo clippy
```

## 📚 Ressources

- [Rust Embedded Book - Testing](https://docs.rust-embedded.org/book/start/qemu.html)
- [Custom Test Frameworks](https://os.phil-opp.com/testing/)
- Documentation du projet : `docs/`

---

**Dernière mise à jour** : 7 Décembre 2025  
**Version** : RustOS v1.2.0
