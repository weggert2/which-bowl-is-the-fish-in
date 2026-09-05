mod loader;
mod rarity;

pub use loader::{load_from_directory, load_from_toml};
pub use rarity::Rarity;

use rand::seq::IteratorRandom;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Represents a single fish in the game library
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Fish {
    /// Unique identifier (e.g., "clownfish_001")
    pub id: String,

    /// Display name (e.g., "Clownfish")
    pub name: String,

    /// Scientific or common species name (e.g., "Amphiprion ocellaris")
    pub species: String,

    /// Facts eligible to be displayed when this fish is successfully revealed.
    pub facts: Vec<String>,

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

        // At least one non-empty, reasonably sized fact is required.
        if self.facts.is_empty() {
            return Err(ValidationError::EmptyFacts);
        }
        for (index, fact) in self.facts.iter().enumerate() {
            if fact.trim().is_empty() {
                return Err(ValidationError::EmptyFact(index));
            }
            if fact.len() > 500 {
                return Err(ValidationError::FactTooLong {
                    index,
                    length: fact.len(),
                });
            }
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
    EmptyFacts,
    EmptyFact(usize),
    FactTooLong { index: usize, length: usize },
    InvalidImagePath(PathBuf),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::InvalidId(id) => write!(f, "Invalid fish ID: {}", id),
            ValidationError::EmptyField(field) => write!(f, "Empty required field: {}", field),
            ValidationError::EmptyFacts => write!(f, "Fish must provide at least one fact"),
            ValidationError::EmptyFact(index) => write!(f, "Fact {} is empty", index + 1),
            ValidationError::FactTooLong { index, length } => {
                write!(f, "Fact {} too long ({} chars, max 500)", index + 1, length)
            }
            ValidationError::InvalidImagePath(path) => {
                write!(f, "Invalid image path: {}", path.display())
            }
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

    /// Selects a random fish with equal probability for all fish.
    ///
    /// This considers the complete library, including entries whose assets have
    /// not been added yet. Use [`Self::select_random_playtest_fish`] for an
    /// initial playtest that must display every selected fish.
    ///
    /// Algorithm:
    /// 1. Try up to 3 times to select a fish that wasn't recently selected
    /// 2. If all attempts fail, just pick any fish
    pub fn select_random_fish(&mut self) -> Result<&Fish, LibraryError> {
        if self.fish_by_id.is_empty() {
            return Err(LibraryError::EmptyLibrary);
        }

        let candidate_ids = self.fish_by_id.keys().cloned().collect();
        self.select_from_candidates(candidate_ids)
    }

    /// Selects a random fish whose asset is a readable file beneath
    /// `assets_directory`.
    ///
    /// Fish library TOML paths are relative to the assets directory (for
    /// example, `fish/clownfish_01.png`), so callers should pass `"assets"`
    /// for the shipped game. The full library remains intact; this only filters
    /// the candidates for this selection.
    pub fn select_random_playtest_fish<P: AsRef<Path>>(
        &mut self,
        assets_directory: P,
    ) -> Result<&Fish, LibraryError> {
        let assets_directory = assets_directory.as_ref();
        let candidate_ids = self
            .fish_by_id
            .iter()
            .filter(|(_, fish)| image_is_readable(assets_directory, &fish.image_path))
            .map(|(id, _)| id.clone())
            .collect();

        self.select_from_candidates(candidate_ids)
    }

    /// Returns the number of fish currently eligible for a playable round.
    pub fn playtest_eligible_count<P: AsRef<Path>>(&self, assets_directory: P) -> usize {
        let assets_directory = assets_directory.as_ref();
        self.fish_by_id
            .values()
            .filter(|fish| image_is_readable(assets_directory, &fish.image_path))
            .count()
    }

    fn select_from_candidates(
        &mut self,
        candidate_ids: Vec<String>,
    ) -> Result<&Fish, LibraryError> {
        if candidate_ids.is_empty() {
            return Err(LibraryError::NoEligibleFish);
        }

        let mut rng = rand::thread_rng();

        // Try up to 3 times to avoid duplicates
        for _attempt in 0..3 {
            if let Some(fish_id) = candidate_ids.iter().choose(&mut rng) {
                let fish_id = fish_id.clone();
                if !self.was_recently_selected(&fish_id) {
                    self.record_selection(&fish_id);
                    return Ok(self.fish_by_id.get(&fish_id).unwrap());
                }
            }
        }

        // If all attempts failed, just pick any fish
        let fish_id = candidate_ids
            .iter()
            .choose(&mut rng)
            .ok_or(LibraryError::NoEligibleFish)?
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
    NoEligibleFish,
}

impl std::fmt::Display for LibraryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LibraryError::Validation(e) => write!(f, "Validation error: {}", e),
            LibraryError::DuplicateId(id) => write!(f, "Duplicate fish ID: {}", id),
            LibraryError::EmptyLibrary => write!(f, "No fish available for selection"),
            LibraryError::NoEligibleFish => write!(
                f,
                "No fish with a readable image is available for playtest selection"
            ),
        }
    }
}

