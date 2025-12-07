

/// Numéros des appels système
#[repr(u64)]
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
    // Gestion des priorités
    SetPriority = 9,
    GetPriority = 10,
    // Gestion des signaux
    Signal = 11,
    Kill = 12,
    SigAction = 13,
    SigProcMask = 14,
    // Mémoire partagée
    ShmGet = 15,
    ShmAt = 16,
    ShmDt = 17,
    ShmCtl = 18,
    // Memory mapping
    Mmap = 19,
    Munmap = 20,
    Symlink = 21,
    Readlink = 22,
    Chmod = 23,
    Chown = 24,
    Chgrp = 25,
    // Gestion des threads
    ThreadCreate = 26,
}

/// Résultat d'un appel système
#[derive(Debug)]
pub enum SyscallResult {
    Success(u64),
    Error(SyscallError),
}

/// Erreurs d'appel système
#[derive(Debug)]
pub enum SyscallError {
    InvalidSyscall,
    InvalidArgument,
    NoSuchProcess,
    NotFound,
    PermissionDenied,
    IoError,
    OutOfMemory,
    NotSupported,
}

/// Gestionnaire d'appels système
pub struct SyscallHandler;

impl SyscallHandler {
    /// Crée un nouveau gestionnaire d'appels système
    pub fn new() -> Self {
        Self
    }
    
    /// Traite un appel système
    pub fn handle(&self, num: u64, args: &[u64]) -> SyscallResult {
        match num {
            x if x == SyscallNumber::Exit as u64 => self.handle_exit(args[0] as i32),
            x if x == SyscallNumber::Fork as u64 => self.handle_fork(),
            x if x == SyscallNumber::Read as u64 => self.handle_read(args[0] as usize, args[1] as *mut u8, args[2] as usize),
            x if x == SyscallNumber::Write as u64 => self.handle_write(args[0] as usize, args[1] as *const u8, args[2] as usize),
            x if x == SyscallNumber::GetPid as u64 => self.handle_getpid(),
            x if x == SyscallNumber::SetPriority as u64 => self.handle_set_priority(args[0], args[1] as u8),
            x if x == SyscallNumber::GetPriority as u64 => self.handle_get_priority(args[0]),
            x if x == SyscallNumber::Signal as u64 => self.handle_signal(args[0] as u8, args[1]),
            x if x == SyscallNumber::Kill as u64 => self.handle_kill(args[0], args[1] as u8),
            x if x == SyscallNumber::SigAction as u64 => self.handle_sigaction(args[0] as u8, args[1], args[2]),
            x if x == SyscallNumber::SigProcMask as u64 => self.handle_sigprocmask(args[0] as i32, args[1], args[2]),
            x if x == SyscallNumber::ShmGet as u64 => self.handle_shmget(args[0] as i32, args[1] as usize, args[2] as i32),
            x if x == SyscallNumber::ShmAt as u64 => self.handle_shmat(args[0] as i32, args[1]),
            x if x == SyscallNumber::ShmDt as u64 => self.handle_shmdt(args[0]),
            x if x == SyscallNumber::ShmCtl as u64 => self.handle_shmctl(args[0] as i32, args[1] as i32),
            x if x == SyscallNumber::Mmap as u64 => self.handle_mmap(args[0], args[1] as usize, args[2] as i32, args[3] as i32, args[4] as i32, args[5]),
            x if x == SyscallNumber::Munmap as u64 => self.handle_munmap(args[0], args[1] as usize),
            x if x == SyscallNumber::Symlink as u64 => self.handle_symlink(args[0] as *const u8, args[1] as *const u8),
            x if x == SyscallNumber::Readlink as u64 => self.handle_readlink(args[0] as *const u8, args[1] as *mut u8, args[2] as usize),
            x if x == SyscallNumber::Chmod as u64 => self.handle_chmod(args[0], args[1] as u16),
            x if x == SyscallNumber::Chown as u64 => self.handle_chown(args[0], args[1] as u32),
            x if x == SyscallNumber::Chgrp as u64 => self.handle_chgrp(args[0], args[1] as u32),
            x if x == SyscallNumber::ThreadCreate as u64 => self.handle_thread_create(args[0]),
            _ => SyscallResult::Error(SyscallError::InvalidSyscall),
        }
    }
    
    fn handle_exit(&self, _status: i32) -> SyscallResult {
        // TODO: Implémenter la terminaison du processus
        SyscallResult::Success(0)
    }
    
    fn handle_fork(&self) -> SyscallResult {
        // TODO: Implémenter la création d'un nouveau processus
        SyscallResult::Error(SyscallError::NotSupported)
    }
    
