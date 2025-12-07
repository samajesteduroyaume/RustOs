/// Module Message Queues
/// 
/// Implémente POSIX message queues

use alloc::collections::{BTreeMap, VecDeque};
use alloc::vec::Vec;
use spin::Mutex;

/// Priorité de message (0-31, 31 = plus haute)
pub type Priority = u8;

/// Message
#[derive(Debug, Clone)]
pub struct Message {
    /// Données
    pub data: Vec<u8>,
    /// Priorité
    pub priority: Priority,
}

impl Message {
    pub fn new(data: Vec<u8>, priority: Priority) -> Self {
        Self { data, priority }
    }
}

/// File de messages
pub struct MessageQueue {
    /// ID de la queue
    pub id: u32,
    /// Messages (triés par priorité)
    messages: Vec<Message>,
    /// Taille maximale d'un message
    pub max_msg_size: usize,
    /// Nombre maximum de messages
    pub max_msgs: usize,
}

impl MessageQueue {
    /// Crée une nouvelle queue
    pub fn new(id: u32, max_msg_size: usize, max_msgs: usize) -> Self {
        Self {
            id,
            messages: Vec::new(),
            max_msg_size,
            max_msgs,
        }
    }
    
    /// Envoie un message
    pub fn send(&mut self, data: Vec<u8>, priority: Priority) -> Result<(), MqError> {
        if data.len() > self.max_msg_size {
            return Err(MqError::MessageTooLarge);
        }
        
        if self.messages.len() >= self.max_msgs {
            return Err(MqError::QueueFull);
        }
        
        let msg = Message::new(data, priority);
        
        // Insérer trié par priorité (plus haute en premier)
        let pos = self.messages.iter()
            .position(|m| m.priority < priority)
            .unwrap_or(self.messages.len());
        
        self.messages.insert(pos, msg);
        
        Ok(())
    }
    
    /// Reçoit un message
    pub fn receive(&mut self) -> Result<Message, MqError> {
        if self.messages.is_empty() {
            return Err(MqError::WouldBlock);
        }
        
        // Retirer le message de plus haute priorité (premier)
        Ok(self.messages.remove(0))
    }
    
    /// Nombre de messages
    pub fn len(&self) -> usize {
        self.messages.len()
    }
    
    /// Vérifie si vide
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
    
    /// Vérifie si pleine
    pub fn is_full(&self) -> bool {
        self.messages.len() >= self.max_msgs
    }
}

/// Gestionnaire de message queues
pub struct MessageQueueManager {
    /// Queues par ID
    queues: BTreeMap<u32, MessageQueue>,
    /// Prochain ID
    next_id: u32,
}

impl MessageQueueManager {
    /// Crée un nouveau gestionnaire
    pub const fn new() -> Self {
        Self {
            queues: BTreeMap::new(),
            next_id: 1,
        }
    }
    
    /// Crée une message queue
    pub fn mq_open(&mut self, max_msg_size: usize, max_msgs: usize) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        
        let queue = MessageQueue::new(id, max_msg_size, max_msgs);
        self.queues.insert(id, queue);
        
        id
    }
    
    /// Envoie un message
    pub fn mq_send(&mut self, id: u32, data: Vec<u8>, priority: Priority) -> Result<(), MqError> {
        let queue = self.queues.get_mut(&id).ok_or(MqError::NotFound)?;
        queue.send(data, priority)
    }
    
    /// Reçoit un message
    pub fn mq_receive(&mut self, id: u32) -> Result<Message, MqError> {
        let queue = self.queues.get_mut(&id).ok_or(MqError::NotFound)?;
        queue.receive()
    }
    
    /// Ferme une queue
    pub fn mq_close(&mut self, id: u32) -> Result<(), MqError> {
        self.queues.remove(&id).ok_or(MqError::NotFound)?;
        Ok(())
    }
    
    /// Retourne les attributs d'une queue
    pub fn mq_getattr(&self, id: u32) -> Result<MqAttr, MqError> {
        let queue = self.queues.get(&id).ok_or(MqError::NotFound)?;
        
        Ok(MqAttr {
            max_msgs: queue.max_msgs,
            max_msg_size: queue.max_msg_size,
            current_msgs: queue.len(),
        })
    }
}

/// Attributs de message queue
#[derive(Debug, Clone, Copy)]
pub struct MqAttr {
    pub max_msgs: usize,
    pub max_msg_size: usize,
    pub current_msgs: usize,
}

/// Erreurs de message queue
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MqError {
    NotFound,
    QueueFull,
    MessageTooLarge,
    WouldBlock,
}

/// Instance globale
use lazy_static::lazy_static;

lazy_static! {
    pub static ref MQ_MANAGER: Mutex<MessageQueueManager> = Mutex::new(MessageQueueManager::new());
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test_case]
    fn test_message_queue() {
        let mut queue = MessageQueue::new(1, 1024, 10);
        
        queue.send(b"Low priority".to_vec(), 5).unwrap();
        queue.send(b"High priority".to_vec(), 20).unwrap();
        queue.send(b"Medium priority".to_vec(), 10).unwrap();
        
        // Doit recevoir par ordre de priorité
        let msg1 = queue.receive().unwrap();
        assert_eq!(msg1.priority, 20);
        
        let msg2 = queue.receive().unwrap();
        assert_eq!(msg2.priority, 10);
        
        let msg3 = queue.receive().unwrap();
        assert_eq!(msg3.priority, 5);
    }
    
    #[test_case]
    fn test_mq_manager() {
        let mut manager = MessageQueueManager::new();
        let id = manager.mq_open(1024, 10);
        
        manager.mq_send(id, b"Test".to_vec(), 10).unwrap();
        
        let msg = manager.mq_receive(id).unwrap();
        assert_eq!(msg.data, b"Test");
        assert_eq!(msg.priority, 10);
    }
    
    #[test_case]
    fn test_queue_full() {
        let mut queue = MessageQueue::new(1, 100, 2);
        
        queue.send(b"Msg1".to_vec(), 1).unwrap();
        queue.send(b"Msg2".to_vec(), 1).unwrap();
        
        let result = queue.send(b"Msg3".to_vec(), 1);
        assert_eq!(result, Err(MqError::QueueFull));
    }
}
