// Incremental indexer and recursive file watcher.

pub mod service;
pub mod watcher;

pub use service::{ChangeKind, FileIndex, IndexChangeRecord, ScanResult};
pub use watcher::FileWatcher;
