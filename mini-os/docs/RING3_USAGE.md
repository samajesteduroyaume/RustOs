# Guide d'utilisation du Mode Utilisateur (Ring 3)

## Intégration dans main.rs

Pour utiliser Ring 3 dans le noyau, vous devez :

### 1. Initialiser Ring 3 au démarrage

```rust
// Dans main.rs ou lib.rs
use mini_os::ring3::{Ring3Manager, Ring3Context};
use mini_os::ring3_memory::{MemoryIsolationManager};

// Initialiser Ring 3
let ring3_mgr = &*mini_os::ring3::RING3_MANAGER;
ring3_mgr.load();

// Initialiser l'isolation mémoire
let mem_isolation = MemoryIsolationManager::new();
```

### 2. Créer un processus utilisateur

```rust
use mini_os::process::{ProcessManager, ProcessContext};

let mut pm = ProcessManager::new();

// Créer un processus utilisateur
let pid = pm.create_process("user_app", user_entry_point, 1)?;

// Configurer le processus pour Ring 3
let process = pm.get_process(pid)?;
let mut ctx = process.lock().context.clone();
ctx.privilege_level = 3;  // Ring 3
ctx.user_rsp = 0x7FFFFFFFF000;  // Pile utilisateur
process.lock().context = ctx;
```

### 3. Lancer le processus en Ring 3

```rust
// Lancer le processus en Ring 3
let process = pm.get_process(pid)?;
process.lock().execute_in_ring3();
```

## Exemple complet : Programme utilisateur simple

### Créer un programme utilisateur

```rust
// user_app.rs
#![no_std]
#![no_main]

extern crate alloc;

use mini_os::ring3_example::{syscall_write, syscall_exit};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Afficher un message
    let message = b"Hello from Ring 3!\n";
    syscall_write(1, message);
    
    // Terminer le processus
    syscall_exit(0);
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
```

### Charger et exécuter le programme

```rust
// Dans main.rs
use mini_os::process::ProcessManager;

let mut pm = ProcessManager::new();

// Charger le programme ELF
let elf_data = include_bytes!("../user_app.elf");
let pid = pm.create_process_from_elf("user_app", elf_data)?;

// Configurer pour Ring 3
let process = pm.get_process(pid)?;
let mut ctx = process.lock().context.clone();
ctx.privilege_level = 3;
ctx.user_rsp = 0x7FFFFFFFF000;
process.lock().context = ctx;

// Exécuter
process.lock().execute_in_ring3();
```

## Gestion des syscalls

### Implémenter un gestionnaire de syscall

```rust
// Dans syscall/mod.rs
use mini_os::ring3_memory::MemoryIsolationManager;

pub struct SyscallHandler {
    mem_isolation: MemoryIsolationManager,
}

impl SyscallHandler {
    pub fn handle(&self, num: u64, args: &[u64]) -> SyscallResult {
        match num {
            1 => self.handle_write(args[0], args[1] as *const u8, args[2]),
            60 => self.handle_exit(args[0] as i32),
            _ => SyscallResult::Error(SyscallError::InvalidSyscall),
        }
    }
    
    fn handle_write(&self, fd: u64, buf: *const u8, count: u64) -> SyscallResult {
        // Valider l'accès mémoire
        let addr = VirtAddr::new(buf as u64);
        self.mem_isolation.validate_ring3_access(addr, count as usize, false)?;
        
        // Effectuer l'opération
        // ...
        
        SyscallResult::Success(count)
    }
}
```

### Gérer les interruptions Ring 3

```rust
// Dans interrupts.rs
use x86_64::structures::idt::InterruptStackFrame;

extern "x86-interrupt" fn syscall_handler(stack_frame: InterruptStackFrame) {
    // Récupérer les arguments du syscall
    let syscall_num = stack_frame.rax;
    let arg1 = stack_frame.rdi;
    let arg2 = stack_frame.rsi;
    let arg3 = stack_frame.rdx;
    
    // Appeler le gestionnaire
    let handler = SyscallHandler::new();
    let result = handler.handle(syscall_num, &[arg1, arg2, arg3]);
    
    // Retourner le résultat dans RAX
    // ...
}
```

