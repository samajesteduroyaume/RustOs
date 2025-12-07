/// Module Write-Back Daemon
/// 
/// Gère l'écriture asynchrone des blocs dirty vers le disque

use alloc::vec::Vec;
use spin::Mutex;
use super::buffer::BUFFER_CACHE;

/// Mode d'écriture
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteMode {
    /// Écriture immédiate (synchrone)
    WriteThrough,
    /// Écriture différée (asynchrone)
    WriteBack,
    /// Écriture ordonnée (métadonnées avant données)
    WriteOrdered,
}

/// Configuration du write-back daemon
#[derive(Debug, Clone)]
pub struct WriteBackConfig {
    /// Mode d'écriture
    pub mode: WriteMode,
    /// Intervalle de flush en ticks (5 secondes = 5000 ticks à 1ms)
    pub flush_interval: usize,
    /// Nombre maximum de blocs dirty avant flush forcé
    pub max_dirty_blocks: usize,
    /// Activer le daemon
    pub enabled: bool,
}

impl WriteBackConfig {
    /// Configuration par défaut
    pub fn default() -> Self {
        Self {
            mode: WriteMode::WriteBack,
            flush_interval: 5000, // 5 secondes
            max_dirty_blocks: 100,
            enabled: true,
        }
    }
}

/// Write-Back Daemon
pub struct WriteBackDaemon {
    /// Configuration
    config: WriteBackConfig,
    /// Compteur de ticks
    tick_count: usize,
    /// Nombre de flush effectués
    flush_count: usize,
    /// Nombre de blocs écrits
    blocks_written: usize,
}

impl WriteBackDaemon {
    /// Crée un nouveau daemon
    pub const fn new() -> Self {
        Self {
            config: WriteBackConfig {
                mode: WriteMode::WriteBack,
                flush_interval: 5000,
                max_dirty_blocks: 100,
                enabled: true,
            },
            tick_count: 0,
            flush_count: 0,
            blocks_written: 0,
        }
    }
    
    /// Configure le daemon
    pub fn configure(&mut self, config: WriteBackConfig) {
        self.config = config;
    }
    
    /// Tick du daemon (appelé périodiquement, ex: toutes les 1ms)
    pub fn tick(&mut self) {
        if !self.config.enabled {
            return;
        }
        
        self.tick_count += 1;
        
        // Vérifier si on doit faire un flush périodique
        if self.tick_count >= self.config.flush_interval {
            self.flush_dirty_blocks();
            self.tick_count = 0;
        }
        
        // Vérifier si on a trop de blocs dirty
        let stats = BUFFER_CACHE.lock().get_stats();
        if stats.dirty_blocks >= self.config.max_dirty_blocks {
            self.flush_dirty_blocks();
        }
    }
    
    /// Flush tous les blocs dirty
    pub fn flush_dirty_blocks(&mut self) {
        let mut cache = BUFFER_CACHE.lock();
        let blocks = cache.flush_all();
        
        // TODO: Écrire réellement les blocs sur le disque
        // Pour l'instant, juste compter
        self.blocks_written += blocks.len();
        self.flush_count += 1;
        
        drop(cache);
    }
    
    /// Flush un bloc spécifique
    pub fn flush_block(&mut self, block_num: u64) {
        let mut cache = BUFFER_CACHE.lock();
        if let Some(_data) = cache.flush_block(block_num) {
            // TODO: Écrire le bloc sur le disque
            self.blocks_written += 1;
        }
        drop(cache);
    }
    
    /// Sync - Force l'écriture de tous les blocs dirty
    pub fn sync(&mut self) {
        self.flush_dirty_blocks();
    }
    
    /// Retourne les statistiques
    pub fn get_stats(&self) -> WriteBackStats {
        WriteBackStats {
            flush_count: self.flush_count,
            blocks_written: self.blocks_written,
            tick_count: self.tick_count,
            enabled: self.config.enabled,
            mode: self.config.mode,
        }
    }
}

/// Statistiques du write-back daemon
#[derive(Debug, Clone)]
pub struct WriteBackStats {
    pub flush_count: usize,
    pub blocks_written: usize,
    pub tick_count: usize,
    pub enabled: bool,
    pub mode: WriteMode,
}

/// Instance globale du write-back daemon
use lazy_static::lazy_static;

lazy_static! {
    pub static ref WRITEBACK_DAEMON: Mutex<WriteBackDaemon> = Mutex::new(WriteBackDaemon::new());
}

/// Fonction de sync globale (à appeler depuis syscall sync())
pub fn sync_all() {
    WRITEBACK_DAEMON.lock().sync();
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test_case]
    fn test_writeback_daemon_creation() {
        let daemon = WriteBackDaemon::new();
        assert!(daemon.config.enabled);
        assert_eq!(daemon.flush_count, 0);
    }
    
    #[test_case]
    fn test_tick() {
        let mut daemon = WriteBackDaemon::new();
        daemon.config.flush_interval = 10;
        
        // Tick 9 fois, pas de flush
        for _ in 0..9 {
            daemon.tick();
        }
        assert_eq!(daemon.flush_count, 0);
        
        // Tick 10, flush
        daemon.tick();
        assert_eq!(daemon.flush_count, 1);
    }
    
    #[test_case]
    fn test_sync() {
        let mut daemon = WriteBackDaemon::new();
        daemon.sync();
        assert_eq!(daemon.flush_count, 1);
    }
}
