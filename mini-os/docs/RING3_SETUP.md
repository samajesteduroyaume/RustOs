# Configuration du Mode Utilisateur (Ring 3)

## Vue d'ensemble

Ce document explique comment configurer et utiliser le Mode Utilisateur (Ring 3) dans mini-os pour exécuter des processus en sécurité avec isolation mémoire.

## Architecture

### Niveaux de privilège x86-64

- **Ring 0** : Noyau (accès complet au matériel)
- **Ring 1, 2** : Non utilisés dans mini-os
- **Ring 3** : Utilisateur (accès restreint, isolation mémoire)

### Modules principaux

#### 1. `ring3.rs` - Gestion des segments et changement de contexte

Ce module fournit :
- **GDT (Global Descriptor Table)** : Configuration des segments Ring 0 et Ring 3
- **TSS (Task State Segment)** : Gestion des piles d'interruption
- **Ring3Manager** : Gestionnaire global pour Ring 3
- **Ring3Context** : Structure pour représenter le contexte d'exécution Ring 3

**Utilisation** :
```rust
use mini_os::ring3::{Ring3Manager, Ring3Context};

// Initialiser Ring 3
let ring3_mgr = &*mini_os::ring3::RING3_MANAGER;
ring3_mgr.load();

// Créer un contexte Ring 3
let context = Ring3Context::new(
    0x400000,  // Point d'entrée (adresse virtuelle)
    0x7FFFFFFFF000,  // Pile utilisateur
);

// Basculer vers Ring 3
unsafe {
    mini_os::ring3::switch_to_ring3(
        &context,
        ring3_mgr.indices().user_code,
        ring3_mgr.indices().user_data,
    );
}
```

#### 2. `ring3_memory.rs` - Isolation mémoire

Ce module fournit :
- **UserAddressSpace** : Espace d'adressage isolé pour chaque processus
- **MemoryIsolation** : Configuration de l'isolation mémoire
- **MemoryIsolationManager** : Gestionnaire global pour l'isolation

**Utilisation** :
```rust
use mini_os::ring3_memory::{UserAddressSpace, MemoryIsolation};
use x86_64::VirtAddr;

// Créer un espace d'adressage utilisateur
let mut user_space = UserAddressSpace::new(
    VirtAddr::new(0x400000),  // Base
    0x7FFFFFFFF000 - 0x400000,  // Taille
);

// Allouer une page
user_space.allocate_page(VirtAddr::new(0x400000))?;

// Valider un accès mémoire
if user_space.is_valid_address(VirtAddr::new(0x500000)) {
    println!("Adresse valide");
}
```

#### 3. `ring3_example.rs` - Exemples de programmes utilisateur

Ce module contient des exemples de programmes qui s'exécutent en Ring 3 :
- `user_program_hello()` : Affiche "Hello from Ring 3!"
- `user_program_math()` : Effectue des opérations mathématiques
- `user_program_fibonacci()` : Calcule la suite de Fibonacci

**Syscalls disponibles** :
- `syscall_write(fd, buf)` : Écrire vers un descripteur de fichier
- `syscall_exit(status)` : Terminer le processus
- `syscall_getpid()` : Obtenir le PID du processus

## Flux d'exécution

### 1. Initialisation (Ring 0)

```
main.rs
  ↓
Initialiser la GDT avec Ring 3 (ring3.rs)
  ↓
Initialiser l'isolation mémoire (ring3_memory.rs)
  ↓
Créer un processus utilisateur (process.rs)
```

### 2. Lancement d'un processus (Ring 0 → Ring 3)

```
ProcessManager::create_process()
  ↓
Allouer l'espace d'adressage utilisateur
  ↓
Charger le code utilisateur (ELF)
  ↓
Créer un contexte Ring 3
  ↓
Appeler switch_to_ring3()
  ↓
Exécution en Ring 3
```

### 3. Syscall (Ring 3 → Ring 0)

```
Programme utilisateur
  ↓
Instruction SYSCALL
  ↓
Interruption (Ring 3 → Ring 0)
  ↓
Gestionnaire de syscall (syscall/mod.rs)
  ↓
Valider l'accès mémoire (ring3_memory.rs)
  ↓
Exécuter le syscall
  ↓
Retour au programme utilisateur (SYSRET)
```

## Configuration de la GDT

La GDT est configurée avec les segments suivants :

```
Index 0 : Null descriptor (obligatoire)
Index 1 : Kernel Code (Ring 0)
Index 2 : Kernel Data (Ring 0)
Index 3 : User Code (Ring 3)
Index 4 : User Data (Ring 3)
Index 5 : TSS (Task State Segment)
```

## Isolation mémoire

### Espace d'adressage utilisateur (x86-64)

```
0x0000000000000000 ┌─────────────────────┐
                   │   Réservé (noyau)   │
0x0000000000400000 ├─────────────────────┤
                   │  Espace utilisateur │
                   │  (Code + Heap)      │
                   │                     │
0x7FFFFFFFF000    ├─────────────────────┤
                   │  Pile utilisateur   │
                   │  (décroissante)     │
0x7FFFFFFFFFF     └─────────────────────┘
```

### Validation des accès

Tous les accès mémoire depuis Ring 3 sont validés :
- Vérifier que l'adresse est dans l'espace utilisateur
- Vérifier les permissions (lecture/écriture)
- Empêcher les accès au noyau

## Gestion des syscalls

### Numéros de syscall

```rust
pub enum SyscallNumber {
    Exit = 0,
    Fork = 1,
    Read = 2,
    Write = 3,
    Open = 4,
    Close = 5,
    Exec = 6,
    Wait = 7,
    GetPid = 8,
}
```

### Appel d'un syscall depuis Ring 3

```rust
// Appel de syscall via SYSCALL
unsafe {
    core::arch::asm!(
        "syscall",
        in("rax") syscall_number,
        in("rdi") arg1,
        in("rsi") arg2,
        in("rdx") arg3,
        out("rax") result,
    );
}
```

## Prochaines étapes

1. **Implémenter les syscalls** :
   - Valider les accès mémoire utilisateur
   - Implémenter read/write/open/close
   - Implémenter fork/exec

2. **Gérer les interruptions Ring 3** :
   - Configurer les handlers d'interruption
   - Sauvegarder/restaurer le contexte
   - Retourner vers Ring 3

3. **Tester l'exécution Ring 3** :
   - Créer des programmes utilisateur simples
   - Tester les syscalls
   - Tester l'isolation mémoire

4. **Optimiser les performances** :
   - Utiliser SYSRET au lieu de IRET
   - Implémenter le cache TLB
   - Optimiser les changements de contexte

## Références

- [Intel 64 and IA-32 Architectures Software Developer's Manual](https://www.intel.com/content/dam/www/public/us/en/documents/manuals/64-ia-32-architectures-software-developer-manual-combined-volumes.pdf)
- [x86-64 ABI](https://refspecs.linuxbase.org/elf/x86-64-abi-0.99.pdf)
- [Linux Syscall Reference](https://man7.org/linux/man-pages/man2/syscalls.2.html)
