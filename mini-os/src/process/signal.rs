/// Module de gestion des signaux POSIX
/// 
/// Ce module implémente un système de signaux similaire à POSIX pour RustOS.
/// Les signaux permettent la communication asynchrone entre processus et le noyau.

use alloc::vec::Vec;
use alloc::boxed::Box;
use spin::Mutex;

/// Types de signaux POSIX
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Signal {
    /// Signal de terminaison (peut être intercepté)
    SIGTERM = 15,
    /// Signal de kill (ne peut pas être intercepté)
    SIGKILL = 9,
    /// Signal de stop (suspend le processus)
    SIGSTOP = 19,
    /// Signal de continue (reprend le processus)
    SIGCONT = 18,
    /// Signal d'interruption (Ctrl+C)
    SIGINT = 2,
    /// Signal de quit (Ctrl+\)
    SIGQUIT = 3,
    /// Signal d'alarme
    SIGALRM = 14,
    /// Signal utilisateur 1
    SIGUSR1 = 10,
    /// Signal utilisateur 2
    SIGUSR2 = 12,
    /// Signal de child terminé
    SIGCHLD = 17,
    /// Signal de pipe cassé
    SIGPIPE = 13,
    /// Signal de segmentation fault
    SIGSEGV = 11,
    /// Signal d'instruction illégale
    SIGILL = 4,
    /// Signal de floating point exception
    SIGFPE = 8,
    /// Signal de bus error
    SIGBUS = 7,
}

impl Signal {
    /// Convertit un u8 en Signal
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            2 => Some(Signal::SIGINT),
            3 => Some(Signal::SIGQUIT),
            4 => Some(Signal::SIGILL),
            7 => Some(Signal::SIGBUS),
            8 => Some(Signal::SIGFPE),
            9 => Some(Signal::SIGKILL),
            10 => Some(Signal::SIGUSR1),
            11 => Some(Signal::SIGSEGV),
            12 => Some(Signal::SIGUSR2),
            13 => Some(Signal::SIGPIPE),
            14 => Some(Signal::SIGALRM),
            15 => Some(Signal::SIGTERM),
            17 => Some(Signal::SIGCHLD),
            18 => Some(Signal::SIGCONT),
            19 => Some(Signal::SIGSTOP),
            _ => None,
        }
    }

    /// Retourne true si le signal peut être intercepté
    pub fn can_be_caught(self) -> bool {
        !matches!(self, Signal::SIGKILL | Signal::SIGSTOP)
    }

    /// Retourne l'action par défaut pour ce signal
    pub fn default_action(self) -> SignalAction {
        match self {
            Signal::SIGTERM | Signal::SIGINT | Signal::SIGQUIT | 
            Signal::SIGKILL | Signal::SIGSEGV | Signal::SIGILL |
            Signal::SIGFPE | Signal::SIGBUS | Signal::SIGPIPE => SignalAction::Terminate,
            
            Signal::SIGSTOP => SignalAction::Stop,
            Signal::SIGCONT => SignalAction::Continue,
            
            Signal::SIGCHLD | Signal::SIGALRM | 
            Signal::SIGUSR1 | Signal::SIGUSR2 => SignalAction::Ignore,
        }
    }
}

/// Action à effectuer lors de la réception d'un signal
#[derive(Debug, Clone, Copy)]
pub enum SignalAction {
    /// Terminer le processus
    Terminate,
    /// Ignorer le signal
    Ignore,
    /// Stopper le processus
    Stop,
    /// Continuer le processus
    Continue,
    /// Appeler un handler personnalisé
    Handler(fn()),
}

/// Handler de signal personnalisé
pub type SignalHandler = fn();

/// Table des handlers de signaux pour un processus
#[derive(Clone)]
pub struct SignalHandlerTable {
    /// Handlers pour chaque signal
    handlers: [SignalAction; 32],
}

impl SignalHandlerTable {
    /// Crée une nouvelle table de handlers avec les actions par défaut
    pub fn new() -> Self {
        let mut handlers = [SignalAction::Ignore; 32];
        
        // Initialiser avec les actions par défaut
        for i in 1..32 {
            if let Some(signal) = Signal::from_u8(i as u8) {
                handlers[i] = signal.default_action();
            }
        }
        
        Self { handlers }
    }

