/// Module exemple pour démontrer l'exécution en Ring 3
/// 
/// Ce module contient un exemple simple d'un programme utilisateur
/// qui s'exécute en Ring 3 et utilise les syscalls pour communiquer
/// avec le noyau.

use crate::ring3::{Ring3Context};
use x86_64::VirtAddr;

/// Point d'entrée d'un programme utilisateur simple
/// 
/// Ce programme affiche un message et se termine
pub fn user_program_hello() {
    // Appeler un syscall pour afficher un message
    // syscall(SYS_WRITE, fd=1, buf="Hello from Ring 3\n", count=18)
    
    let message = b"Hello from Ring 3!\n";
    let _result = syscall_write(1, message);
    
    // Appeler un syscall pour se terminer
    // syscall(SYS_EXIT, status=0)
    syscall_exit(0);
}

/// Programme utilisateur qui teste les opérations mathématiques
pub fn user_program_math() {
    // Effectuer quelques opérations mathématiques
    let a: i32 = 10;
    let b: i32 = 20;
    let sum = a + b;
    let product = a * b;
    
    // Afficher les résultats via syscall
    let message = b"Math test completed\n";
    let _result = syscall_write(1, message);
    
    syscall_exit(0);
}

/// Programme utilisateur qui teste la récursion
pub fn user_program_fibonacci(n: u32) -> u32 {
    if n <= 1 {
        n
    } else {
        user_program_fibonacci(n - 1) + user_program_fibonacci(n - 2)
    }
}

/// Syscall : écrire vers un descripteur de fichier
/// 
/// # Arguments
/// * `fd` - Descripteur de fichier (1 = stdout)
/// * `buf` - Buffer à écrire
/// 
/// # Retour
/// Nombre de bytes écrits
pub fn syscall_write(fd: usize, buf: &[u8]) -> usize {
    let result: usize;
    
    unsafe {
        core::arch::asm!(
            "syscall",
            inout("rax") 1usize => result,  // SYS_WRITE
            in("rdi") fd,
            in("rsi") buf.as_ptr(),
            in("rdx") buf.len(),
        );
    }
    
    result
}

/// Syscall : terminer le processus
/// 
/// # Arguments
/// * `status` - Code de sortie
pub fn syscall_exit(status: i32) -> ! {
    unsafe {
        core::arch::asm!(
            "syscall",
            in("rax") 60,  // SYS_EXIT
            in("rdi") status,
            options(noreturn)
        );
    }
}

/// Syscall : obtenir le PID du processus
pub fn syscall_getpid() -> u32 {
    let result: u32;
    
    unsafe {
        core::arch::asm!(
            "syscall",
            inout("rax") 39u32 => result,  // SYS_GETPID
        );
    }
    
    result
}

/// Crée un contexte Ring 3 pour un programme utilisateur
pub fn create_user_context(entry_point: u64, user_stack: u64) -> Ring3Context {
    Ring3Context::new(entry_point, user_stack)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fibonacci() {
        let result = user_program_fibonacci(5);
        assert_eq!(result, 5); // fib(5) = 5
    }
    
    #[test]
    fn test_math_operations() {
        let a = 10;
        let b = 20;
        let sum = a + b;
        let product = a * b;
        
        assert_eq!(sum, 30);
        assert_eq!(product, 200);
    }
}
