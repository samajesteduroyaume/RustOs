/// Module I/O Queues pour NVMe
/// 
/// Implémente les queues d'I/O multiples pour performance optimale

use alloc::vec::Vec;
use alloc::collections::VecDeque;
use spin::Mutex;
use super::nvme::NVMeCommand;

/// Taille maximale d'une queue
pub const MAX_QUEUE_SIZE: usize = 256;

/// Nombre de queues I/O (1 par CPU idéalement)
pub const NUM_IO_QUEUES: usize = 4;

/// Submission Queue Entry
#[derive(Debug, Clone)]
pub struct SubmissionQueueEntry {
    /// Commande NVMe
    pub command: NVMeCommand,
    /// ID de la commande
    pub command_id: u16,
    /// Callback quand complété
    pub callback: Option<fn(u16, u32)>,
}

/// Completion Queue Entry
#[derive(Debug, Clone, Copy)]
pub struct CompletionQueueEntry {
    /// Résultat
    pub result: u32,
    /// ID de la commande
    pub command_id: u16,
    /// Status
    pub status: u16,
}

/// Submission Queue
pub struct SubmissionQueue {
    /// Entrées
    entries: VecDeque<SubmissionQueueEntry>,
    /// Taille maximale
    max_size: usize,
    /// Head pointer
    head: u16,
    /// Tail pointer
    tail: u16,
    /// Nombre de commandes soumises
    submitted: usize,
}

impl SubmissionQueue {
    /// Crée une nouvelle queue
    pub fn new(max_size: usize) -> Self {
        Self {
            entries: VecDeque::with_capacity(max_size),
            max_size,
            head: 0,
            tail: 0,
            submitted: 0,
        }
    }
    
    /// Soumet une commande
    pub fn submit(&mut self, entry: SubmissionQueueEntry) -> Result<(), QueueError> {
        if self.entries.len() >= self.max_size {
            return Err(QueueError::QueueFull);
        }
        
        self.entries.push_back(entry);
        self.tail = ((self.tail + 1) % self.max_size as u16) as u16;
        self.submitted += 1;
        
        Ok(())
    }
    
    /// Retire une commande
    pub fn pop(&mut self) -> Option<SubmissionQueueEntry> {
        if let Some(entry) = self.entries.pop_front() {
            self.head = ((self.head + 1) % self.max_size as u16) as u16;
            Some(entry)
        } else {
            None
        }
    }
    
    /// Nombre d'entrées
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    
    /// Vérifie si vide
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    
    /// Vérifie si pleine
    pub fn is_full(&self) -> bool {
        self.entries.len() >= self.max_size
    }
}

/// Completion Queue
pub struct CompletionQueue {
    /// Entrées
    entries: VecDeque<CompletionQueueEntry>,
    /// Taille maximale
    max_size: usize,
    /// Head pointer
    head: u16,
    /// Nombre de complétions
    completed: usize,
}

impl CompletionQueue {
    /// Crée une nouvelle queue
    pub fn new(max_size: usize) -> Self {
        Self {
            entries: VecDeque::with_capacity(max_size),
            max_size,
            head: 0,
            completed: 0,
        }
    }
    
    /// Ajoute une complétion
    pub fn add(&mut self, entry: CompletionQueueEntry) -> Result<(), QueueError> {
        if self.entries.len() >= self.max_size {
            return Err(QueueError::QueueFull);
        }
        
        self.entries.push_back(entry);
        self.completed += 1;
        
        Ok(())
    }
    
    /// Retire une complétion
    pub fn pop(&mut self) -> Option<CompletionQueueEntry> {
        if let Some(entry) = self.entries.pop_front() {
            self.head = ((self.head + 1) % self.max_size as u16) as u16;
            Some(entry)
        } else {
            None
        }
    }
    
    /// Nombre d'entrées
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    
    /// Vérifie si vide
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Paire de queues I/O
pub struct IoQueuePair {
    /// Submission queue
    pub sq: SubmissionQueue,
    /// Completion queue
    pub cq: CompletionQueue,
    /// ID de la queue
    pub id: u16,
}

impl IoQueuePair {
    /// Crée une nouvelle paire
    pub fn new(id: u16, size: usize) -> Self {
        Self {
            sq: SubmissionQueue::new(size),
            cq: CompletionQueue::new(size),
            id,
        }
    }
    