## Isolation mémoire

### Valider un accès mémoire

```rust
use mini_os::ring3_memory::MemoryIsolationManager;
use x86_64::VirtAddr;

let mem_isolation = MemoryIsolationManager::new();

// Valider une lecture
mem_isolation.validate_ring3_access(
    VirtAddr::new(0x400000),
    1024,
    false,  // lecture
)?;

// Valider une écriture
mem_isolation.validate_ring3_access(
    VirtAddr::new(0x500000),
    512,
    true,   // écriture
)?;
```

### Allouer de la mémoire utilisateur

```rust
use mini_os::ring3_memory::UserAddressSpace;
use x86_64::VirtAddr;

let mut user_space = UserAddressSpace::new(
    VirtAddr::new(0x400000),
    0x7FFFFFFFF000 - 0x400000,
);

// Allouer une page
user_space.allocate_page(VirtAddr::new(0x400000))?;

// Vérifier si une adresse est valide
if user_space.is_valid_address(VirtAddr::new(0x500000)) {
    println!("Adresse valide");
}
```

## Contexte d'exécution Ring 3

### Créer un contexte Ring 3

```rust
use mini_os::ring3::Ring3Context;

let context = Ring3Context::new(
    0x400000,           // Point d'entrée
    0x7FFFFFFFF000,     // Pile utilisateur
);

// Configurer les registres
let mut context = context;
context.registers[0] = 0;  // RAX
context.registers[1] = 0;  // RBX
// ...
```

### Basculer vers Ring 3

```rust
use mini_os::ring3::switch_to_ring3;

let ring3_mgr = &*mini_os::ring3::RING3_MANAGER;
let selectors = ring3_mgr.selectors();

unsafe {
    switch_to_ring3(
        &context,
        selectors.user_code,
        selectors.user_data,
    );
}
```

## Débogage

### Afficher les informations Ring 3

```rust
println!("Ring 3 Configuration:");
println!("  User Code Selector: 0x{:x}", ring3_mgr.selectors().user_code);
println!("  User Data Selector: 0x{:x}", ring3_mgr.selectors().user_data);
println!("  User Stack: 0x{:x}", context.user_rsp);
println!("  Entry Point: 0x{:x}", context.rip);
```

### Tracer les syscalls

```rust
// Dans le gestionnaire de syscall
println!("Syscall #{} from Ring 3", syscall_num);
println!("  arg1: 0x{:x}", arg1);
println!("  arg2: 0x{:x}", arg2);
println!("  arg3: 0x{:x}", arg3);
```

## Optimisations

### Utiliser SYSRET au lieu de IRET

Pour améliorer les performances, utilisez SYSRET au lieu de IRET :

```rust
// Utiliser SYSRET pour retourner vers Ring 3
unsafe {
    core::arch::asm!(
        "sysretq",
        in("rcx") context.rip,      // RIP de retour
        in("r11") context.rflags,   // RFLAGS
        options(noreturn)
    );
}
```

### Implémenter le cache TLB

```rust
// Invalider le TLB après un changement de contexte
unsafe {
    x86_64::instructions::tlb::flush_all();
}
```

## Dépannage

### Erreur : "Process not configured for Ring 3 execution"

Assurez-vous que `privilege_level` est défini à 3 :

```rust
ctx.privilege_level = 3;
```

### Erreur : "Access outside user address space"

Vérifiez que l'adresse est dans l'espace utilisateur (0x400000 - 0x7FFFFFFFF000) :

```rust
if user_space.is_valid_address(addr) {
    // OK
}
```

### Erreur : "Syscall not supported"

Implémentez le gestionnaire de syscall pour le numéro spécifié :

```rust
match syscall_num {
    1 => self.handle_write(...),
    // ...
    _ => SyscallResult::Error(SyscallError::InvalidSyscall),
}
```

## Ressources

- `src/ring3.rs` : Gestion des segments et changement de contexte
- `src/ring3_memory.rs` : Isolation mémoire
- `src/ring3_example.rs` : Exemples de programmes utilisateur
- `RING3_SETUP.md` : Documentation complète
- `RING3_IMPLEMENTATION.md` : Détails d'implémentation
