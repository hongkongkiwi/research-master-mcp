//! Progress tracking utilities for long-running operations.
//!
//! This module provides progress bar support for operations like
//! batch downloads, searches, and paper processing.
//!
//! # Usage
//!
//! ```ignore
//! use research_master_mcp::utils::ProgressReporter;
//!
//! let reporter = ProgressReporter::new("Processing papers", 100);
//! for i in 0..100 {
//!     // Do some work...
//!     reporter.inc(1);
//! }
//! reporter.finish();
//! ```

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Progress reporter with optional terminal output
///
/// Supports both quiet mode (no output) and verbose mode with progress bars.
/// Uses atomic counters for thread-safe updates.
#[derive(Debug, Clone)]
pub struct ProgressReporter {
    /// Name of the operation being tracked
    name: String,

    /// Total units of work (0 if unknown)
    total: usize,

    /// Current progress (atomic for thread safety)
    current: Arc<AtomicUsize>,

    /// Start time for calculating ETA
    start_time: Instant,

    /// Whether to show progress output
    quiet: bool,
}

impl ProgressReporter {
    /// Create a new progress reporter
    ///
    /// - `name`: Description of the operation
    /// - `total`: Total number of units of work (0 for indeterminate)
    pub fn new(name: &str, total: usize) -> Self {
        Self {
            name: name.to_string(),
            total,
            current: Arc::new(AtomicUsize::new(0)),
            start_time: Instant::now(),
            quiet: std::env::var("RESEARCH_MASTER_QUIET").is_ok(),
        }
    }

    /// Create a quiet reporter that doesn't output anything
    pub fn quiet(name: &str, total: usize) -> Self {
        Self {
            name: name.to_string(),
            total,
            current: Arc::new(AtomicUsize::new(0)),
            start_time: Instant::now(),
            quiet: true,
        }
    }

    /// Increment progress by one unit
    pub fn inc(&self) {
        self.inc_by(1);
    }

    /// Increment progress by multiple units
    pub fn inc_by(&self, delta: usize) {
        let new_value = self.current.fetch_add(delta, Ordering::SeqCst) + delta;

        if !self.quiet && new_value % 10 == 0 {
            self.print_progress(new_value);
        }
    }

    /// Set the current progress to a specific value
    pub fn set(&self, value: usize) {
        self.current.store(value, Ordering::SeqCst);

        if !self.quiet {
            self.print_progress(value);
        }
    }

    /// Print current progress
    fn print_progress(&self, current: usize) {
        let elapsed = self.start_time.elapsed();

        if self.total > 0 {
            // Deterministic progress
            let percent = (current as f64 / self.total as f64 * 100.0).min(100.0);
            let eta = self.estimate_eta(current);

            print!(
                "\r{}: [{:>3.0}%] {}/{} ({:?} elapsed, ETA: {:?})",
                self.name,
                percent,
                current,
                self.total,
                Self::format_duration(elapsed),
                eta
            );
        } else {
            // Indeterminate progress
            let dots = Self::loading_dots(current);
            print!(
                "\r{}: {} ({:?} elapsed)",
                self.name,
                dots,
                Self::format_duration(elapsed)
            );
        }

        if current >= self.total && self.total > 0 {
            println!(); // New line on completion
        } else {
            use std::io::Write;
            let _ = std::io::stdout().flush();
        }
    }

    /// Estimate time remaining
    fn estimate_eta(&self, current: usize) -> Duration {
        if current == 0 {
            return Duration::from_secs(u64::MAX);
        }

        let elapsed = self.start_time.elapsed();
        let per_unit_secs = elapsed.as_secs_f64() / current as f64;
        let remaining = self.total.saturating_sub(current);

        Duration::from_secs((per_unit_secs * remaining as f64) as u64)
    }

    /// Format duration for display
    fn format_duration(duration: Duration) -> String {
        let secs = duration.as_secs();

        if secs >= 60 {
            format!("{}m {}s", secs / 60, secs % 60)
        } else {
            format!("{}s", secs)
        }
    }