fn image_is_readable(assets_directory: &Path, image_path: &Path) -> bool {
    // Library image paths are intentionally relative to `assets`; do not let a
    // malformed catalog entry escape that directory during eligibility checks.
    if image_path.is_absolute()
        || image_path
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return false;
    }

    let resolved_path = assets_directory.join(image_path);
    resolved_path.is_file() && std::fs::File::open(resolved_path).is_ok()
}

impl std::error::Error for LibraryError {}

impl From<ValidationError> for LibraryError {
    fn from(err: ValidationError) -> Self {
        LibraryError::Validation(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static TEMP_DIRECTORY_COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn test_fish(id: &str, image_path: &str) -> Fish {
        Fish {
            id: id.to_string(),
            name: id.to_string(),
            species: "Test species".to_string(),
            facts: vec!["Test fact one".to_string(), "Test fact two".to_string()],
            image_path: PathBuf::from(image_path),
            is_real: true,
            fact_is_real: true,
        }
    }

    fn temporary_assets_directory() -> PathBuf {
        let counter = TEMP_DIRECTORY_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "which-bowl-fish-library-{}-{}",
            std::process::id(),
            counter
        ));
        fs::create_dir_all(path.join("fish")).unwrap();
        path
    }

    #[test]
    fn playtest_selection_only_returns_fish_with_readable_assets() {
        let assets_directory = temporary_assets_directory();
        fs::write(assets_directory.join("fish/available.png"), b"test image").unwrap();

        let mut library = FishLibrary::new();
        library
            .add_fish(test_fish("available", "fish/available.png"))
            .unwrap();
        library
            .add_fish(test_fish("missing", "fish/missing.png"))
            .unwrap();

        assert_eq!(library.total_count(), 2);
        assert_eq!(library.playtest_eligible_count(&assets_directory), 1);
        for _ in 0..10 {
            assert_eq!(
                library
                    .select_random_playtest_fish(&assets_directory)
                    .unwrap()
                    .id,
                "available"
            );
        }

        fs::remove_dir_all(assets_directory).unwrap();
    }

    #[test]
    fn playtest_selection_reports_when_no_readable_assets_exist() {
        let assets_directory = temporary_assets_directory();
        let mut library = FishLibrary::new();
        library
            .add_fish(test_fish("missing", "fish/missing.png"))
            .unwrap();

        assert!(matches!(
            library.select_random_playtest_fish(&assets_directory),
            Err(LibraryError::NoEligibleFish)
        ));

        fs::remove_dir_all(assets_directory).unwrap();
    }

    #[test]
    fn fish_requires_non_empty_facts() {
        let mut fish = test_fish("facts", "fish/facts.png");
        fish.facts.clear();
        assert!(matches!(fish.validate(), Err(ValidationError::EmptyFacts)));

        fish.facts = vec!["   ".to_string()];
        assert!(matches!(
            fish.validate(),
            Err(ValidationError::EmptyFact(0))
        ));
    }
}
