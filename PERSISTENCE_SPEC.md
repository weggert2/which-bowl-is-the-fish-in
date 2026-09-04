# Persistence System Technical Specification

**Project:** Which Bowl is the Fish In?  
**Version:** 1.0  
**Date:** 2026-09-04

---

## 1. Overview

This document specifies the persistence system for the "Which Bowl is the Fish In?" game. The system manages saving and loading player progress, including fish discoveries, statistics, and game history using a binary serialization format (bincode).

### 1.1 Goals

- Persist player progress across game sessions
- Track fish discovery journal with timestamps
- Maintain game statistics (guesses, correct answers, play time)
- Provide atomic, corruption-resistant file operations
- Support cross-platform save file locations
- Enable future data migration through versioning

### 1.2 Non-Goals

- Cloud synchronization
- Multiple save slots
- Encrypted save files
- Undo/redo history

---

## 2. Module Structure

The persistence system is organized into two primary modules:

```
src/persistence/
├── mod.rs          # PersistenceManager and public API
└── journal.rs      # Journal and DiscoveryEntry types
```

### 2.1 Module Dependencies

```rust
// External dependencies (Cargo.toml)
[dependencies]
serde = { version = "1.0", features = ["derive"] }
bincode = "1.3"
chrono = { version = "0.4", features = ["serde"] }
dirs = "5.0"  # For platform-specific config directories
```

---

## 3. Journal Data Structure

### 3.1 Journal Struct

The `Journal` struct is the top-level container for all persistent game data.

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Journal {
    /// Schema version for future migrations
    pub version: u32,
    
    /// Map of fish_id -> discovery entry
    pub discoveries: HashMap<String, DiscoveryEntry>,
    
    /// Aggregate statistics
    pub stats: GameStats,
}

impl Journal {
    pub fn new() -> Self {
        Self {
            version: 1,
            discoveries: HashMap::new(),
            stats: GameStats::default(),
        }
    }
}

impl Default for Journal {
    fn default() -> Self {
        Self::new()
    }
}
```

### 3.2 DiscoveryEntry Struct

Each discovered fish has an associated `DiscoveryEntry` tracking when and how often it was found.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryEntry {
    /// Unique identifier for the fish species
    pub fish_id: String,
    
    /// UTC timestamp of first discovery
    pub discovered_at: DateTime<Utc>,
    
    /// Total number of times this fish has been found
    pub times_found: u32,
    
    /// UTC timestamp of most recent discovery
    pub last_found_at: DateTime<Utc>,
}

impl DiscoveryEntry {
    pub fn new(fish_id: String) -> Self {
        let now = Utc::now();
        Self {
            fish_id,
            discovered_at: now,
            times_found: 1,
            last_found_at: now,
        }
    }
    
    /// Record another discovery of this fish
    pub fn increment(&mut self) {
        self.times_found += 1;
        self.last_found_at = Utc::now();
    }
}
```

### 3.3 GameStats Struct

Aggregate statistics across all gameplay sessions.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GameStats {
    /// Total number of guesses made (right or wrong)
    pub total_guesses: u64,
    
    /// Number of correct guesses (found fish)
    pub correct_guesses: u64,
    
    /// Number of incorrect guesses (empty bowls)
    pub incorrect_guesses: u64,
    
    /// UTC timestamp of first game session
    pub first_played: Option<DateTime<Utc>>,
    
    /// UTC timestamp of most recent game session
    pub last_played: Option<DateTime<Utc>>,
    
    /// Total playtime in seconds (tracked while app is active)
    pub total_playtime_seconds: u64,
}

