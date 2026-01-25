//! History tracking for searches and downloads.
//!
//! This module provides simple history tracking stored in the config directory.

use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// History entry type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HistoryEntryType {
    /// Search query
    Search,
    /// Paper download
    Download,
    /// Paper viewed/read
    View,
}

/// A single history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// Type of entry
    pub entry_type: HistoryEntryType,
    /// Timestamp (Unix epoch)
    pub timestamp: u64,
    /// Query or paper ID
    pub query: String,
    /// Source (if applicable)
    pub source: Option<String>,
    /// Paper title (for downloads/views)
    pub title: Option<String>,
    /// Additional details
    pub details: Option<String>,
}

/// History service
#[derive(Debug, Clone)]
pub struct HistoryService {
    /// History file path
    path: PathBuf,
}

impl HistoryService {
    /// Create a new history service
    pub fn new() -> Self {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("~/.config"))
            .join("research-master");
        let path = config_dir.join("history.jsonl");
        Self { path }
    }

    /// Ensure history file exists
    fn ensure_file(&self) -> io::Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        if !self.path.exists() {
            File::create(&self.path)?;
        }
        Ok(())
    }

    /// Add a search entry
    pub fn add_search(&self, query: &str, source: Option<&str>) -> io::Result<()> {
        self.ensure_file()?;
        let entry = HistoryEntry {
            entry_type: HistoryEntryType::Search,
            timestamp: now(),
            query: query.to_string(),
            source: source.map(|s| s.to_string()),
            title: None,
            details: None,
        };
        self.append_entry(&entry)
    }

    /// Add a download entry
    pub fn add_download(
        &self,
        paper_id: &str,
        source: &str,
        title: Option<&str>,
        path: Option<&str>,
    ) -> io::Result<()> {
        self.ensure_file()?;
        let entry = HistoryEntry {
            entry_type: HistoryEntryType::Download,
            timestamp: now(),
            query: paper_id.to_string(),
            source: Some(source.to_string()),
            title: title.map(|s| s.to_string()),
            details: path.map(|s| s.to_string()),
        };
        self.append_entry(&entry)
    }

    /// Add a view entry
    pub fn add_view(&self, paper_id: &str, source: &str, title: Option<&str>) -> io::Result<()> {
        self.ensure_file()?;
        let entry = HistoryEntry {
            entry_type: HistoryEntryType::View,
            timestamp: now(),
            query: paper_id.to_string(),
            source: Some(source.to_string()),
            title: title.map(|s| s.to_string()),
            details: None,
        };
        self.append_entry(&entry)
    }

    /// Append an entry to the history file
    fn append_entry(&self, entry: &HistoryEntry) -> io::Result<()> {
        let mut file = fs::OpenOptions::new().append(true).open(&self.path)?;
        let json = serde_json::to_string(entry)?;
        writeln!(file, "{}", json)?;
        Ok(())
    }

    /// Read history entries
    pub fn read_entries(&self, limit: usize) -> io::Result<Vec<HistoryEntry>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);
        let mut entries: Vec<HistoryEntry> = Vec::new();

        for line in reader.lines().take(limit * 2) {
            let line = line?;
            if let Ok(entry) = serde_json::from_str(&line) {
                entries.push(entry);
            }
        }

        // Reverse to get newest first, then take limit
        entries.reverse();
        entries.truncate(limit);

        Ok(entries)
    }

    /// Filter entries by type
    pub fn filter_entries(
        &self,
        entries: &[HistoryEntry],
        entry_type: HistoryEntryType,
    ) -> Vec<HistoryEntry> {
        entries
            .iter()
            .filter(|e| e.entry_type == entry_type)
            .cloned()
            .collect()
    }

    /// Clear history
    pub fn clear(&self) -> io::Result<()> {
        if self.path.exists() {
            fs::remove_file(&self.path)?;
        }
        self.ensure_file()?;
        Ok(())
    }

    /// Get history file path (for external access)
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Default for HistoryService {
    fn default() -> Self {
        Self::new()
    }
}

/// Get current timestamp
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