    /// Traite les complétions
    pub fn process_completions(&mut self) -> usize {
        let mut processed = 0;
        
        while let Some(completion) = self.cq.pop() {
            // TODO: Appeler le callback associé à la commande
            processed += 1;
        }
        
        processed
    }
}

/// Gestionnaire de queues multiples
pub struct IoQueueManager {
    /// Queues I/O
    queues: Vec<IoQueuePair>,
    /// Prochaine queue à utiliser (round-robin)
    next_queue: usize,
}

impl IoQueueManager {
    /// Crée un nouveau gestionnaire
    pub fn new(num_queues: usize, queue_size: usize) -> Self {
        let mut queues = Vec::with_capacity(num_queues);
        
        for i in 0..num_queues {
            queues.push(IoQueuePair::new(i as u16, queue_size));
        }
        
        Self {
            queues,
            next_queue: 0,
        }
    }
    
    /// Soumet une commande (round-robin)
    pub fn submit_command(&mut self, entry: SubmissionQueueEntry) -> Result<u16, QueueError> {
        let queue_id = self.next_queue;
        
        // Essayer de soumettre
        self.queues[queue_id].sq.submit(entry)?;
        
        // Passer à la queue suivante
        self.next_queue = (self.next_queue + 1) % self.queues.len();
        
        Ok(queue_id as u16)
    }
    
    /// Soumet une commande sur une queue spécifique
    pub fn submit_to_queue(&mut self, queue_id: usize, entry: SubmissionQueueEntry) -> Result<(), QueueError> {
        if queue_id >= self.queues.len() {
            return Err(QueueError::InvalidQueue);
        }
        
        self.queues[queue_id].sq.submit(entry)
    }
    
    /// Traite toutes les complétions
    pub fn process_all_completions(&mut self) -> usize {
        let mut total = 0;
        
        for queue in &mut self.queues {
            total += queue.process_completions();
        }
        
        total
    }
    
    /// Retourne les statistiques
    pub fn get_stats(&self) -> IoQueueStats {
        let mut total_submitted = 0;
        let mut total_completed = 0;
        let mut total_pending = 0;
        
        for queue in &self.queues {
            total_submitted += queue.sq.submitted;
            total_completed += queue.cq.completed;
            total_pending += queue.sq.len();
        }
        
        IoQueueStats {
            num_queues: self.queues.len(),
            total_submitted,
            total_completed,
            total_pending,
        }
    }
}

/// Erreurs de queue
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueueError {
    QueueFull,
    QueueEmpty,
    InvalidQueue,
}

/// Statistiques des queues
#[derive(Debug, Clone)]
pub struct IoQueueStats {
    pub num_queues: usize,
    pub total_submitted: usize,
    pub total_completed: usize,
    pub total_pending: usize,
}

/// Instance globale du gestionnaire de queues
use lazy_static::lazy_static;

lazy_static! {
    pub static ref IO_QUEUE_MANAGER: Mutex<IoQueueManager> = 
        Mutex::new(IoQueueManager::new(NUM_IO_QUEUES, MAX_QUEUE_SIZE));
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test_case]
    fn test_submission_queue() {
        let mut sq = SubmissionQueue::new(4);
        assert!(sq.is_empty());
        assert!(!sq.is_full());
    }
    
    #[test_case]
    fn test_queue_pair() {
        let mut pair = IoQueuePair::new(0, 4);
        assert_eq!(pair.id, 0);
        assert!(pair.sq.is_empty());
        assert!(pair.cq.is_empty());
    }
    
    #[test_case]
    fn test_queue_manager() {
        let manager = IoQueueManager::new(4, 256);
        let stats = manager.get_stats();
        assert_eq!(stats.num_queues, 4);
        assert_eq!(stats.total_submitted, 0);
    }
}
