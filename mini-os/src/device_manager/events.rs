/// Module de gestion des événements de périphériques
/// Intègre le système d'événements avec le scheduler

use alloc::vec::Vec;
use spin::Mutex;
use lazy_static::lazy_static;

/// Types d'événements de périphériques
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceEventType {
    /// Périphérique inséré/connecté
    Added,
    /// Périphérique retiré/déconnecté
    Removed,
    /// Périphérique connecté
    Connected,
    /// Périphérique déconnecté
    Disconnected,
    /// Changement d'état du périphérique
    StatusChanged,
    /// Erreur du périphérique
    Error,
}

/// Événement de périphérique
#[derive(Debug, Clone)]
pub struct DeviceEvent {
    /// Type d'événement
    pub event_type: DeviceEventType,
    /// Nom du périphérique
    pub device_name: alloc::string::String,
    /// Timestamp (en ticks)
    pub timestamp: u64,
    /// Données additionnelles
    pub data: u32,
}

/// Gestionnaire d'événements de périphériques
pub struct EventManager {
    /// File d'attente des événements
    events: Vec<DeviceEvent>,
    /// Nombre maximum d'événements en attente
    max_events: usize,
    /// Compteur d'événements traités
    processed_count: u64,
}

impl EventManager {
    /// Crée un nouveau gestionnaire d'événements
    pub fn new(max_events: usize) -> Self {
        Self {
            events: Vec::new(),
            max_events,
            processed_count: 0,
        }
    }
    
    /// Ajoute un événement à la file
    pub fn push_event(&mut self, event: DeviceEvent) -> Result<(), &'static str> {
        if self.events.len() >= self.max_events {
            return Err("Event queue full");
        }
        self.events.push(event);
        Ok(())
    }
    
    /// Récupère le prochain événement
    pub fn pop_event(&mut self) -> Option<DeviceEvent> {
        if self.events.is_empty() {
            return None;
        }
        self.processed_count += 1;
        Some(self.events.remove(0))
    }
    
    /// Retourne le nombre d'événements en attente
    pub fn pending_events(&self) -> usize {
        self.events.len()
    }
    
    /// Retourne le nombre d'événements traités
    pub fn processed_events(&self) -> u64 {
        self.processed_count
    }
    
    /// Vide la file d'attente
    pub fn clear(&mut self) {
        self.events.clear();
    }
}

lazy_static! {
    /// Gestionnaire global d'événements
    pub static ref EVENT_MANAGER: Mutex<EventManager> = Mutex::new(EventManager::new(256));
}

/// Enregistre un événement de périphérique
pub fn register_device_event(
    event_type: DeviceEventType,
    device_name: alloc::string::String,
    data: u32,
) -> Result<(), &'static str> {
    let event = DeviceEvent {
        event_type,
        device_name,
        timestamp: 0, // À remplir par le système
        data,
    };
    
    let mut manager = EVENT_MANAGER.lock();
    manager.push_event(event)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test_case]
    fn test_event_manager_creation() {
        let manager = EventManager::new(10);
        assert_eq!(manager.pending_events(), 0);
        assert_eq!(manager.processed_events(), 0);
    }
    
    #[test_case]
    fn test_push_pop_event() {
        let mut manager = EventManager::new(10);
        let event = DeviceEvent {
            event_type: DeviceEventType::Added,
            device_name: alloc::string::String::from("eth0"),
            timestamp: 0,
            data: 0,
        };
        
        assert!(manager.push_event(event.clone()).is_ok());
        assert_eq!(manager.pending_events(), 1);
        
        let popped = manager.pop_event();
        assert!(popped.is_some());
        assert_eq!(manager.pending_events(), 0);
        assert_eq!(manager.processed_events(), 1);
    }
    
    #[test_case]
    fn test_event_queue_full() {
        let mut manager = EventManager::new(2);
        let event = DeviceEvent {
            event_type: DeviceEventType::Added,
            device_name: alloc::string::String::from("eth0"),
            timestamp: 0,
            data: 0,
        };
        
        assert!(manager.push_event(event.clone()).is_ok());
        assert!(manager.push_event(event.clone()).is_ok());
        assert!(manager.push_event(event.clone()).is_err());
    }
}