impl GameStats {
    pub fn accuracy_percentage(&self) -> f64 {
        if self.total_guesses == 0 {
            return 0.0;
        }
        (self.correct_guesses as f64 / self.total_guesses as f64) * 100.0
    }
}
```

---

## 4. Journal API

### 4.1 Core Methods

```rust
impl Journal {
    /// Record a fish discovery (new or repeated)
    pub fn discover_fish(&mut self, fish_id: String) {
        self.stats.total_guesses += 1;
        self.stats.correct_guesses += 1;
        self.stats.last_played = Some(Utc::now());
        
        if self.stats.first_played.is_none() {
            self.stats.first_played = Some(Utc::now());
        }
        
        self.discoveries
            .entry(fish_id.clone())
            .and_modify(|entry| entry.increment())
            .or_insert_with(|| DiscoveryEntry::new(fish_id));
    }
    
    /// Record a wrong guess (empty bowl)
    pub fn record_wrong_guess(&mut self) {
        self.stats.total_guesses += 1;
        self.stats.incorrect_guesses += 1;
        self.stats.last_played = Some(Utc::now());
        
        if self.stats.first_played.is_none() {
            self.stats.first_played = Some(Utc::now());
        }
    }
    
    /// Check if a fish has been discovered
    pub fn has_discovered(&self, fish_id: &str) -> bool {
        self.discoveries.contains_key(fish_id)
    }
    
    /// Get discovery entry for a fish (if discovered)
    pub fn get_discovery(&self, fish_id: &str) -> Option<&DiscoveryEntry> {
        self.discoveries.get(fish_id)
    }
    
    /// Calculate completion percentage based on total fish count
    pub fn completion_percentage(&self, total_fish_count: usize) -> f64 {
        if total_fish_count == 0 {
            return 0.0;
        }
        (self.discoveries.len() as f64 / total_fish_count as f64) * 100.0
    }
    
    /// Get number of unique fish discovered
    pub fn discovered_count(&self) -> usize {
        self.discoveries.len()
    }
    
    /// Get list of all discovered fish IDs, sorted by discovery date
    pub fn discovered_fish_ids(&self) -> Vec<String> {
        let mut entries: Vec<_> = self.discoveries.values().collect();
        entries.sort_by_key(|e| e.discovered_at);
        entries.iter().map(|e| e.fish_id.clone()).collect()
    }
    
    /// Add playtime (called periodically while app is running)
    pub fn add_playtime(&mut self, seconds: u64) {
        self.stats.total_playtime_seconds += seconds;
    }
}
```

---

## 5. PersistenceManager

### 5.1 Manager Struct

The `PersistenceManager` handles file I/O, save path resolution, and debounced writes.

```rust
use std::path::PathBuf;
use std::time::{Duration, Instant};

pub struct PersistenceManager {
    /// Path to the save file
    save_path: PathBuf,
    
    /// In-memory journal
    journal: Journal,
    
    /// Last time a change was made (for debouncing)
    last_change: Option<Instant>,
    
    /// Whether there are unsaved changes
    dirty: bool,
    
    /// Debounce delay (default: 2 seconds)
    debounce_delay: Duration,
}
```

### 5.2 Initialization

```rust
impl PersistenceManager {
    /// Create a new PersistenceManager and attempt to load existing save
    pub fn new() -> Result<Self, PersistenceError> {
        let save_path = Self::determine_save_path()?;
        let journal = Self::load_from_path(&save_path)?;
        
        Ok(Self {
            save_path,
            journal,
            last_change: None,
            dirty: false,
            debounce_delay: Duration::from_secs(2),
        })
    }
    
    /// Determine platform-specific save file path
    fn determine_save_path() -> Result<PathBuf, PersistenceError> {
        let config_dir = dirs::config_dir()
            .ok_or(PersistenceError::NoConfigDirectory)?;
        
        let app_dir = config_dir.join("which-bowl");
        
        // Create directory if it doesn't exist
        if !app_dir.exists() {
            std::fs::create_dir_all(&app_dir)
                .map_err(|e| PersistenceError::DirectoryCreation(e))?;
        }
        
        Ok(app_dir.join("journal.bin"))
    }
    
    /// Get immutable reference to journal
    pub fn journal(&self) -> &Journal {
        &self.journal
    }
    
