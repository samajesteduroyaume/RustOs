# Changelog

Tous les changements notables de ce projet sont documentés dans ce fichier.

Le format est basé sur [Keep a Changelog](https://keepachangelog.com/fr/1.0.0/),
et ce projet adhère au [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.4.0] - Non publié (En Développement)

### 🚀 Nouvelles Fonctionnalités
- **Système de Fichiers**
  - Support complet d'EXT3/EXT4 avec journalisation
  - Implémentation d'un système de cache avancé avec write-back
  - Support de la lecture/écriture asynchrone

### 🛠️ Améliorations
- **Performances**
  - Optimisation du cache disque avec prélecture (readahead)
  - Réduction de la latence des opérations E/S disque
- **Réseau**
  - Amélioration de la stabilité du pilote RTL8139
  - Optimisation du traitement des paquets réseau

### 🐛 Corrections de Bugs
- Correction d'une condition de course dans le gestionnaire de fichiers
- Résolution d'un problème de fuite mémoire dans le gestionnaire de processus
- Correction de la gestion des interruptions matérielles sur les cœurs secondaires

---

## [1.3.0] - 2025-12-08

### 🚀 Fonctionnalités
- **Symmetric Multi-Processing (SMP)**
  - Détection multicœur automatique via tables ACPI MADT
  - Initialisation et réveil des cœurs secondaires (APs) via séquence SIPI
  - Scheduler distribué avec support du multitâche sur N cœurs
- **Gestion Avancée de l'Énergie**
  - Extinction ACPI S5 avec fallback QEMU
  - Redémarrage via contrôleur clavier ou Triple Fault
  - Boucle Idle utilisant l'instruction `hlt` pour économiser l'énergie
- **Pilotes Matériels**
  - Support initial pour les contrôleurs USB (UHCI/EHCI/XHCI)
  - Pilote Bluetooth HCI de base
  - Support amélioré pour les disques NVMe

### 🔧 Améliorations Techniques
- **Scheduler** : Refactorisation vers une architecture *stateless* compatible SMP
- **Interruptions** : Centralisation de la gestion EOI (End of Interrupt) pour l'APIC
- **Mémoire** : Optimisation de l'allocateur de mémoire pour les systèmes multicœurs

---

## [1.2.0] - 2025-12-07

### ✨ Nouveautés
- **Virtual File System (VFS)**
  - Système de fichiers virtuel complet (abstraction POSIX-like)
  - Support natif : `open`, `read`, `write`, `mkdir`, `dentry cache`
- **RamFS**
  - Système de fichiers haute performance en mémoire (monté sur `/`)
- **Support Ext2/3/4**
  - Driver de lecture pour partitions Linux Ext2/3/4
  - **Auto-Mount** : Montage automatique de la première partition détectée au boot
- **Loader ELF 64-bit**
  - Chargement dynamique et exécution de binaires utilisateurs

### ⚡ Optimisations
- Réduction de 15% de l'utilisation de la RAM pour les buffers de fichiers
- Amélioration des performances d'E/S disque avec un cache optimisé

---

## [1.1.0] - 2025-12-06

### 🔌 Matériel & Drivers
- **Détection Hardware** : Scan PCI récursif et parsing ACPI
- **Architecture Plug & Play** : Structure pour futurs drivers USB (UHCI/EHCI/XHCI) et Bluetooth
- **Réseau** : Pilote stable pour cartes Realtek RTL8139 avec support TCP/IP complet
- **Stockage** : Support de base pour les contrôleurs ATA/SATA et NVMe
- **Affichage** : Pilotes VESA et VGA avec support du mode texte et graphique

### 🐚 Interface (Shell)
- Nouvelles commandes : `devices`, `netstat`, `cat`, `ls -l`.
- Amélioration de l'UX : Historique persistant, complétion, variables d'environnement.

---

## [1.0.0] - 2025-11-20

### 🌐 Réseau & Noyau
- **Stack TCP/IP Complète (IPv4)** : Support TCP, UDP, ICMP, ARP.
- **Services Réseau** : Client DHCP et Résolveur DNS.
- **Stockage** : Driver ATA/SATA PIO haute compatibilité.
- **Mémoire** : Nouvel allocateur hybride (Slab + Buddy System).

---

## [0.2.0] - 2025-11-10

### 🧠 Cœur du Système
- **Multitasking** : Premier scheduler préemptif (CFS).
- **Isolation** : Séparation stricte Kernel (Ring 0) / User (Ring 3).
- **Syscalls** : API système de base pour les programmes utilisateurs.

---

## [0.1.0] - 2025-11-01

### 🌱 Genèse
- **Bootloader** : Prise en charge Multiboot2.
- **Foundation** : GDT, IDT, et sortie VGA fonctionnels.
- **Hello World** : Premier démarrage en mode Long (64-bit).