    /// Définit le handler pour un signal
    pub fn set_handler(&mut self, signal: Signal, action: SignalAction) -> Result<(), &'static str> {
        if !signal.can_be_caught() {
            return Err("Cannot set handler for SIGKILL or SIGSTOP");
        }
        
        let index = signal as usize;
        if index < 32 {
            self.handlers[index] = action;
            Ok(())
        } else {
            Err("Invalid signal number")
        }
    }

    /// Obtient l'action pour un signal
    pub fn get_action(&self, signal: Signal) -> &SignalAction {
        let index = signal as usize;
        if index < 32 {
            &self.handlers[index]
        } else {
            &SignalAction::Ignore
        }
    }

    /// Réinitialise un handler à son action par défaut
    pub fn reset_handler(&mut self, signal: Signal) {
        let index = signal as usize;
        if index < 32 {
            self.handlers[index] = signal.default_action();
        }
    }
}

impl Default for SignalHandlerTable {
    fn default() -> Self {
        Self::new()
    }
}

/// Queue de signaux en attente pour un processus
pub struct SignalQueue {
    /// Signaux en attente
    pending: Vec<Signal>,
    /// Masque de signaux bloqués
    blocked: u32,
}

impl SignalQueue {
    /// Crée une nouvelle queue de signaux
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
            blocked: 0,
        }
    }

    /// Ajoute un signal à la queue
    pub fn enqueue(&mut self, signal: Signal) {
        // Vérifier si le signal est bloqué
        let signal_bit = 1 << (signal as u8);
        if self.blocked & signal_bit == 0 {
            // Signal non bloqué, l'ajouter à la queue
            if !self.pending.contains(&signal) {
                self.pending.push(signal);
            }
        }
    }

    /// Retire et retourne le prochain signal de la queue
    pub fn dequeue(&mut self) -> Option<Signal> {
        if self.pending.is_empty() {
            None
        } else {
            Some(self.pending.remove(0))
        }
    }

    /// Vérifie si la queue contient des signaux
    pub fn has_pending(&self) -> bool {
        !self.pending.is_empty()
    }

    /// Bloque un signal
    pub fn block(&mut self, signal: Signal) {
        let signal_bit = 1 << (signal as u8);
        self.blocked |= signal_bit;
    }

    /// Débloque un signal
    pub fn unblock(&mut self, signal: Signal) {
        let signal_bit = 1 << (signal as u8);
        self.blocked &= !signal_bit;
    }

    /// Vérifie si un signal est bloqué
    pub fn is_blocked(&self, signal: Signal) -> bool {
        let signal_bit = 1 << (signal as u8);
        self.blocked & signal_bit != 0
    }

    /// Vide la queue de signaux
    pub fn clear(&mut self) {
        self.pending.clear();
    }
}

impl Default for SignalQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// Gestionnaire global de signaux
pub struct SignalManager {
    // Pour l'instant vide, sera étendu plus tard
}

impl SignalManager {
    /// Crée un nouveau gestionnaire de signaux
    pub const fn new() -> Self {
        Self {}
    }

    /// Envoie un signal à un processus
    pub fn send_signal(&self, target_pid: u64, signal: Signal, process_manager: &mut crate::process::ProcessManager) -> Result<(), &'static str> {
        // Trouver le processus cible
        let target_process = process_manager.processes()
            .iter()
            .find(|p| p.lock().pid == target_pid)
            .ok_or("Processus cible introuvable")?;
        
        // Ajouter le signal à sa queue
        target_process.lock().signal_queue.enqueue(signal);
        
        // Si le processus est bloqué et que le signal devrait le réveiller, le réveiller
        if target_process.lock().state == crate::process::ProcessState::Blocked {
            match signal {
                Signal::SIGCONT | Signal::SIGKILL => {
                    target_process.lock().state = crate::process::ProcessState::Ready;
                }
                _ => {}
            }
        }
        
