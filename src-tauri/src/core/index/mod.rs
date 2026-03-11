// Incremental indexer, tag index, and recursive file watcher.

pub mod metrics;
pub mod service;
pub mod tag_index;
pub mod watcher;

pub use service::{ChangeKind, FileIndex, IndexChangeRecord, ScanResult};
pub use tag_index::TagIndex;
pub use watcher::FileWatcher;
