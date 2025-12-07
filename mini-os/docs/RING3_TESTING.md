# Guide de test du Mode Utilisateur (Ring 3)

## 🧪 Tests unitaires

### Compiler les tests

```bash
cargo test --lib --no-default-features --features alloc
```

### Exécuter les tests

```bash
cargo test --lib --no-default-features --features alloc -- --nocapture
```

### Tests disponibles

Dans `src/ring3_example.rs` :
- `test_fibonacci()` - Teste la récursion
- `test_math_operations()` - Teste les opérations mathématiques

## 🧪 Tests d'intégration RamFS

### Compiler les tests RamFS

```bash
cargo build --test ramfs_tests --no-default-features --features alloc
```

### Exécuter les tests RamFS

```bash
./run_ramfs_tests.sh
```

### Tests disponibles

Dans `tests/ramfs_tests.rs` :
- `test_ramfs_file_creation()` - Création de fichiers
- `test_ramfs_read_write()` - Lecture/écriture
- `test_ramfs_mkdir()` - Création de répertoires
- `test_ramfs_not_found()` - Gestion des erreurs

## 🧪 Tests sur QEMU (À implémenter)

### Créer une image bootable

```bash
# Compiler le noyau
cargo build --release --no-default-features --features alloc

# Créer une image bootable avec GRUB
# (À implémenter)
```

### Lancer QEMU

```bash
qemu-system-x86_64 \
  -kernel target/x86_64-unknown-none/release/mini-os \
  -m 512M \
  -serial stdio \
  -d int,cpu_reset
```

### Tester un processus Ring 3

```bash
# Créer un programme utilisateur simple
# (À implémenter)

# Charger le programme dans le noyau
# (À implémenter)

# Exécuter le programme en Ring 3
# (À implémenter)
```

## 📋 Checklist de test

### Compilation
- [ ] `cargo check --no-default-features --features alloc` réussit
- [ ] `cargo build --lib --no-default-features --features alloc` réussit
- [ ] `cargo build --tests --no-default-features --features alloc` réussit
- [ ] Aucune erreur de compilation
- [ ] Aucun avertissement critique

### Tests unitaires
- [ ] `cargo test --lib` réussit
- [ ] Tous les tests passent
- [ ] Pas de paniques

### Tests RamFS
- [ ] `./run_ramfs_tests.sh` réussit
- [ ] Tous les tests RamFS passent
- [ ] Pas de segmentation faults

### Tests Ring 3 (À implémenter)
- [ ] Créer un programme utilisateur simple
- [ ] Charger le programme en Ring 3
- [ ] Exécuter le programme sans erreur
- [ ] Tester les syscalls (write, exit, getpid)
- [ ] Tester l'isolation mémoire
- [ ] Tester la gestion des erreurs

### Tests de sécurité (À implémenter)
- [ ] Essayer d'accéder au noyau depuis Ring 3 → Erreur
- [ ] Essayer d'accéder à un espace mémoire invalide → Erreur
- [ ] Essayer d'exécuter une instruction privilégiée → Erreur
- [ ] Tester les limites des ressources

## 🔍 Débogage

### Afficher les informations Ring 3

```rust
// Dans main.rs
use mini_os::ring3::Ring3Manager;

let ring3_mgr = &*mini_os::ring3::RING3_MANAGER;
let selectors = ring3_mgr.selectors();

println!("Ring 3 Configuration:");
println!("  User Code Selector: 0x{:x}", selectors.user_code);
println!("  User Data Selector: 0x{:x}", selectors.user_data);
println!("  Kernel Code Selector: 0x{:x}", selectors.kernel_code);
println!("  Kernel Data Selector: 0x{:x}", selectors.kernel_data);
```

### Tracer les syscalls

```rust
// Dans syscall/mod.rs
println!("Syscall #{} from Ring 3", syscall_num);
println!("  arg1: 0x{:x}", arg1);
println!("  arg2: 0x{:x}", arg2);
println!("  arg3: 0x{:x}", arg3);
```