        Ok(())
    }

    /// Traite les signaux en attente pour un processus
    /// Retourne true si le processus doit être terminé
    pub fn deliver_signals(process: &mut crate::process::Process) -> bool {
        let mut should_terminate = false;
        
        // Traiter tous les signaux en attente
        while let Some(signal) = process.signal_queue.dequeue() {
            // Obtenir l'action pour ce signal
            let action = process.signal_handlers.get_action(signal).clone();
            
            match action {
                SignalAction::Terminate => {
                    // Terminer le processus
                    process.state = crate::process::ProcessState::Terminated;
                    should_terminate = true;
                    break; // Arrêter le traitement des signaux
                }
                
                SignalAction::Stop => {
                    // Stopper le processus
                    process.state = crate::process::ProcessState::Blocked;
                    break; // Arrêter le traitement des signaux
                }
                
                SignalAction::Continue => {
                    // Continuer le processus s'il était stoppé
                    if process.state == crate::process::ProcessState::Blocked {
                        process.state = crate::process::ProcessState::Ready;
                    }
                }
                
                SignalAction::Ignore => {
                    // Ignorer le signal, continuer avec le suivant
                    continue;
                }
                
                SignalAction::Handler(handler_fn) => {
                    // Exécuter le handler personnalisé
                    // IMPORTANT: Dans un vrai OS, on devrait:
                    // 1. Sauvegarder le contexte actuel
                    // 2. Configurer la pile pour exécuter le handler
                    // 3. Retourner au code normal après le handler
                    // Pour l'instant, on appelle directement le handler
                    handler_fn();
                }
            }
        }
        
        should_terminate
    }
}

/// Instance globale du gestionnaire de signaux
pub static SIGNAL_MANAGER: Mutex<SignalManager> = Mutex::new(SignalManager::new());

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_signal_from_u8() {
        assert_eq!(Signal::from_u8(9), Some(Signal::SIGKILL));
        assert_eq!(Signal::from_u8(15), Some(Signal::SIGTERM));
        assert_eq!(Signal::from_u8(99), None);
    }

    #[test_case]
    fn test_signal_can_be_caught() {
        assert!(!Signal::SIGKILL.can_be_caught());
        assert!(!Signal::SIGSTOP.can_be_caught());
        assert!(Signal::SIGTERM.can_be_caught());
        assert!(Signal::SIGINT.can_be_caught());
    }

    #[test_case]
    fn test_signal_queue() {
        let mut queue = SignalQueue::new();
        
        assert!(!queue.has_pending());
        
        queue.enqueue(Signal::SIGTERM);
        queue.enqueue(Signal::SIGINT);
        
        assert!(queue.has_pending());
        assert_eq!(queue.dequeue(), Some(Signal::SIGTERM));
        assert_eq!(queue.dequeue(), Some(Signal::SIGINT));
        assert_eq!(queue.dequeue(), None);
    }

    #[test_case]
    fn test_signal_blocking() {
        let mut queue = SignalQueue::new();
        
        queue.block(Signal::SIGTERM);
        queue.enqueue(Signal::SIGTERM);
        
        // Le signal bloqué ne devrait pas être dans la queue
        assert!(!queue.has_pending());
        
        queue.unblock(Signal::SIGTERM);
        queue.enqueue(Signal::SIGTERM);
        
        // Maintenant il devrait être dans la queue
        assert!(queue.has_pending());
    }

    #[test_case]
    fn test_handler_table() {
        let mut table = SignalHandlerTable::new();
        
        // Vérifier l'action par défaut
        match table.get_action(Signal::SIGTERM) {
            SignalAction::Terminate => {},
            _ => panic!("Wrong default action for SIGTERM"),
        }
        
        // Définir un handler personnalisé
        fn custom_handler() {}
        let result = table.set_handler(Signal::SIGTERM, SignalAction::Handler(custom_handler));
        assert!(result.is_ok());
        
        // Vérifier qu'on ne peut pas définir de handler pour SIGKILL
        let result = table.set_handler(Signal::SIGKILL, SignalAction::Ignore);
        assert!(result.is_err());
    }
}