    /// Get mutable reference to journal and mark as dirty
    pub fn journal_mut(&mut self) -> &mut Journal {
        self.mark_dirty();
        &mut self.journal
    }
    
    /// Mark that changes have been made
    fn mark_dirty(&mut self) {
        self.dirty = true;
        self.last_change = Some(Instant::now());
    }
}
```

### 5.3 Platform-Specific Paths

The save location varies by operating system:

- **Linux:** `~/.config/which-bowl/journal.bin`
- **macOS:** `~/Library/Application Support/which-bowl/journal.bin`
- **Windows:** `%APPDATA%\which-bowl\journal.bin`

These paths are determined automatically using the `dirs` crate.

---

## 6. Save/Load Operations

### 6.1 Loading

```rust
impl PersistenceManager {
    /// Load journal from disk, or create new if file doesn't exist
    fn load_from_path(path: &PathBuf) -> Result<Journal, PersistenceError> {
        if !path.exists() {
            // First run - create new journal
            return Ok(Journal::new());
        }
        
        let bytes = std::fs::read(path)
            .map_err(|e| PersistenceError::ReadFailed(e))?;
        
        // Attempt to deserialize
        match bincode::deserialize::<Journal>(&bytes) {
            Ok(journal) => {
                // Validate version
                if journal.version > 1 {
                    return Err(PersistenceError::UnsupportedVersion(journal.version));
                }
                Ok(journal)
            }
            Err(_) => {
                // Corrupted file - create backup and start fresh
                Self::create_backup(path)?;
                Ok(Journal::new())
            }
        }
    }
    
    /// Force reload from disk (discards unsaved changes)
    pub fn reload(&mut self) -> Result<(), PersistenceError> {
        self.journal = Self::load_from_path(&self.save_path)?;
        self.dirty = false;
        self.last_change = None;
        Ok(())
    }
}
```

### 6.2 Saving

```rust
impl PersistenceManager {
    /// Save journal to disk immediately
    pub fn save(&mut self) -> Result<(), PersistenceError> {
        if !self.dirty {
            return Ok(());  // No changes to save
        }
        
        // Serialize to bytes
        let bytes = bincode::serialize(&self.journal)
            .map_err(|e| PersistenceError::SerializationFailed(e.to_string()))?;
        
        // Atomic write: write to temp file, then rename
        let temp_path = self.save_path.with_extension("tmp");
        
        std::fs::write(&temp_path, &bytes)
            .map_err(|e| PersistenceError::WriteFailed(e))?;
        
        std::fs::rename(&temp_path, &self.save_path)
            .map_err(|e| PersistenceError::WriteFailed(e))?;
        
        self.dirty = false;
        self.last_change = None;
        
        Ok(())
    }
    
    /// Save only if debounce period has elapsed
    pub fn save_if_needed(&mut self) -> Result<(), PersistenceError> {
        if !self.dirty {
            return Ok(());
        }
        
        if let Some(last_change) = self.last_change {
            if last_change.elapsed() >= self.debounce_delay {
                return self.save();
            }
        }
        
        Ok(())
    }
    
    /// Force save regardless of debounce (call on app exit)
    pub fn flush(&mut self) -> Result<(), PersistenceError> {
        if self.dirty {
            self.save()
        } else {
            Ok(())
        }
    }
    
    /// Create backup of existing save file
    fn create_backup(path: &PathBuf) -> Result<(), PersistenceError> {
        if !path.exists() {
            return Ok(());
        }
        
        let backup_path = path.with_extension("bak");
        std::fs::copy(path, backup_path)
            .map_err(|e| PersistenceError::BackupFailed(e))?;
        
        Ok(())
    }
}
```

---

## 7. Debounced Saving

### 7.1 Strategy

To avoid excessive disk I/O, saves are debounced with a 2-second delay:

1. When journal is modified, mark as dirty and record timestamp
2. Periodically call `save_if_needed()` (e.g., in main game loop)
3. If 2+ seconds have elapsed since last change, write to disk
4. On app exit, call `flush()` to ensure all changes are saved

### 7.2 Integration Example

```rust
// In main game loop (egui update function)
impl MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // ... game logic ...
        
        // Attempt debounced save (non-blocking)
        if let Err(e) = self.persistence.save_if_needed() {
            eprintln!("Save failed: {}", e);
        }
        
        // Request repaint to keep checking for save opportunities
        ctx.request_repaint();
    }
}