    /// Generate loading dots for indeterminate progress
    fn loading_dots(count: usize) -> String {
        let dots = count % 5;
        format!("{}{}", ".".repeat(dots), " ".repeat(4 - dots))
    }

    /// Finish the progress and print final stats
    pub fn finish(&self) {
        let current = self.current.load(Ordering::SeqCst);
        let elapsed = self.start_time.elapsed();

        if !self.quiet {
            if self.total > 0 {
                println!(
                    "{}: completed {}/{} in {:?} ({:.1} items/sec)",
                    self.name,
                    current,
                    self.total,
                    elapsed,
                    current as f64 / elapsed.as_secs_f64().max(0.001)
                );
            } else {
                println!(
                    "{}: completed {} items in {:?}",
                    self.name, current, elapsed
                );
            }
        }
    }

    /// Get the current progress count
    pub fn current(&self) -> usize {
        self.current.load(Ordering::SeqCst)
    }

    /// Check if the operation is complete
    pub fn is_done(&self) -> bool {
        let current = self.current.load(Ordering::SeqCst);
        self.total > 0 && current >= self.total
    }
}

/// Thread-safe progress tracker that can be shared across threads
#[derive(Clone)]
pub struct SharedProgress {
    /// Inner reporter
    reporter: ProgressReporter,

    /// Callback for progress updates (called from any thread)
    #[allow(dead_code)]
    callback: Option<Arc<dyn Fn(usize, usize) + Send + Sync>>,
}

impl SharedProgress {
    /// Create a new shared progress tracker
    pub fn new(name: &str, total: usize) -> Self {
        Self {
            reporter: ProgressReporter::new(name, total),
            callback: None,
        }

        // let callback = Arc::new(callback);
    }

    /// Set a callback for progress updates
    #[allow(dead_code)]
    pub fn set_callback<F>(&mut self, callback: F)
    where
        F: Fn(usize, usize) + Send + Sync + 'static,
    {
        self.callback = Some(Arc::new(callback));
    }

    /// Increment progress
    pub fn inc(&self) {
        self.reporter.inc();
    }

    /// Increment by a delta
    pub fn inc_by(&self, delta: usize) {
        self.reporter.inc_by(delta);
    }

    /// Set progress to a specific value
    pub fn set(&self, value: usize) {
        self.reporter.set(value);
    }

    /// Finish the progress
    pub fn finish(&self) {
        self.reporter.finish();
    }
}

impl std::fmt::Debug for SharedProgress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SharedProgress")
            .field("reporter", &self.reporter)
            .field(
                "callback",
                &self.callback.as_ref().map(|_| "Fn(usize, usize)"),
            )
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_reporter_creation() {
        let reporter = ProgressReporter::quiet("test", 100);
        assert_eq!(reporter.total, 100);
        assert!(reporter.quiet);
    }

    #[test]
    fn test_progress_reporter_increment() {
        let reporter = ProgressReporter::quiet("test", 100);
        reporter.inc();
        assert_eq!(reporter.current(), 1);

        reporter.inc_by(5);
        assert_eq!(reporter.current(), 6);
    }

    #[test]
    fn test_progress_reporter_set() {
        let reporter = ProgressReporter::quiet("test", 100);
        reporter.set(50);
        assert_eq!(reporter.current(), 50);
    }

    #[test]
    fn test_progress_reporter_is_done() {
        let reporter = ProgressReporter::quiet("test", 10);
        assert!(!reporter.is_done());

        reporter.set(5);
        assert!(!reporter.is_done());

        reporter.set(10);
        assert!(reporter.is_done());
    }

    #[test]
    fn test_progress_reporter_zero_total() {
        let reporter = ProgressReporter::quiet("test", 0);
        assert!(!reporter.is_done());

        reporter.inc();
        assert!(!reporter.is_done()); // Never done when total is 0
    }

    #[test]
    fn test_shared_progress() {
        let progress = SharedProgress::new("test", 100);
        progress.inc();
        assert_eq!(progress.reporter.current(), 1);

        progress.inc_by(10);
        assert_eq!(progress.reporter.current(), 11);

        progress.set(50);
        assert_eq!(progress.reporter.current(), 50);
    }
}