    fn handle_read(&self, _fd: usize, _buf: *mut u8, _count: usize) -> SyscallResult {
        // TODO: Implémenter la lecture depuis un descripteur de fichier
        SyscallResult::Error(SyscallError::NotSupported)
    }
    
    fn handle_write(&self, _fd: usize, _buf: *const u8, _count: usize) -> SyscallResult {
        // TODO: Implémenter l'écriture vers un descripteur de fichier
        SyscallResult::Error(SyscallError::NotSupported)
    }
    
    fn handle_getpid(&self) -> SyscallResult {
        // TODO: Implémenter la récupération du PID
        SyscallResult::Success(0)
    }
    
    /// Définit la priorité d'un processus
    /// args[0] = pid (0 = processus actuel)
    /// args[1] = priority (0-4)
    fn handle_set_priority(&self, pid: u64, priority: u8) -> SyscallResult {
        use crate::process::{ProcessPriority, PROCESS_MANAGER, current_process, get_process_by_pid};
        
        // Valider la priorité
        if priority > 4 {
            return SyscallResult::Error(SyscallError::InvalidArgument);
        }
        
        let priority = ProcessPriority::from_u8(priority);
        
        // Obtenir le processus cible
        let target = if pid == 0 {
            // Processus actuel
            current_process()
        } else {
            // Processus spécifique
            get_process_by_pid(pid)
        };
        
        match target {
            Some(process) => {
                process.lock().set_priority(priority);
                SyscallResult::Success(0)
            }
            None => SyscallResult::Error(SyscallError::NoSuchProcess),
        }
    }
    
    /// Obtient la priorité d'un processus
    /// args[0] = pid (0 = processus actuel)
    fn handle_get_priority(&self, pid: u64) -> SyscallResult {
        use crate::process::{current_process, get_process_by_pid};
        
        // Obtenir le processus cible
        let target = if pid == 0 {
            current_process()
        } else {
            get_process_by_pid(pid)
        };
        
        match target {
            Some(process) => {
                let priority = process.lock().get_priority();
                SyscallResult::Success(priority.to_u8() as u64)
            }
            None => SyscallResult::Error(SyscallError::NoSuchProcess),
        }
    }
    
    /// Définit un handler de signal
    /// args[0] = signal number
    /// args[1] = handler address (0 = default, 1 = ignore, other = custom handler)
    fn handle_signal(&self, signal_num: u8, handler: u64) -> SyscallResult {
        use crate::process::signal::{Signal, SignalAction};
        use crate::process::current_process;
        
        // Valider le numéro de signal
        let signal = match Signal::from_u8(signal_num) {
            Some(s) => s,
            None => return SyscallResult::Error(SyscallError::InvalidArgument),
        };
        
        // Vérifier si le signal peut être intercepté
        if !signal.can_be_caught() {
            return SyscallResult::Error(SyscallError::PermissionDenied);
        }
        
        // Déterminer l'action
        let action = match handler {
            0 => signal.default_action(),
            1 => SignalAction::Ignore,
            _ => {
                // Handler personnalisé
                let handler_fn: fn() = unsafe { core::mem::transmute(handler as usize) };
                SignalAction::Handler(handler_fn)
            }
        };
        
        // Définir le handler pour le processus actuel
        match current_process() {
            Some(process) => {
                match process.lock().signal_handlers.set_handler(signal, action) {
                    Ok(_) => SyscallResult::Success(0),
                    Err(_) => SyscallResult::Error(SyscallError::InvalidArgument),
                }
            }
            None => SyscallResult::Error(SyscallError::NoSuchProcess),
        }
    }
    
    /// Envoie un signal à un processus
    /// args[0] = pid
    /// args[1] = signal number
    fn handle_kill(&self, pid: u64, signal_num: u8) -> SyscallResult {
        use crate::process::signal::{Signal, SIGNAL_MANAGER};
        use crate::process::PROCESS_MANAGER;
        
        // Valider le numéro de signal
        let signal = match Signal::from_u8(signal_num) {
            Some(s) => s,
            None => return SyscallResult::Error(SyscallError::InvalidArgument),
        };
        
        // Envoyer le signal au processus cible
        let mut pm = PROCESS_MANAGER.lock();
        match SIGNAL_MANAGER.lock().send_signal(pid, signal, &mut *pm) {
            Ok(_) => SyscallResult::Success(0),
            Err(_) => SyscallResult::Error(SyscallError::NoSuchProcess),
        }
    }
    