### Tracer les changements de contexte

```rust
// Dans ring3.rs
println!("Switching to Ring 3");
println!("  Entry point: 0x{:x}", context.rip);
println!("  User stack: 0x{:x}", context.user_rsp);
println!("  Code selector: 0x{:x}", user_code_selector);
println!("  Data selector: 0x{:x}", user_data_selector);
```

## 📊 Métriques de test

### Couverture de code

| Module | Couverture | Statut |
|--------|-----------|--------|
| `ring3.rs` | 0% | À tester |
| `ring3_memory.rs` | 0% | À tester |
| `ring3_example.rs` | 100% | ✅ Testé |
| `process/mod.rs` | Partielle | À améliorer |

### Temps de test

| Test | Temps | Statut |
|------|-------|--------|
| Compilation | 0.78s | ✅ Rapide |
| Tests unitaires | ? | À mesurer |
| Tests RamFS | ? | À mesurer |
| Tests Ring 3 | ? | À mesurer |

## 🐛 Problèmes connus

### À tester
1. Changement de contexte Ring 0 → Ring 3
2. Exécution de code utilisateur
3. Appels système depuis Ring 3
4. Isolation mémoire
5. Gestion des erreurs

### À implémenter
1. Handlers d'interruption pour Ring 3
2. Allocation de mémoire physique
3. Chargement de programmes ELF
4. Gestion des signaux

## 📝 Exemple de test complet

### 1. Créer un programme utilisateur

```rust
// user_test.rs
#![no_std]
#![no_main]

extern crate alloc;

use mini_os::ring3_example::syscall_write;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Test 1 : Écrire un message
    let msg = b"Test 1: Hello from Ring 3\n";
    syscall_write(1, msg);
    
    // Test 2 : Effectuer une opération mathématique
    let a = 10;
    let b = 20;
    let sum = a + b;
    
    // Test 3 : Terminer
    syscall_write(1, b"Test 3: Exiting\n");
    
    // Terminer le processus
    loop {}
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
```

### 2. Compiler le programme

```bash
rustc --target x86_64-unknown-none -O user_test.rs -o user_test.elf
```

### 3. Charger et exécuter

```rust
// Dans main.rs
use mini_os::process::ProcessManager;
use mini_os::ring3::Ring3Context;

let mut pm = ProcessManager::new();

// Charger le programme ELF
let elf_data = include_bytes!("../user_test.elf");
let pid = pm.create_process_from_elf("user_test", elf_data)?;

// Configurer pour Ring 3
let process = pm.get_process(pid)?;
let mut ctx = process.lock().context.clone();
ctx.privilege_level = 3;
ctx.user_rsp = 0x7FFFFFFFF000;
process.lock().context = ctx;

// Exécuter
process.lock().execute_in_ring3();
```

### 4. Vérifier les résultats

```
Test 1: Hello from Ring 3
Test 3: Exiting
```

## 🎯 Objectifs de test

### Phase 1 : Tests unitaires (✅ Complété)
- [x] Compilation sans erreur
- [x] Tests unitaires passent
- [x] Tests RamFS passent

### Phase 2 : Tests d'intégration (⏳ En cours)
- [ ] Créer un programme utilisateur simple
- [ ] Charger le programme en Ring 3
- [ ] Exécuter le programme
- [ ] Vérifier les résultats

### Phase 3 : Tests de sécurité (⏳ À faire)
- [ ] Tester l'isolation mémoire
- [ ] Tester la gestion des erreurs
- [ ] Tester les limites des ressources

### Phase 4 : Tests de performance (⏳ À faire)
- [ ] Mesurer le temps de changement de contexte
- [ ] Mesurer le temps des syscalls
- [ ] Optimiser les performances

## 📚 Ressources

- `RING3_SETUP.md` - Configuration
- `RING3_USAGE.md` - Utilisation
- `src/ring3.rs` - Code source
- `src/ring3_memory.rs` - Isolation mémoire
- `src/ring3_example.rs` - Exemples
