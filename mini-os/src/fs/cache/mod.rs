/// Module de cache pour le système de fichiers

pub mod buffer;
pub mod writeback;
pub mod readahead;

pub use buffer::{BufferCache, BufferCacheEntry, BufferCacheStats, BUFFER_CACHE, BLOCK_SIZE};
pub use writeback::{WriteBackDaemon, WriteBackConfig, WriteBackStats, WriteMode, WRITEBACK_DAEMON, sync_all};
pub use readahead::{ReadAheadManager, ReadAheadStats, READAHEAD_MANAGER};
