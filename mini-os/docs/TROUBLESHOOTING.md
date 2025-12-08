# RustOS Troubleshooting Guide

Ce guide recense les erreurs courantes rencontrées lors du développement de systèmes d'exploitation en Rust, avec des solutions spécifiques pour Limine et le noyau RustOS.

## 1. Erreurs de Bootloader (Limine)

| Erreur | Cause Possible | Solution Typique |
|--------|----------------|------------------|
| `Stage X file not found` | Fichiers du bootloader manquants ou mal placés. | Copier `limine-bios.sys` et `limine.cfg` dans `/boot/limine/` (ou racine). Vérifier le script de build. |
| `Failed to load stage 2/3` | Corruption du bootloader, problème MBR/UEFI. | Réinstaller Limine avec `limine bios-install --force-mbr` sur les images brutes. |
| `No bootable device` | Disque non reconnu ou bootloader non installé. | Vérifier l'ordre de boot QEMU/BIOS. Réinstaller le bootloader. |
| `Invalid partition table` | Table de partitions corrompue. | Utiliser un outil de partitionnement ou `--force-mbr` pour les images sans partition. |

## 2. Erreurs du Noyau (Kernel)

| Erreur | Cause Possible | Solution Typique |
|--------|----------------|------------------|
| `Kernel panic - not syncing` | Erreur critique (ex: `PageAlreadyMapped`). | Vérifier les logs. Pour `PageAlreadyMapped`, s'assurer que le mappage mémoire est idempotent. |
| `GPF (General Protection Fault)` | Accès mémoire invalide. | Déboguer avec QEMU (`-s -S`) et GDB. Vérifier les pointeurs. |
| `PageAlreadyMapped` | Tentative de re-mapper une page déjà active (ex: Limine FB). | Modifier `map_page` pour ignorer l'erreur si le mappage existe déjà. |

## 3. Erreurs Rust (No_std)

| Erreur | Cause Possible | Solution Typique |
|--------|----------------|------------------|
| `Segmentation fault` | Accès invalide dans bloc `unsafe`. | Vérifier l'initialisation des structures (ex: `SerialPort`). |
| `failed to allocate memory` | Allocateur (Heap) non initialisé. | Appeler `allocator.init()` au début du `_start`. |
| `Undefined reference to memcpy` | Manque `compiler_builtins` ou `rlibc`. | Ajouter `compiler_builtins` ou activer la feature `compiler-builtins-mem`. |

## 4. Erreurs de Système de Fichiers (VFS)

| Erreur | Cause Possible | Solution Typique |
|--------|----------------|------------------|
| `EXT4-fs: unable to read superblock` | Partition non formatée ou corrompue. | Vérifier `mkfs.ext2` dans le script de build. |
| `No such file or directory` | Fichier manquant dans l'image. | Vérifier l'étape d'injection `debugfs` dans le script de build. |

## 5. Astuces Débogage

- **QEMU Serial** : Les logs du noyau sont redirigés vers le port série. Lancez QEMU avec `-serial stdio` pour les voir dans le terminal.
- **DebugFS** : Utilisez `debugfs -R "ls -l /boot" rustos_limine.img` pour inspecter le contenu de l'image disque sans la monter.
