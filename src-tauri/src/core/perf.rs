//! Lightweight performance instrumentation for hot-path operations.
//!
//! Provides a [`PerfTimer`] that measures elapsed wall-clock time and logs
//! the result when dropped.  Also provides the [`time_operation`] helper for
//! one-shot measurements that return both the result and duration.
//!
//! All output goes through `eprintln!` in debug builds only, keeping release
//! binaries silent unless explicitly opted-in via the `VIGIL_PERF` env var.
//!
//! # Instrumentation requirements (from `editor-performance-budget.md`)
//!
//! Rust timers around:
//! - workspace scan
//! - fuzzy search
//! - content search
//! - git hunk computation

use std::time::{Duration, Instant};

/// A simple RAII timer that records the start time on creation and logs the
/// elapsed duration when dropped.
///
/// ```ignore
/// let _t = PerfTimer::start("full_scan");
/// // … work …
/// // prints on drop: [perf] full_scan: 12.345 ms
/// ```
pub struct PerfTimer {
    label: &'static str,
    start: Instant,
}

impl PerfTimer {
    /// Start a new timer with the given human-readable label.
    #[inline]
    pub fn start(label: &'static str) -> Self {
        Self {
            label,
            start: Instant::now(),
        }
    }

    /// Return the elapsed time since the timer was started without stopping it.
    #[inline]
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Return the elapsed time in milliseconds (fractional).
    #[inline]
    pub fn elapsed_ms(&self) -> f64 {
        self.start.elapsed().as_secs_f64() * 1000.0
    }
}

impl Drop for PerfTimer {
    fn drop(&mut self) {
        let elapsed = self.start.elapsed();
        let should_log = cfg!(debug_assertions) || std::env::var("VIGIL_PERF").is_ok();
        if should_log {
            eprintln!(
                "[perf] {}: {:.3} ms",
                self.label,
                elapsed.as_secs_f64() * 1000.0,
            );
        }
    }
}

/// Execute `f`, measure its duration, and return `(result, duration)`.
///
/// Unlike [`PerfTimer`] this does **not** log automatically; the caller
/// decides what to do with the duration.
#[inline]
pub fn time_operation<F, R>(f: F) -> (R, Duration)
where
    F: FnOnce() -> R,
{
    let start = Instant::now();
    let result = f();
    let elapsed = start.elapsed();
    (result, elapsed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn perf_timer_measures_elapsed() {
        let timer = PerfTimer::start("test_timer");
        thread::sleep(Duration::from_millis(5));
        let ms = timer.elapsed_ms();
        assert!(ms >= 4.0, "expected at least 4 ms, got {ms}");
    }

    #[test]
    fn time_operation_returns_result_and_duration() {
        let (result, duration) = time_operation(|| {
            thread::sleep(Duration::from_millis(5));
            42
        });
        assert_eq!(result, 42);
        assert!(duration.as_millis() >= 4, "expected at least 4 ms");
    }
}
