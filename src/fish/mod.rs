mod rarity;
mod loader;

pub use rarity::Rarity;
pub use loader::load_from_toml;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use rand::seq::IteratorRandom;

/// Represents a single fish in the game library
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Fish {
    /// Unique identifier (e.g., "clownfish_001")
    pub id: String,

    /// Display name (e.g., "Clownfish")
    pub name: String,

    /// Scientific or common species name (e.g., "Amphiprion ocellaris")
    pub species: String,

    /// Fun fact displayed to player
    pub fact: String,

    /// Path to fish image (relative to assets directory)
    pub image_path: PathBuf,

    /// Whether this fish exists in reality
    pub is_real: bool,

    /// Whether the fact is truthful or fabricated
    pub fact_is_real: bool,
}

impl Fish {
    /// Validates fish data integrity
    pub fn validate(&self) -> Result<(), ValidationError> {
        // ID must be non-empty and alphanumeric with underscores
        if self.id.is_empty() || !self.id.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(ValidationError::InvalidId(self.id.clone()));
        }

        // Name must be non-empty
        if self.name.trim().is_empty() {
            return Err(ValidationError::EmptyField("name".to_string()));
        }

        // Species must be non-empty
        if self.species.trim().is_empty() {
            return Err(ValidationError::EmptyField("species".to_string()));
        }

        // Fact must be non-empty and reasonable length
        if self.fact.trim().is_empty() {
            return Err(ValidationError::EmptyField("fact".to_string()));
        }
        if self.fact.len() > 500 {
            return Err(ValidationError::FactTooLong(self.fact.len()));
        }

        // Image path must have an extension
        if self.image_path.extension().is_none() {
            return Err(ValidationError::InvalidImagePath(self.image_path.clone()));
        }

        Ok(())
    }
}

/// Validation errors
#[derive(Debug)]
pub enum ValidationError {
    InvalidId(String),
    EmptyField(String),
    FactTooLong(usize),
    InvalidImagePath(PathBuf),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::InvalidId(id) => write!(f, "Invalid fish ID: {}", id),
            ValidationError::EmptyField(field) => write!(f, "Empty required field: {}", field),
            ValidationError::FactTooLong(len) => write!(f, "Fact too long ({} chars, max 500)", len),
            ValidationError::InvalidImagePath(path) => write!(f, "Invalid image path: {}", path.display()),
        }
    }
}

impl std::error::Error for ValidationError {}

/// Main fish library managing all available fish
pub struct FishLibrary {
    /// All fish, indexed by ID
    fish_by_id: HashMap<String, Fish>,

    /// Recently selected fish IDs (for duplicate prevention)
    recent_selections: Vec<String>,

    /// Maximum size of recent selections history
    recent_history_size: usize,
}

impl FishLibrary {
    /// Creates a new empty fish library
    pub fn new() -> Self {
        Self {
            fish_by_id: HashMap::new(),
            recent_selections: Vec::new(),
            recent_history_size: 5, // Prevent duplicates in last 5 selections
        }
    }

    /// Adds a fish to the library
    pub fn add_fish(&mut self, fish: Fish) -> Result<(), LibraryError> {
        // Validate fish data
        fish.validate().map_err(LibraryError::Validation)?;

        // Check for duplicate ID
        if self.fish_by_id.contains_key(&fish.id) {
            return Err(LibraryError::DuplicateId(fish.id.clone()));
        }

        let id = fish.id.clone();

        // Add to main storage
        self.fish_by_id.insert(id, fish);

        Ok(())
    }

    /// Returns the total number of fish in the library
    pub fn total_count(&self) -> usize {
        self.fish_by_id.len()
    }

    /// Gets a fish by ID
    pub fn get_fish(&self, id: &str) -> Option<&Fish> {
        self.fish_by_id.get(id)
    }

    /// Returns statistics about the library
    pub fn stats(&self) -> LibraryStats {
        let mut stats = LibraryStats::default();
        stats.total_fish = self.total_count();
        stats
    }

    /// Selects a random fish with equal probability for all fish
    ///
    /// Algorithm:
    /// 1. Try up to 3 times to select a fish that wasn't recently selected
    /// 2. If all attempts fail, just pick any fish
    pub fn select_random_fish(&mut self) -> Result<&Fish, LibraryError> {
        if self.fish_by_id.is_empty() {
            return Err(LibraryError::EmptyLibrary);
        }

        let mut rng = rand::thread_rng();

        // Try up to 3 times to avoid duplicates
        for _attempt in 0..3 {
            if let Some(fish_id) = self.fish_by_id.keys().choose(&mut rng) {
                let fish_id = fish_id.clone();
                if !self.was_recently_selected(&fish_id) {
                    self.record_selection(&fish_id);
                    return Ok(self.fish_by_id.get(&fish_id).unwrap());
                }
            }
        }

        // If all attempts failed, just pick any fish
        let fish_id = self.fish_by_id
            .keys()
            .choose(&mut rng)
            .ok_or(LibraryError::EmptyLibrary)?
            .clone();
        self.record_selection(&fish_id);
        Ok(self.fish_by_id.get(&fish_id).unwrap())
    }

    /// Checks if a fish was recently selected
    fn was_recently_selected(&self, id: &str) -> bool {
        self.recent_selections.contains(&id.to_string())
    }

    /// Records a fish selection in history
    fn record_selection(&mut self, id: &str) {
        self.recent_selections.push(id.to_string());

        // Trim history to max size
        if self.recent_selections.len() > self.recent_history_size {
            self.recent_selections.remove(0);
        }
    }

    /// Clears the recent selection history
    pub fn clear_history(&mut self) {
        self.recent_selections.clear();
    }
}

impl Default for FishLibrary {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about the fish library
#[derive(Debug, Default)]
pub struct LibraryStats {
    pub total_fish: usize,
}

/// Library management errors
#[derive(Debug)]
pub enum LibraryError {
    Validation(ValidationError),
    DuplicateId(String),
    EmptyLibrary,
}

impl std::fmt::Display for LibraryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LibraryError::Validation(e) => write!(f, "Validation error: {}", e),
            LibraryError::DuplicateId(id) => write!(f, "Duplicate fish ID: {}", id),
            LibraryError::EmptyLibrary => write!(f, "No fish available for selection"),
        }
    }
}

impl std::error::Error for LibraryError {}

impl From<ValidationError> for LibraryError {
    fn from(err: ValidationError) -> Self {
        LibraryError::Validation(err)
    }
}