    /// Configure l'action pour un signal (version avancée de signal)
    /// args[0] = signal number
    /// args[1] = pointer to new sigaction struct
    /// args[2] = pointer to old sigaction struct (can be null)
    fn handle_sigaction(&self, signal_num: u8, new_action: u64, old_action: u64) -> SyscallResult {
        use crate::process::signal::Signal;
        
        // Valider le numéro de signal
        let signal = match Signal::from_u8(signal_num) {
            Some(s) => s,
            None => return SyscallResult::Error(SyscallError::InvalidArgument),
        };
        
        // Vérifier si le signal peut être intercepté
        if !signal.can_be_caught() {
            return SyscallResult::Error(SyscallError::PermissionDenied);
        }
        
        // TODO: Implémenter la lecture de la structure sigaction
        // TODO: Sauvegarder l'ancienne action si old_action != 0
        // TODO: Définir la nouvelle action
        SyscallResult::Success(0)
    }
    
    /// Examine et modifie le masque de signaux bloqués
    /// args[0] = how (0=SIG_BLOCK, 1=SIG_UNBLOCK, 2=SIG_SETMASK)
    /// args[1] = pointer to new mask (can be null)
    /// args[2] = pointer to old mask (can be null)
    fn handle_sigprocmask(&self, how: i32, new_mask: u64, old_mask: u64) -> SyscallResult {
        // Valider le paramètre 'how'
        if how < 0 || how > 2 {
            return SyscallResult::Error(SyscallError::InvalidArgument);
        }
        
        // TODO: Récupérer le processus actuel
        // TODO: Sauvegarder l'ancien masque si old_mask != 0
        // TODO: Modifier le masque selon 'how' et 'new_mask'
        SyscallResult::Success(0)
    }
    
    /// Crée ou récupère un segment de mémoire partagée
    /// args[0] = key
    /// args[1] = size
    /// args[2] = flags
    fn handle_shmget(&self, key: i32, size: usize, flags: i32) -> SyscallResult {
        use crate::memory::SHM_MANAGER;
        
        // TODO: Récupérer UID/GID du processus actuel
        let uid = 1000; // Placeholder
        let gid = 1000; // Placeholder
        
        match SHM_MANAGER.lock().shmget(key, size, flags, uid, gid) {
            Ok(id) => SyscallResult::Success(id as u64),
            Err(_) => SyscallResult::Error(SyscallError::OutOfMemory),
        }
    }
    
    /// Attache un segment de mémoire partagée
    /// args[0] = id
    /// args[1] = addr (0 = auto)
    fn handle_shmat(&self, id: i32, addr: u64) -> SyscallResult {
        use crate::memory::SHM_MANAGER;
        use x86_64::VirtAddr;
        
        // TODO: Récupérer UID/GID du processus actuel
        let uid = 1000; // Placeholder
        let gid = 1000; // Placeholder
        
        let virt_addr = if addr == 0 {
            None
        } else {
            Some(VirtAddr::new(addr))
        };
        
        match SHM_MANAGER.lock().shmat(id, virt_addr, uid, gid) {
            Ok(addr) => SyscallResult::Success(addr.as_u64()),
            Err(_) => SyscallResult::Error(SyscallError::PermissionDenied),
        }
    }
    
    /// Détache un segment de mémoire partagée
    /// args[0] = addr
    fn handle_shmdt(&self, addr: u64) -> SyscallResult {
        use crate::memory::SHM_MANAGER;
        use x86_64::VirtAddr;
        
        match SHM_MANAGER.lock().shmdt(VirtAddr::new(addr)) {
            Ok(_) => SyscallResult::Success(0),
            Err(_) => SyscallResult::Error(SyscallError::InvalidArgument),
        }
    }
    
    /// Contrôle un segment de mémoire partagée
    /// args[0] = id
    /// args[1] = cmd (1=IPC_STAT, 2=IPC_SET, 3=IPC_RMID)
    fn handle_shmctl(&self, id: i32, cmd: i32) -> SyscallResult {
        use crate::memory::{SHM_MANAGER, ShmCmd};
        
        let shm_cmd = match cmd {
            1 => ShmCmd::IpcStat,
            2 => ShmCmd::IpcSet,
            3 => ShmCmd::IpcRmid,
            _ => return SyscallResult::Error(SyscallError::InvalidArgument),
        };
        
        // TODO: Récupérer UID du processus actuel
        let uid = 1000; // Placeholder
        
        match SHM_MANAGER.lock().shmctl(id, shm_cmd, uid) {
            Ok(_) => SyscallResult::Success(0),
            Err(_) => SyscallResult::Error(SyscallError::PermissionDenied),
        }
    }
    
