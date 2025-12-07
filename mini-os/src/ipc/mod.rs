/// Module IPC (Inter-Process Communication)
/// 
/// Mécanismes de communication entre processus

pub mod pipe;
pub mod mqueue;
pub mod semaphore;

pub use pipe::{Pipe, PipeManager, PIPE_MANAGER, PIPE_BUF_SIZE};
pub use mqueue::{MessageQueue, MessageQueueManager, Message, Priority, MQ_MANAGER};
pub use semaphore::{Semaphore, SemaphoreManager, SEM_MANAGER};
