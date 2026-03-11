//! Recursive file watcher using the `notify` crate with debounced events.
//!
//! The watcher runs on a dedicated thread and dispatches debounced filesystem
//! events to a callback. Events are coalesced over a 150ms window using
//! `crossbeam-channel` to avoid flooding the index with rapid-fire changes.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crossbeam_channel::{Receiver, Sender};
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use super::service::ChangeKind;

/// Debounce window for coalescing filesystem events.
const DEBOUNCE_MS: u64 = 150;

/// A debounced filesystem change event.
#[derive(Debug, Clone)]
pub struct FsChangeEvent {
    /// Absolute path to the changed file/directory.
    pub path: PathBuf,
    /// Classified change kind.
    pub kind: ChangeKind,
}

/// Manages a recursive filesystem watcher on a workspace root.
///
/// Call `start()` to begin watching and provide a callback that receives
/// batches of debounced change events. Call `stop()` or drop to tear down.
pub struct FileWatcher {
    /// Signal to stop the watcher thread.
    stop_flag: Arc<AtomicBool>,
    /// The underlying notify watcher; kept alive as long as this struct lives.
    _watcher: RecommendedWatcher,
}

impl FileWatcher {
    /// Start watching `root` recursively.
    ///
    /// `on_changes` is called on a background thread with batches of debounced
    /// events. The callback should update the index and emit Tauri events.
    pub fn start<F>(root: &Path, on_changes: F) -> Result<Self, notify::Error>
    where
        F: Fn(Vec<FsChangeEvent>) + Send + 'static,
    {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let stop_flag_clone = stop_flag.clone();

        // Channel for raw notify events -> debounce thread.
        let (raw_tx, raw_rx): (Sender<notify::Event>, Receiver<notify::Event>) =
            crossbeam_channel::unbounded();

        // Set up the notify watcher.
        let watcher =
            notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| {
                if let Ok(event) = res {
                    let _ = raw_tx.send(event);
                }
            })?;

        let mut watcher = watcher;
        watcher.watch(root, RecursiveMode::Recursive)?;

        // Spawn the debounce thread.
        std::thread::Builder::new()
            .name("vigil-fs-watcher".into())
            .spawn(move || {
                debounce_loop(raw_rx, stop_flag_clone, on_changes);
            })
            .expect("failed to spawn watcher thread");

        Ok(Self {
            stop_flag,
            _watcher: watcher,
        })
    }

    /// Signal the watcher to stop.
    pub fn stop(&self) {
        self.stop_flag.store(true, Ordering::SeqCst);
    }
}

impl Drop for FileWatcher {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Classify a `notify::EventKind` into our `ChangeKind`.
fn classify_event(kind: &EventKind) -> Option<ChangeKind> {
    match kind {
        EventKind::Create(_) => Some(ChangeKind::Created),
        EventKind::Modify(_) => Some(ChangeKind::Changed),
        EventKind::Remove(_) => Some(ChangeKind::Deleted),
        _ => None,
    }
}

/// Debounce loop: collects raw events, coalesces by path over a time window,
/// then dispatches batches to the callback.
fn debounce_loop<F>(raw_rx: Receiver<notify::Event>, stop_flag: Arc<AtomicBool>, on_changes: F)
where
    F: Fn(Vec<FsChangeEvent>),
{
    let debounce_duration = Duration::from_millis(DEBOUNCE_MS);

    // Map from path -> (latest change kind, last event time).
    let mut pending: HashMap<PathBuf, (ChangeKind, Instant)> = HashMap::new();

    loop {
        if stop_flag.load(Ordering::SeqCst) {
            break;
        }

        // Wait for the next event, with a short timeout for drain/flush checks.
        match raw_rx.recv_timeout(Duration::from_millis(50)) {
            Ok(event) => {
                if let Some(change_kind) = classify_event(&event.kind) {
                    for path in event.paths {
                        pending.insert(path, (change_kind, Instant::now()));
                    }
                }
            }
            Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                // Check for ready events below.
            }
            Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
                break;
            }
        }

        // Drain any additional queued events without blocking.
        while let Ok(event) = raw_rx.try_recv() {
            if let Some(change_kind) = classify_event(&event.kind) {
                for path in event.paths {
                    pending.insert(path, (change_kind, Instant::now()));
                }
            }
        }

        // Flush events that have been stable for the debounce window.
        let now = Instant::now();
        let ready: Vec<FsChangeEvent> = pending
            .iter()
            .filter(|(_, (_, last))| now.duration_since(*last) >= debounce_duration)
            .map(|(path, (kind, _))| FsChangeEvent {
                path: path.clone(),
                kind: *kind,
            })
            .collect();

        if !ready.is_empty() {
            for event in &ready {
                pending.remove(&event.path);
            }
            on_changes(ready);
        }
    }

    // Flush any remaining pending events on shutdown.
    if !pending.is_empty() {
        let final_batch: Vec<FsChangeEvent> = pending
            .into_iter()
            .map(|(path, (kind, _))| FsChangeEvent { path, kind })
            .collect();
        on_changes(final_batch);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_create_event() {
        let kind = EventKind::Create(notify::event::CreateKind::File);
        assert_eq!(classify_event(&kind), Some(ChangeKind::Created));
    }

    #[test]
    fn classify_modify_event() {
        let kind = EventKind::Modify(notify::event::ModifyKind::Data(
            notify::event::DataChange::Content,
        ));
        assert_eq!(classify_event(&kind), Some(ChangeKind::Changed));
    }

    #[test]
    fn classify_remove_event() {
        let kind = EventKind::Remove(notify::event::RemoveKind::File);
        assert_eq!(classify_event(&kind), Some(ChangeKind::Deleted));
    }

    #[test]
    fn classify_other_event_is_none() {
        let kind = EventKind::Access(notify::event::AccessKind::Read);
        assert_eq!(classify_event(&kind), None);
    }
}