// On app exit
impl eframe::App for MyApp {
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if let Err(e) = self.persistence.flush() {
            eprintln!("Failed to save on exit: {}", e);
        }
    }
}
```

### 7.3 Manual Save Trigger

Optionally provide a "Save Game" button for explicit saves:

```rust
if ui.button("Save Game").clicked() {
    match persistence.save() {
        Ok(_) => {
            // Show success notification
        }
        Err(e) => {
            // Show error dialog
        }
    }
}
```

---

## 8. Error Handling

### 8.1 Error Types

```rust
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PersistenceError {
    #[error("No config directory found for this platform")]
    NoConfigDirectory,
    
    #[error("Failed to create directory: {0}")]
    DirectoryCreation(#[source] io::Error),
    
    #[error("Failed to read save file: {0}")]
    ReadFailed(#[source] io::Error),
    
    #[error("Failed to write save file: {0}")]
    WriteFailed(#[source] io::Error),
    
    #[error("Serialization failed: {0}")]
    SerializationFailed(String),
    
    #[error("Failed to create backup: {0}")]
    BackupFailed(#[source] io::Error),
    
    #[error("Unsupported save file version: {0}")]
    UnsupportedVersion(u32),
}

pub type Result<T> = std::result::Result<T, PersistenceError>;
```

### 8.2 Error Recovery Strategies

| Error | Recovery Strategy |
|-------|------------------|
| `NoConfigDirectory` | Fall back to current directory + `.which-bowl/` |
| `DirectoryCreation` | Display error to user, run without persistence |
| `ReadFailed` (permissions) | Display error, run without persistence |
| `ReadFailed` (corruption) | Create backup, start with fresh journal |
| `WriteFailed` (disk full) | Warn user, continue running with unsaved changes |
| `WriteFailed` (permissions) | Warn user, suggest alternative save location |
| `UnsupportedVersion` | Cannot read future versions - display error and exit |

### 8.3 Corruption Handling

When a corrupted save file is detected:

1. Create backup: `journal.bak`
2. Log warning with backup path
3. Start with fresh `Journal::new()`
4. Continue normally

```rust
fn load_from_path(path: &PathBuf) -> Result<Journal> {
    // ... read file ...
    
    match bincode::deserialize::<Journal>(&bytes) {
        Ok(journal) => Ok(journal),
        Err(e) => {
            // Corruption detected
            eprintln!("Warning: Save file corrupted. Creating backup.");
            Self::create_backup(path)?;
            eprintln!("Backup saved to: {:?}", path.with_extension("bak"));
            Ok(Journal::new())
        }
    }
}
```

---

## 9. File Format

### 9.1 Binary Format (bincode)

The save file uses `bincode` serialization:

- **Compact:** 10-100x smaller than JSON
- **Fast:** Microsecond serialize/deserialize times
- **Type-safe:** Enforced by Rust's type system
- **Not human-readable:** Trade-off for performance

### 9.2 Example File Structure

```
[4 bytes]   version: u32 = 1
[variable]  discoveries: HashMap<String, DiscoveryEntry>
  - [4 bytes]   map length
  - For each entry:
    - [variable] fish_id: String
    - [8 bytes]  discovered_at: i64 (Unix timestamp)
    - [4 bytes]  times_found: u32
    - [8 bytes]  last_found_at: i64
[variable]  stats: GameStats
  - [8 bytes]  total_guesses: u64
  - [8 bytes]  correct_guesses: u64
  - [8 bytes]  incorrect_guesses: u64
  - [1 byte]   first_played: Option<DateTime> (Some = 1, None = 0)
  - [8 bytes]  first_played timestamp (if Some)
  - [1 byte]   last_played: Option<DateTime>
  - [8 bytes]  last_played timestamp (if Some)
  - [8 bytes]  total_playtime_seconds: u64
```

Typical file size: **500 bytes - 5 KB** (depending on number of discoveries)

### 9.3 Version Migration

The `version` field enables future migrations:

```rust
impl Journal {
    fn migrate(&mut self) {
        match self.version {
            1 => {
                // Current version - no migration needed
            }
            2 => {
                // Future: migrate v1 -> v2
                // Example: add new fields with defaults
            }
            _ => {
                eprintln!("Unknown version: {}", self.version);
            }
        }
    }
}
```

---

## 10. Testing Strategy

### 10.1 Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_new_journal() {
        let journal = Journal::new();
        assert_eq!(journal.version, 1);
        assert_eq!(journal.discoveries.len(), 0);
        assert_eq!(journal.stats.total_guesses, 0);
    }
    
    #[test]
    fn test_discover_fish() {
        let mut journal = Journal::new();
        journal.discover_fish("clownfish".to_string());
        
        assert_eq!(journal.discovered_count(), 1);
        assert!(journal.has_discovered("clownfish"));
        assert_eq!(journal.stats.correct_guesses, 1);
    }
    
    #[test]
    fn test_discover_same_fish_twice() {
        let mut journal = Journal::new();
        journal.discover_fish("clownfish".to_string());
        journal.discover_fish("clownfish".to_string());
        
        assert_eq!(journal.discovered_count(), 1);  // Still 1 unique fish
        assert_eq!(journal.stats.correct_guesses, 2);  // 2 total finds
        
        let entry = journal.get_discovery("clownfish").unwrap();
        assert_eq!(entry.times_found, 2);
    }
    
    #[test]
    fn test_completion_percentage() {
        let mut journal = Journal::new();
        journal.discover_fish("fish1".to_string());
        journal.discover_fish("fish2".to_string());
        
        assert_eq!(journal.completion_percentage(10), 20.0);
    }
    
    #[test]
    fn test_accuracy_percentage() {
        let mut journal = Journal::new();
        journal.discover_fish("fish1".to_string());
        journal.record_wrong_guess();
        journal.record_wrong_guess();
        
        assert_eq!(journal.stats.accuracy_percentage(), 33.333333333333336);
    }
    
    #[test]
    fn test_serialization_roundtrip() {
        let mut journal = Journal::new();
        journal.discover_fish("clownfish".to_string());
        journal.record_wrong_guess();
        
        let bytes = bincode::serialize(&journal).unwrap();
        let deserialized: Journal = bincode::deserialize(&bytes).unwrap();
        
        assert_eq!(journal.version, deserialized.version);
        assert_eq!(journal.discovered_count(), deserialized.discovered_count());
        assert_eq!(journal.stats.total_guesses, deserialized.stats.total_guesses);
    }
}
```

### 10.2 Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let save_path = temp_dir.path().join("test_journal.bin");
        
        // Create and save journal
        {
            let mut journal = Journal::new();
            journal.discover_fish("clownfish".to_string());
            
            let bytes = bincode::serialize(&journal).unwrap();
            std::fs::write(&save_path, bytes).unwrap();
        }
        
        // Load journal
        {
            let bytes = std::fs::read(&save_path).unwrap();
            let journal: Journal = bincode::deserialize(&bytes).unwrap();
            
            assert!(journal.has_discovered("clownfish"));
        }
    }
    
    #[test]
    fn test_corrupted_file_recovery() {
        let temp_dir = TempDir::new().unwrap();
        let save_path = temp_dir.path().join("corrupted.bin");
        
        // Write garbage data
        std::fs::write(&save_path, b"not valid bincode data").unwrap();
        
        // Should handle gracefully by creating backup
        let result = PersistenceManager::load_from_path(&save_path);
        assert!(result.is_ok());
        
        let journal = result.unwrap();
        assert_eq!(journal.discovered_count(), 0);  // Fresh journal
        
        // Backup should exist
        assert!(save_path.with_extension("bak").exists());
    }
}
```

---

## 11. Performance Considerations

### 11.1 Memory Usage

- `Journal` struct: ~1-10 KB in memory (typical usage)
- `HashMap` overhead: 16 bytes per entry + key/value size
- `DateTime`: 12 bytes per timestamp

**Expected memory footprint:** < 50 KB for 100 discovered fish

### 11.2 Disk I/O

- **Load:** Single read operation on startup (~1ms)
- **Save:** Single write operation every 2+ seconds (~1-2ms)
- **Atomic rename:** Prevents corruption, adds negligible overhead

### 11.3 Serialization Performance

Bincode benchmarks (approximate):

- **Serialize:** 1-10 microseconds for typical journal
- **Deserialize:** 1-10 microseconds for typical journal

**Impact:** Negligible compared to disk I/O (milliseconds)

---

## 12. Future Enhancements

### 12.1 Potential Features (Not in Scope)

- **Cloud sync:** Upload save to server for multi-device play
- **Multiple profiles:** Separate journals for different players
- **Export to JSON:** For manual editing or debugging
- **Statistics dashboard:** Rich analytics on gameplay patterns
- **Achievement system:** Unlock badges for milestones
- **Undo/redo:** Revert accidental discoveries (requires event log)

### 12.2 Migration Path

When adding new fields to `Journal` or `GameStats`:

1. Increment `version` number
2. Add migration logic in `Journal::migrate()`
3. Provide defaults for new fields
4. Test migration with old save files

Example:

```rust
// Version 2: Add new field
#[derive(Serialize, Deserialize)]
pub struct GameStats {
    // ... existing fields ...
    
    #[serde(default)]  // Use Default::default() if missing
    pub achievements_unlocked: Vec<String>,
}
```

---

## 13. Implementation Checklist

- [ ] Define `Journal`, `DiscoveryEntry`, and `GameStats` structs
- [ ] Implement `Journal` API methods (discover, check, stats)
- [ ] Create `PersistenceManager` with save path resolution
- [ ] Implement `load_from_path()` with corruption handling
- [ ] Implement `save()` with atomic writes
- [ ] Implement debounced `save_if_needed()`
- [ ] Define `PersistenceError` enum
- [ ] Add backup creation on corruption
- [ ] Write unit tests for journal operations
- [ ] Write integration tests for save/load
- [ ] Test on Linux, macOS, and Windows
- [ ] Document usage in main README
- [ ] Add example code for game integration

---

## 14. References

### 14.1 Dependencies

- **serde:** https://serde.rs/
- **bincode:** https://github.com/bincode-org/bincode
- **chrono:** https://docs.rs/chrono/
- **dirs:** https://docs.rs/dirs/
- **thiserror:** https://docs.rs/thiserror/

### 14.2 Related Standards

- **XDG Base Directory Specification** (Linux): https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html
- **Apple File System Programming Guide** (macOS): https://developer.apple.com/library/archive/documentation/FileManagement/Conceptual/FileSystemProgrammingGuide/

---

## Appendix A: Complete Example Usage

```rust
use crate::persistence::PersistenceManager;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize persistence
    let mut persistence = PersistenceManager::new()?;
    
    println!("Loaded journal with {} discoveries", 
             persistence.journal().discovered_count());
    
    // Discover a fish
    persistence.journal_mut().discover_fish("clownfish".to_string());
    
    // Check if discovered
    if persistence.journal().has_discovered("clownfish") {
        println!("You've found clownfish before!");
    }
    
    // Calculate progress (assuming 50 total fish)
    let progress = persistence.journal().completion_percentage(50);
    println!("Progress: {:.1}%", progress);
    
    // Save (will be debounced)
    persistence.save_if_needed()?;
    
    // Force save on exit
    persistence.flush()?;
    
    Ok(())
}
```

---

**End of Specification**
