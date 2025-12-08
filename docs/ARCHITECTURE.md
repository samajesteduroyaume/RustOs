# Architecture Technique de RustOS

Ce document décrit l'architecture interne actuelle de RustOS (v1.2.0+). Il est destiné aux développeurs et ingénieurs souhaitant comprendre les mécanismes implémentés.

## 1. Noyau & Multitâche

### Point d'Entrée
- **Boot**: Multiboot2 compliant.
- **Entrée Rust**: `_start` dans `src/main.rs`.
- **Initialisation**:
    1.  VGA Buffer.
    2.  Détection Matérielle (`hardware::detect_*`).
    3.  Allocateur Mémoire (Heap).
    4.  Interruptions (IDT) & GDT.
    5.  Filesystem (VFS + RamFS).
    6.  Drivers & Périphériques.
    7.  SMP Init (AP Boot).
    8.  Scheduler Start.

### Symmetric Multi-Processing (SMP)
- **Détection**: Parsing ACPI MADT pour lister les cœurs (LAPIC IDs).
- **Boot AP**: Utilisation d'un Trampoline 16-bit (`src/smp/trampoline.rs`) copié en mémoire basse (`0x8000`). Signal envoyé via IPI (Inter-Processor Interrupts).
- **Données Per-CPU**: Registre `GS` configuré pour pointer vers une structure `PerCpuData` propre à chaque cœur (contient `lapic_id`, `current_thread`).

### Planificateur (Scheduler)
- **Type**: CFS (Completely Fair Scheduler) simplifié / Round-Robin avec priorités.
- **Entités**: Gère des `Thread`s. Les `Process` sont des conteneurs de ressources.
- **Boucle**:
    - Chaque CPU exécute sa propre boucle de planification.
    - Si aucun thread n'est prêt, exécution de l'instruction `hlt` (Idle Loop).
    - Préemption via Timer Interrupt (LAPIC Timer).

## 2. Gestion de la Mémoire

- **Paging**: Basé sur un `OffsetPageTable`.
- **Allocateur**: `HybridAllocator` combinant :
    - **Bump Allocator** pour les petites allocations rapides au démarrage.
    - **LinkedList Allocator** ou **Slab** pour la gestion dynamique (implémentation dans `src/memory/`).
- **Isolation**: Chaque processus utilisateur (Ring 3) possède son propre Page Directory (CR3), assurant l'isolation mémoire.

## 3. Système de Fichiers (VFS)

L'architecture VFS (Virtual File System) abstrait les détails des systèmes de fichiers concrets.

### Composants
- **Dentry**: Cache des entrées de répertoire (noms -> inodes).
- **Inode**: Métadonnées d'un fichier (taille, permissions, type).
- **FileDescriptor**: Handle ouvert par un processus.

### Implémentations
1.  **RamFS** (Actif) : Système de fichiers en mémoire volatile.
    - *État*: Complètement fonctionnel. Utilisé comme racine `/`.
    - *Support*: Liens symboliques, répertoires, fichiers réguliers.
2.  **Ext2** (Supporté) : Pilote pour lire/écrire des partitions Ext2.
    - *État*: Implémentation présente (`src/ext2.rs`). Tentative de montage automatique de la 1ère partition du disque maître au démarrage (`src/main.rs`).
3.  **UFAT** (Partiel) : Système de fichiers personnalisé.
    - *État*: Support du formatage (`format_ufat`) fonctionnel. Opérations de lecture/écriture en cours d'implémentation (stubs).

## 4. Pile Réseau (`src/net`)

Stack TCP/IP complète implémentée en Rust.

- **Couche Liaison**: Ethernet (trames, MAC).
- **Couche Réseau**: 
    - IPv4 (Adressage, Fragmentation).
    - ARP (Cache, Résolution).
    - ICMP (Ping Echo/Reply).
- **Couche Transport**:
    - UDP (Datagrammes, Sockets).
    - TCP (Machine à états complète : SYN, ACK, FIN, Timers, Retransmissions).
- **Application**:
    - DHCP (Client pour configuration auto).
    - DNS (Résolution de noms avec cache).
    - HTTP (Client basique).

## 5. Gestion des Drivers (`src/drivers`)

### Driver Manager
Un gestionnaire centralisé enregistre et initialise les pilotes détectés.

### Pilotes Matériels
- **PCI**: Énumération des bus/slots/fonctions.
- **Réseau**: Pilote `RTL8139` (Ethernet) fonctionnel.
- **Stockage**: Pilote ATA/SATA PIO pour disques durs.
- **Entrées**: Clavier/Souris PS/2.

### Statut des Pilotes "Avancés"
- **USB** (`src/drivers/usb_*.rs`) : Architecture définie (Status: **Alpha/Structure**).
    - Détection des contrôleurs (UHCI, EHCI, XHCI) implémentée.
    - Logique de transfert et énumération des périphériques : *Non implémentée (Stubs)*.
- **Bluetooth** (`src/drivers/bluetooth_*.rs`) : Architecture définie (Status: **Alpha/Structure**).
    - Structures HCI et L2CAP présentes.
    - Communication matérielle : *Non implémentée*.
- **NVMe**: Structures définies, logique d'initialisation partielle.

## 6. Mode Utilisateur (Ring 3)

- **Syscalls**: Instructions `syscall` / `sysret`.
- **API**: Dispatcher central dans `src/syscall/mod.rs`.
- **Capacités**:
    - Création de processus/threads.
    - Entrées/Sorties (Print console).
    - Opérations Fichiers (Open, Read, Write).
    - Sommeil (Sleep).

---
*Documentation générée automatiquement après audit du code source (07/12/2025).*