    /// Mappe une région de mémoire
    /// args[0] = addr (0 = auto)
    /// args[1] = size
    /// args[2] = prot (PROT_READ | PROT_WRITE | PROT_EXEC)
    /// args[3] = flags (MAP_SHARED | MAP_PRIVATE | MAP_ANONYMOUS)
    /// args[4] = fd (-1 pour anonymous)
    /// args[5] = offset
    fn handle_mmap(&self, addr: u64, size: usize, prot: i32, flags: i32, fd: i32, offset: u64) -> SyscallResult {
        use crate::memory::MMAP_MANAGER;
        use x86_64::VirtAddr;
        
        // TODO: Récupérer UID du processus actuel
        let pid = 1; // Placeholder
        
        let virt_addr = if addr == 0 {
            None
        } else {
            Some(VirtAddr::new(addr))
        };
        
        let file_id = if fd >= 0 {
            Some(fd as u64)
        } else {
            None
        };
        
        match MMAP_MANAGER.lock().mmap(virt_addr, size, prot, flags, file_id, offset, pid) {
            Ok(addr) => SyscallResult::Success(addr.as_u64()),
            Err(_) => SyscallResult::Error(SyscallError::OutOfMemory),
        }
    }
    
    /// Démappe une région de mémoire
    /// args[0] = addr
    /// args[1] = size
    fn handle_munmap(&self, addr: u64, size: usize) -> SyscallResult {
        use crate::memory::MMAP_MANAGER;
        use x86_64::VirtAddr;
        
        match MMAP_MANAGER.lock().munmap(VirtAddr::new(addr), size) {
            Ok(_) => SyscallResult::Success(0),
            Err(_) => SyscallResult::Error(SyscallError::InvalidArgument),
        }
    }
    
    fn handle_symlink(&self, _target_ptr: *const u8, _link_ptr: *const u8) -> SyscallResult {
        use crate::fs::SYMLINK_MANAGER;
        use alloc::string::String;
        let target_path = String::from("/target");
        let link_path = String::from("/link");
        match SYMLINK_MANAGER.lock().create_symlink(link_path, target_path, 1000, 1000) {
            Ok(inode) => SyscallResult::Success(inode),
            Err(_) => SyscallResult::Error(SyscallError::InvalidArgument),
        }
    }
    
    fn handle_readlink(&self, _link_ptr: *const u8, _buf_ptr: *mut u8, _buf_size: usize) -> SyscallResult {
        use crate::fs::SYMLINK_MANAGER;
        use alloc::string::String;
        let link_path = String::from("/link");
        match SYMLINK_MANAGER.lock().readlink(&link_path) {
            Ok(target) => SyscallResult::Success(target.len() as u64),
            Err(_) => SyscallResult::Error(SyscallError::NotFound),
        }
    }
    
    fn handle_chmod(&self, inode: u64, mode: u16) -> SyscallResult {
        use crate::fs::PERMISSION_MANAGER;
        let caller_uid = 1000; // TODO: Récupérer l'UID du processus actuel
        match PERMISSION_MANAGER.lock().chmod(inode, mode, caller_uid) {
            Ok(_) => SyscallResult::Success(0),
            Err(_) => SyscallResult::Error(SyscallError::PermissionDenied),
        }
    }
    
    fn handle_chown(&self, inode: u64, uid: u32) -> SyscallResult {
        use crate::fs::PERMISSION_MANAGER;
        let caller_uid = 0; // TODO: Récupérer l'UID du processus actuel
        match PERMISSION_MANAGER.lock().chown(inode, uid, caller_uid) {
            Ok(_) => SyscallResult::Success(0),
            Err(_) => SyscallResult::Error(SyscallError::PermissionDenied),
        }
    }
    
    fn handle_chgrp(&self, inode: u64, gid: u32) -> SyscallResult {
        use crate::fs::PERMISSION_MANAGER;
        let caller_uid = 1000; // TODO: Récupérer l'UID du processus actuel
        match PERMISSION_MANAGER.lock().chgrp(inode, gid, caller_uid) {
            Ok(_) => SyscallResult::Success(0),
            Err(_) => SyscallResult::Error(SyscallError::PermissionDenied),
        }
    }
    
    /// Crée un nouveau thread dans le processus actuel
    /// args[0] = entry_point
    fn handle_thread_create(&self, entry_point: u64) -> SyscallResult {
        use crate::process::{PROCESS_MANAGER, current_process};
        
        // Obtenir le PID du processus actuel
        let current_pid = match current_process() {
            Some(p) => p.lock().pid,
            None => return SyscallResult::Error(SyscallError::NoSuchProcess),
        };
        
        // Créer le thread via le ProcessManager
        let mut pm = PROCESS_MANAGER.lock();
        match pm.create_thread(current_pid, entry_point) {
            Ok(tid) => SyscallResult::Success(tid),
            Err(_) => SyscallResult::Error(SyscallError::OutOfMemory), // Ou autre erreur appropriée
        }
    }
}
