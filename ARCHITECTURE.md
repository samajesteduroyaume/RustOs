# Architecture Complète de RustOS

## Vue d'ensemble

RustOS est un système d'exploitation minimal écrit en Rust, implémentant un noyau multitâche avec support de la mémoire virtuelle, de la synchronisation et du système de fichiers UFAT.

## Diagramme d'Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Applications                              │
├─────────────────────────────────────────────────────────────────┤
│                      Appels Système (Syscall)                    │
├─────────────────────────────────────────────────────────────────┤
│                           Noyau RustOS                           │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                  Gestion des Processus                   │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌──────────────────┐ │   │
│  │  │ ProcessMgr  │  │  Scheduler  │  │  Synchronisation │ │   │
│  │  │             │  │             │  │  (Mutex, Sem)    │ │   │
│  │  └─────────────┘  └─────────────┘  └──────────────────┘ │   │
│  └──────────────────────────────────────────────────────────┘   │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                  Gestion de la Mémoire                   │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐   │   │
│  │  │ VMManager    │  │ AddressSpace │  │ CowManager   │   │   │
│  │  │              │  │              │  │              │   │   │
│  │  └──────────────┘  └──────────────┘  └──────────────┘   │   │
│  └──────────────────────────────────────────────────────────┘   │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                  Système de Fichiers                     │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐   │   │
│  │  │ UFAT         │  │ FileDescMgr  │  │ Filesystem   │   │   │
│  │  │              │  │              │  │              │   │   │
│  │  └──────────────┘  └──────────────┘  └──────────────┘   │   │
│  └──────────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────────┤
│                      Gestionnaire d'Interruptions                │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │ Timer (32)   │  │ Keyboard (33)│  │ PageFault(14)│          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
├─────────────────────────────────────────────────────────────────┤
│                      Pilotes Matériel                            │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │ VGA          │  │ Clavier      │  │ Stockage     │          │
│  │              │  │              │  │              │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
├─────────────────────────────────────────────────────────────────┤
│                      Matériel (x86-64)                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │ CPU          │  │ Mémoire      │  │ Disque       │          │
│  │              │  │              │  │              │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
└─────────────────────────────────────────────────────────────────┘
```

## Composants Principaux

### 1. Gestion des Processus

#### ProcessManager
- Crée et gère les processus
- Maintient une liste des processus actifs
- Génère les PID uniques
- Gère les transitions d'état

#### Process
- Représente un processus en cours d'exécution
- Contient le contexte d'exécution
- Maintient l'espace d'adressage
- Gère les pages CoW

#### ProcessContext
- Sauvegarde les registres (RSP, RIP, etc.)
- Maintient la table des pages
- Stocke les registres généraux

### 2. Planificateur

#### Scheduler
- Sélectionne le prochain processus à exécuter
- Implémente plusieurs politiques (RoundRobin, Priority, FIFO)
- Gère le quantum (temps alloué par processus)
- Effectue le changement de contexte

#### Politiques de Planification
- **RoundRobin** : Chaque processus reçoit un quantum égal
- **Priority** : Les processus avec priorité plus haute s'exécutent d'abord
- **FIFO** : Premier arrivé, premier servi

### 3. Gestion de la Mémoire

#### VMManager
- Gère les espaces d'adressage globaux
- Crée les espaces d'adressage pour les processus
- Commute entre les espaces d'adressage

#### AddressSpace
- Représente l'espace d'adressage d'un processus
- Mappe les pages virtuelles aux cadres physiques
- Gère les permissions d'accès

#### FrameAllocator
- Alloue les cadres physiques
- Suit les cadres utilisés
- Libère les cadres inutilisés

#### CowManager
- Gère la copie sur écriture
- Suit les pages partagées avec compteur de références
- Gère les défauts de page de protection
- Duplique les pages lors de l'écriture

### 4. Synchronisation

#### Semaphore
- Compteur atomique
- Opérations wait/signal
- Queue de processus en attente

#### MutexLock
- Exclusion mutuelle
- Propriétaire du verrou
- Queue de processus en attente

#### ConditionVariable
- Synchronisation basée sur une condition
- Opérations wait/signal/broadcast
- Intégration avec les mutex

#### Barrier
- Synchronisation de groupe
- Attend que tous les processus arrivent
- Réveille tous les processus simultanément

### 5. Système de Fichiers

#### UFAT
- Système de fichiers unifié (FAT32 + EXT2)
- Superbloc avec métadonnées
- Groupes de blocs
- Bitmaps d'inodes et de blocs
- Inodes pour les fichiers et répertoires

#### FileDescriptorManager
- Gère les tables de descripteurs par processus
- Crée et supprime les tables
- Fournit l'accès aux descripteurs

#### FileDescriptorTable
- Table de descripteurs pour un processus
- Opérations open/close/dup2
- Modes d'ouverture (ReadOnly, WriteOnly, ReadWrite)

### 6. Appels Système

#### SyscallHandler
- Traite les appels système
- Valide les arguments
- Retourne les résultats

#### Appels Système Supportés
- Fork : Créer un processus enfant
- Exit : Terminer le processus
- Read : Lire depuis un fichier
- Write : Écrire dans un fichier
- Open : Ouvrir un fichier
- Close : Fermer un fichier
- Exec : Exécuter un programme
- Wait : Attendre la fin d'un processus
- GetPid : Obtenir le PID du processus

### 7. Gestionnaire d'Interruptions

#### Interruptions Gérées
- **Timer (32)** : Tick du planificateur
- **Keyboard (33)** : Entrée clavier
- **Page Fault (14)** : Défaut de page

#### Gestionnaire de Défaut de Page
- Détecte les violations de protection
- Gère la copie sur écriture
- Signale les erreurs non gérées

## Flux d'Exécution

### Démarrage du Système

```
1. Bootloader charge le noyau
2. Initialisation du tas (heap)
3. Initialisation de la table des interruptions (IDT)
4. Activation des interruptions
5. Création du gestionnaire de processus
6. Création du processus initial (init)
7. Initialisation du planificateur
8. Démarrage du planificateur (boucle infinie)
```

### Changement de Contexte

```
1. Interruption timer
2. Appel du gestionnaire d'interruption
3. Sauvegarde du contexte du processus actuel
4. Sélection du prochain processus
5. Restauration du contexte du nouveau processus
6. Retour du contrôle au nouveau processus
```

### Création de Processus (Fork)

```
1. Appel système fork()
2. Création d'un nouvel espace d'adressage
3. Copie des mappages de page (avec CoW)
4. Marquage des pages comme lecture seule
5. Retour du PID enfant au parent (0 à l'enfant)
```

### Gestion de Défaut de Page

```
1. Tentative d'accès à une page protégée
2. Exception de page
3. Appel du gestionnaire de défaut de page
4. Vérification si c'est une page CoW
5. Duplication de la page si nécessaire
6. Retour au processus
```

## Structures de Données Principales

### Process
```rust
pub struct Process {
    pub pid: u64,
    pub name: String,
    pub state: ProcessState,
    pub context: ProcessContext,
    pub priority: u8,
    pub kstack: Option<PhysAddr>,
    pub address_space_id: usize,
    pub cow_pages: Vec<PhysFrame>,
}
```

### ProcessContext
```rust
pub struct ProcessContext {
    pub rsp: u64,
    pub rip: u64,
    pub registers: [u64; 16],
    pub page_table: Arc<Mutex<PageTable>>,
}
```

### FileDescriptor
```rust
pub struct FileDescriptor {
    pub fd: usize,
    pub path: String,
    pub mode: OpenMode,
    pub offset: u64,
    pub size: u64,
}
```

### UfatSuperBlock
```rust
pub struct UfatSuperBlock {
    pub magic: u32,
    pub version: u32,
    pub block_size: u32,
    pub block_count: u64,
    pub free_blocks: u64,
    pub inode_count: u64,
    pub free_inodes: u64,
    pub first_data_block: u32,
    pub inodes_per_group: u32,
    pub blocks_per_group: u32,
    pub volume_name: [u8; 32],
    pub last_mount: u32,
    pub last_write: u32,
    pub mount_count: u16,
    pub max_mounts: u16,
    pub checksum: u32,
    pub reserved: [u8; 448],
}
```

## Interactions Entre Composants

### Création de Processus
```
Application → Syscall (fork) → SyscallHandler → ProcessManager → 
VMManager → AddressSpace → CowManager → Process
```

### Changement de Contexte
```
Timer Interrupt → IDT → Scheduler → ProcessManager → 
Process.save_context() → Process.restore_context()
```

### Accès aux Fichiers
```
Application → Syscall (open/read/write) → SyscallHandler → 
FileDescriptorManager → FileDescriptorTable → UFAT
```

### Gestion de Défaut de Page
```
CPU → Page Fault Exception → IDT → page_fault_handler → 
CowManager → FrameAllocator → VMManager
```

## Sécurité et Isolation

### Isolation de la Mémoire
- Chaque processus a son propre espace d'adressage
- Les pages sont protégées par les permissions du MMU
- La copie sur écriture assure l'isolation des données

### Contrôle d'Accès
- Les descripteurs de fichiers sont isolés par processus
- Les permissions de fichiers sont vérifiées
- Les appels système valident les arguments

### Synchronisation
- Les mutex assurent l'exclusion mutuelle
- Les sémaphores permettent la coordination
- Les variables de condition permettent l'attente

## Performance

### Optimisations
- Allocation rapide de cadres physiques (O(1))
- Changement de contexte rapide (O(1))
- Copie sur écriture pour économiser la mémoire
- Cache TLB pour les traductions d'adresses

### Limitations Actuelles
- Pas de pagination sur demande
- Pas de swap de mémoire
- Pas de cache de disque
- Pas de planification avec priorité dynamique

## Extensibilité

### Points d'Extension
- Nouvelles politiques de planification
- Nouveaux appels système
- Nouveaux pilotes matériel
- Nouvelles primitives de synchronisation

### Modularité
- Chaque composant est indépendant
- Les interfaces sont bien définies
- Les dépendances sont minimales

## Limitations Connues

1. **Pas de pagination sur demande** : Toute la mémoire doit être allouée à l'avance
2. **Pas de swap** : Pas de gestion de la mémoire virtuelle sur disque
3. **Pas de niveaux de privilège** : Tous les processus s'exécutent en ring 0
4. **Pas de signaux** : Pas de gestion des signaux POSIX
5. **Pas de communication inter-processus** : Pas de pipes, sockets, etc.

## Prochaines Améliorations

1. Implémenter la pagination sur demande
2. Ajouter le support pour les niveaux de privilège
3. Implémenter la gestion des signaux
4. Ajouter la communication inter-processus
5. Optimiser la performance avec le cache TLB

---

**Dernière mise à jour** : 6 Décembre 2025
**Auteur** : Assistant IA Cascade
