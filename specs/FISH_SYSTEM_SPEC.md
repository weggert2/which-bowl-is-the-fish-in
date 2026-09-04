# Technical Specification: Fish Library and Rarity System

**Project:** Which Bowl is the Fish In?  
**Language:** Rust  
**UI Framework:** egui  
**Version:** 1.0  
**Last Updated:** 2026-09-04

---

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Fish Data Structure](#fish-data-structure)
4. [Rarity System](#rarity-system)
5. [Fish Library](#fish-library)
6. [Weighted Random Selection](#weighted-random-selection)
7. [Fish Data Loading](#fish-data-loading)
8. [Example Fish Data](#example-fish-data)
9. [Error Handling](#error-handling)
10. [Testing Strategy](#testing-strategy)

---

## Overview

The fish library system manages a collection of fish with varying rarity levels, providing weighted random selection for gameplay. The system supports both real and fantastical fish, each with associated facts, images, and metadata.

### Design Goals

- **Performance**: Fast O(1) rarity-based lookups
- **Extensibility**: Easy to add new fish via TOML files
- **Reliability**: Strong validation and error handling
- **Maintainability**: Clear separation of concerns across modules

---

## Architecture

### Module Structure

```
src/fish/
├── mod.rs           # Public API and Fish struct
├── rarity.rs        # Rarity enum and weighting system
└── loader.rs        # TOML parsing and validation
```

### Dependencies

Add to `Cargo.toml`:

```toml
[dependencies]
egui = "0.29"
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
rand = "0.8"
thiserror = "1.0"
```

---

## Fish Data Structure

### Core Fish Struct

**File:** `src/fish/mod.rs`

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::fish::rarity::Rarity;

/// Represents a single fish in the game library
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Fish {
    /// Unique identifier (e.g., "clownfish_001")
    pub id: String,
    
    /// Display name (e.g., "Clownfish")
    pub name: String,
    
    /// Scientific or common species name (e.g., "Amphiprion ocellaris")
    pub species: String,
    
    /// Rarity tier affecting drop rate
    pub rarity: Rarity,
    
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
    
    /// Returns the UI display color for this fish's rarity
    pub fn rarity_color(&self) -> egui::Color32 {
        self.rarity.color()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Invalid fish ID: {0}")]
    InvalidId(String),
    
    #[error("Empty required field: {0}")]
    EmptyField(String),
    
    #[error("Fact too long ({0} chars, max 500)")]
    FactTooLong(usize),
    
    #[error("Invalid image path: {0}")]
    InvalidImagePath(PathBuf),
}
```

### Field Specifications

| Field | Type | Required | Validation Rules |
|-------|------|----------|------------------|
| `id` | String | Yes | Non-empty, alphanumeric + underscores, unique |
| `name` | String | Yes | Non-empty, max 100 chars |
| `species` | String | Yes | Non-empty, max 200 chars |
| `rarity` | Rarity | Yes | Valid enum variant |
| `fact` | String | Yes | Non-empty, max 500 chars |
| `image_path` | PathBuf | Yes | Valid path with image extension |
| `is_real` | bool | Yes | - |
| `fact_is_real` | bool | Yes | - |

---

## Rarity System

### Rarity Enum

**File:** `src/fish/rarity.rs`

```rust
use serde::{Deserialize, Serialize};
use rand::Rng;

/// Rarity tiers with associated weights for random selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
    Mythic,
}

impl Rarity {
    /// Returns the weight value for weighted random selection
    /// 
    /// Total weight: 10000 (for precision)
    /// These weights produce the following probabilities:
    /// - Common: 50.0%
    /// - Uncommon: 30.0%
    /// - Rare: 15.0%
    /// - Epic: 4.0%
    /// - Legendary: 0.8%
    /// - Mythic: 0.2%
    pub fn weight(&self) -> u32 {
        match self {
            Rarity::Common     => 5000,  // 50.0%
            Rarity::Uncommon   => 3000,  // 30.0%
            Rarity::Rare       => 1500,  // 15.0%
            Rarity::Epic       =>  400,  //  4.0%
            Rarity::Legendary  =>   80,  //  0.8%
            Rarity::Mythic     =>   20,  //  0.2%
        }
    }
    
    /// Returns the total weight sum (for calculating probabilities)
    pub fn total_weight() -> u32 {
        5000 + 3000 + 1500 + 400 + 80 + 20 // 10000
    }
    
    /// Returns the drop rate as a percentage
    pub fn drop_rate(&self) -> f32 {
        (self.weight() as f32 / Self::total_weight() as f32) * 100.0
    }
    
    /// Returns the display color for UI rendering
    pub fn color(&self) -> egui::Color32 {
        match self {
            Rarity::Common    => egui::Color32::from_rgb(155, 155, 155), // Gray
            Rarity::Uncommon  => egui::Color32::from_rgb(85, 255, 85),   // Green
            Rarity::Rare      => egui::Color32::from_rgb(85, 170, 255),  // Blue
            Rarity::Epic      => egui::Color32::from_rgb(200, 85, 255),  // Purple
            Rarity::Legendary => egui::Color32::from_rgb(255, 170, 0),   // Orange
            Rarity::Mythic    => egui::Color32::from_rgb(255, 85, 255),  // Magenta
        }
    }
    
    /// Returns the display name for this rarity
    pub fn display_name(&self) -> &'static str {
        match self {
            Rarity::Common    => "Common",
            Rarity::Uncommon  => "Uncommon",
            Rarity::Rare      => "Rare",
            Rarity::Epic      => "Epic",
            Rarity::Legendary => "Legendary",
            Rarity::Mythic    => "Mythic",
        }
    }
    
    /// Returns all rarity variants in order
    pub fn all() -> [Rarity; 6] {
        [
            Rarity::Common,
            Rarity::Uncommon,
            Rarity::Rare,
            Rarity::Epic,
            Rarity::Legendary,
            Rarity::Mythic,
        ]
    }
}

impl std::fmt::Display for Rarity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}
```

### Rarity Distribution Table

| Rarity | Weight | Drop Rate | Color | Hex Code |
|--------|--------|-----------|-------|----------|
| Common | 5000 | 50.0% | Gray | #9B9B9B |
| Uncommon | 3000 | 30.0% | Green | #55FF55 |
| Rare | 1500 | 15.0% | Blue | #55AAFF |
| Epic | 400 | 4.0% | Purple | #C855FF |
| Legendary | 80 | 0.8% | Orange | #FFAA00 |
| Mythic | 20 | 0.2% | Magenta | #FF55FF |

**Total Weight:** 10,000 (for precision with integer arithmetic)

---

## Fish Library

### FishLibrary Structure

**File:** `src/fish/mod.rs` (continued)

```rust
use std::collections::HashMap;
use rand::seq::SliceRandom;
use rand::Rng;

/// Main fish library managing all available fish
pub struct FishLibrary {
    /// All fish, indexed by ID
    fish_by_id: HashMap<String, Fish>,
    
    /// Fish grouped by rarity for efficient weighted selection
    fish_by_rarity: HashMap<Rarity, Vec<String>>,
    
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
            fish_by_rarity: HashMap::new(),
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
        let rarity = fish.rarity;
        
        // Add to main storage
        self.fish_by_id.insert(id.clone(), fish);
        
        // Add to rarity index
        self.fish_by_rarity
            .entry(rarity)
            .or_insert_with(Vec::new)
            .push(id);
        
        Ok(())
    }
    
    /// Returns the total number of fish in the library
    pub fn total_count(&self) -> usize {
        self.fish_by_id.len()
    }
    
    /// Returns the count of fish for a specific rarity
    pub fn count_by_rarity(&self, rarity: Rarity) -> usize {
        self.fish_by_rarity
            .get(&rarity)
            .map(|v| v.len())
            .unwrap_or(0)
    }
    
    /// Gets a fish by ID
    pub fn get_fish(&self, id: &str) -> Option<&Fish> {
        self.fish_by_id.get(id)
    }
    
    /// Lists all fish IDs for a given rarity
    pub fn fish_ids_by_rarity(&self, rarity: Rarity) -> Vec<&str> {
        self.fish_by_rarity
            .get(&rarity)
            .map(|ids| ids.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }
    
    /// Returns statistics about the library
    pub fn stats(&self) -> LibraryStats {
        let mut stats = LibraryStats::default();
        stats.total_fish = self.total_count();
        
        for rarity in Rarity::all() {
            let count = self.count_by_rarity(rarity);
            stats.by_rarity.insert(rarity, count);
        }
        
        Ok(stats)
    }
}

/// Statistics about the fish library
#[derive(Debug, Default)]
pub struct LibraryStats {
    pub total_fish: usize,
    pub by_rarity: HashMap<Rarity, usize>,
}

#[derive(Debug, thiserror::Error)]
pub enum LibraryError {
    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),
    
    #[error("Duplicate fish ID: {0}")]
    DuplicateId(String),
    
    #[error("No fish available for selection")]
    EmptyLibrary,
}
```

---

## Weighted Random Selection

### Selection Algorithm

**File:** `src/fish/mod.rs` (continued)

```rust
impl FishLibrary {
    /// Selects a random fish based on rarity weights
    /// 
    /// Algorithm:
    /// 1. Generate random number in range [0, total_weight)
    /// 2. Iterate through rarities, subtracting their weights
    /// 3. When accumulated weight exceeds random number, select that rarity
    /// 4. Randomly pick a fish from that rarity tier
    /// 5. If fish was recently selected, retry up to 3 times
    pub fn select_random_fish(&mut self) -> Result<&Fish, LibraryError> {
        if self.fish_by_id.is_empty() {
            return Err(LibraryError::EmptyLibrary);
        }
        
        let mut rng = rand::thread_rng();
        
        // Try up to 3 times to avoid duplicates
        for _attempt in 0..3 {
            let selected_rarity = self.select_weighted_rarity(&mut rng);
            
            if let Some(fish) = self.select_from_rarity(selected_rarity, &mut rng) {
                if !self.was_recently_selected(&fish.id) {
                    self.record_selection(&fish.id);
                    return Ok(fish);
                }
            }
        }
        
        // If all attempts failed, just pick any fish
        let fish = self.select_any_fish(&mut rng)?;
        self.record_selection(&fish.id);
        Ok(fish)
    }
    
    /// Selects a rarity tier using weighted random selection
    fn select_weighted_rarity<R: Rng>(&self, rng: &mut R) -> Rarity {
        let total_weight = Rarity::total_weight();
        let mut random_weight = rng.gen_range(0..total_weight);
        
        for rarity in Rarity::all() {
            let weight = rarity.weight();
            if random_weight < weight {
                return rarity;
            }
            random_weight -= weight;
        }
        
        // Fallback (should never reach here due to weight calculation)
        Rarity::Common
    }
    
    /// Selects a random fish from a specific rarity tier
    fn select_from_rarity<R: Rng>(&self, rarity: Rarity, rng: &mut R) -> Option<&Fish> {
        let fish_ids = self.fish_by_rarity.get(&rarity)?;
        if fish_ids.is_empty() {
            return None;
        }
        
        let id = fish_ids.choose(rng)?;
        self.fish_by_id.get(id)
    }
    
    /// Selects any random fish (fallback method)
    fn select_any_fish<R: Rng>(&self, rng: &mut R) -> Result<&Fish, LibraryError> {
        let id = self.fish_by_id
            .keys()
            .choose(rng)
            .ok_or(LibraryError::EmptyLibrary)?;
        
        Ok(self.fish_by_id.get(id).unwrap())
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
```

### Selection Flow Diagram

```
┌─────────────────────────────────┐
│ select_random_fish()            │
└────────────┬────────────────────┘
             │
             ▼
┌─────────────────────────────────┐
│ Generate random weight [0, 10K) │
└────────────┬────────────────────┘
             │
             ▼
┌─────────────────────────────────┐
│ select_weighted_rarity()        │
│ - Common:    [0,    5000)       │
│ - Uncommon:  [5000, 8000)       │
│ - Rare:      [8000, 9500)       │
│ - Epic:      [9500, 9900)       │
│ - Legendary: [9900, 9980)       │
│ - Mythic:    [9980, 10000)      │
└────────────┬────────────────────┘
             │
             ▼
┌─────────────────────────────────┐
│ select_from_rarity()            │
│ - Get all fish IDs for rarity   │
│ - Choose random ID from list    │
└────────────┬────────────────────┘
             │
             ▼
┌─────────────────────────────────┐
│ Check recent history            │
│ - If duplicate, retry (max 3x)  │
│ - Record selection              │
└────────────┬────────────────────┘
             │
             ▼
┌─────────────────────────────────┐
│ Return Fish reference           │
└─────────────────────────────────┘
```

---

## Fish Data Loading

### TOML File Format

**File:** `data/fish/fish_library.toml`

```toml
# Fish Library Data File
# Format: TOML
# Each [[fish]] entry defines one fish

[[fish]]
id = "clownfish_001"
name = "Clownfish"
species = "Amphiprion ocellaris"
rarity = "Common"
fact = "Clownfish are all born male. The dominant male will turn female when the breeding female dies."
image_path = "assets/fish/clownfish.png"
is_real = true
fact_is_real = true

[[fish]]
id = "anglerfish_001"
name = "Deep Sea Anglerfish"
species = "Melanocetus johnsonii"
rarity = "Rare"
fact = "Male anglerfish are tiny parasites that permanently fuse to females, becoming nothing more than sperm-producing appendages."
image_path = "assets/fish/anglerfish.png"
is_real = true
fact_is_real = true

[[fish]]
id = "moonfish_mythic"
name = "Lunar Moonfish"
species = "Selenichthys nocturnus"
rarity = "Mythic"
fact = "This fish only appears during full moons and is said to grant wishes to those who catch it."
image_path = "assets/fish/moonfish.png"
is_real = false
fact_is_real = false
```

### Loader Implementation

**File:** `src/fish/loader.rs`

```rust
use std::fs;
use std::path::Path;
use serde::Deserialize;
use crate::fish::{Fish, FishLibrary, LibraryError};

/// Container for deserializing TOML fish data
#[derive(Debug, Deserialize)]
struct FishData {
    fish: Vec<Fish>,
}

/// Loads fish from a TOML file into the library
pub fn load_from_toml<P: AsRef<Path>>(
    path: P,
    library: &mut FishLibrary,
) -> Result<usize, LoadError> {
    let path = path.as_ref();
    
    // Read file contents
    let contents = fs::read_to_string(path)
        .map_err(|e| LoadError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;
    
    // Parse TOML
    let fish_data: FishData = toml::from_str(&contents)
        .map_err(|e| LoadError::Parse {
            path: path.to_path_buf(),
            source: e,
        })?;
    
    // Validate and add each fish
    let mut loaded_count = 0;
    let mut errors = Vec::new();
    
    for fish in fish_data.fish {
        match library.add_fish(fish.clone()) {
            Ok(()) => loaded_count += 1,
            Err(e) => errors.push((fish.id.clone(), e)),
        }
    }
    
    // Report errors if any
    if !errors.is_empty() {
        return Err(LoadError::PartialLoad {
            loaded: loaded_count,
            errors,
        });
    }
    
    Ok(loaded_count)
}

/// Loads fish from multiple TOML files
pub fn load_from_directory<P: AsRef<Path>>(
    dir_path: P,
    library: &mut FishLibrary,
) -> Result<usize, LoadError> {
    let dir_path = dir_path.as_ref();
    
    if !dir_path.is_dir() {
        return Err(LoadError::NotADirectory(dir_path.to_path_buf()));
    }
    
    let mut total_loaded = 0;
    
    for entry in fs::read_dir(dir_path).map_err(|e| LoadError::Io {
        path: dir_path.to_path_buf(),
        source: e,
    })? {
        let entry = entry.map_err(|e| LoadError::Io {
            path: dir_path.to_path_buf(),
            source: e,
        })?;
        
        let path = entry.path();
        
        // Only process .toml files
        if path.extension().and_then(|s| s.to_str()) == Some("toml") {
            match load_from_toml(&path, library) {
                Ok(count) => total_loaded += count,
                Err(e) => eprintln!("Warning: Failed to load {}: {}", path.display(), e),
            }
        }
    }
    
    Ok(total_loaded)
}

#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("IO error reading {path}: {source}")]
    Io {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
    
    #[error("Parse error in {path}: {source}")]
    Parse {
        path: std::path::PathBuf,
        #[source]
        source: toml::de::Error,
    },
    
    #[error("Partial load: {loaded} fish loaded, {} errors", errors.len())]
    PartialLoad {
        loaded: usize,
        errors: Vec<(String, LibraryError)>,
    },
    
    #[error("Not a directory: {0}")]
    NotADirectory(std::path::PathBuf),
}
```

### Validation During Load

The loader validates:
1. **TOML syntax** - Valid TOML format
2. **Required fields** - All fields present
3. **Type correctness** - Correct types for each field
4. **Business rules** - Via `Fish::validate()`
5. **Uniqueness** - No duplicate fish IDs

---

## Example Fish Data

### Sample Fish Library

**File:** `data/fish/example_fish.toml`

```toml
# ============================================================================
# REAL FISH - Common
# ============================================================================

[[fish]]
id = "goldfish_001"
name = "Goldfish"
species = "Carassius auratus"
rarity = "Common"
fact = "Goldfish can recognize their owners and can be trained to perform tricks, disproving the '3-second memory' myth."
image_path = "assets/fish/goldfish.png"
is_real = true
fact_is_real = true

[[fish]]
id = "guppy_001"
name = "Guppy"
species = "Poecilia reticulata"
rarity = "Common"
fact = "Female guppies can store sperm for up to 10 months and produce multiple batches of fry from a single mating."
image_path = "assets/fish/guppy.png"
is_real = true
fact_is_real = true

# ============================================================================
# REAL FISH - Uncommon
# ============================================================================

[[fish]]
id = "betta_001"
name = "Siamese Fighting Fish"
species = "Betta splendens"
rarity = "Uncommon"
fact = "Male bettas build bubble nests at the water's surface to protect their eggs, even when no female is present."
image_path = "assets/fish/betta.png"
is_real = true
fact_is_real = true

[[fish]]
id = "discus_001"
name = "Discus Fish"
species = "Symphysodon aequifasciatus"
rarity = "Uncommon"
fact = "Discus fish produce a mucus on their skin that their fry feed on for the first few weeks of life."
image_path = "assets/fish/discus.png"
is_real = true
fact_is_real = true

# ============================================================================
# REAL FISH - Rare
# ============================================================================

[[fish]]
id = "mantis_shrimp_001"
name = "Mantis Shrimp"
species = "Odontodactylus scyllarus"
rarity = "Rare"
fact = "Mantis shrimp can punch with the force of a .22 caliber bullet, creating cavitation bubbles that reach the temperature of the sun's surface."
image_path = "assets/fish/mantis_shrimp.png"
is_real = true
fact_is_real = true

[[fish]]
id = "leafy_seadragon_001"
name = "Leafy Seadragon"
species = "Phycodurus eques"
rarity = "Rare"
fact = "Leafy seadragons are the only species where the male becomes pregnant and gives birth to hundreds of babies."
image_path = "assets/fish/leafy_seadragon.png"
is_real = true
fact_is_real = true

# ============================================================================
# REAL FISH - Epic
# ============================================================================

[[fish]]
id = "coelacanth_001"
name = "Coelacanth"
species = "Latimeria chalumnae"
rarity = "Epic"
fact = "Thought extinct for 66 million years, the coelacanth was rediscovered in 1938. It can live for 100+ years and gestates babies for 5 years."
image_path = "assets/fish/coelacanth.png"
is_real = true
fact_is_real = true

[[fish]]
id = "goblin_shark_001"
name = "Goblin Shark"
species = "Mitsukurina owstoni"
rarity = "Epic"
fact = "The goblin shark has a protrusible jaw that shoots forward to catch prey, making it look like an alien creature from nightmares."
image_path = "assets/fish/goblin_shark.png"
is_real = true
fact_is_real = true

# ============================================================================
# FANTASTICAL FISH - Legendary
# ============================================================================

[[fish]]
id = "phoenix_fish_001"
name = "Phoenix Koi"
species = "Cyprinus ignis immortalis"
rarity = "Legendary"
fact = "When a Phoenix Koi dies, it bursts into flames and is reborn from the ashes as a smaller version of itself."
image_path = "assets/fish/phoenix_koi.png"
is_real = false
fact_is_real = false

[[fish]]
id = "crystal_fish_001"
name = "Crystal Lightfish"
species = "Crystallus lucidus"
rarity = "Legendary"
fact = "This fish's transparent body contains bioluminescent crystals that refract light into rainbow patterns, used for hypnotizing prey."
image_path = "assets/fish/crystal_fish.png"
is_real = false
fact_is_real = false

# ============================================================================
# FANTASTICAL FISH - Mythic
# ============================================================================

[[fish]]
id = "void_whale_001"
name = "Void Whale"
species = "Vacuus cetus astralis"
rarity = "Mythic"
fact = "Said to swim through the space between dimensions, the Void Whale is larger than galaxies and feeds on dying stars."
image_path = "assets/fish/void_whale.png"
is_real = false
fact_is_real = false

[[fish]]
id = "time_fish_001"
name = "Temporal Tetra"
species = "Chronos tetraodontidae"
rarity = "Mythic"
fact = "This fish exists in all points of time simultaneously. When you observe it, you're seeing its past, present, and future at once."
image_path = "assets/fish/time_fish.png"
is_real = false
fact_is_real = false

# ============================================================================
# REAL FISH WITH FAKE FACTS - For extra chaos
# ============================================================================

[[fish]]
id = "salmon_fake_fact"
name = "Atlantic Salmon"
species = "Salmo salar"
rarity = "Uncommon"
fact = "Salmon can communicate through interpretive dance, performing elaborate routines to warn of predators."
image_path = "assets/fish/salmon.png"
is_real = true
fact_is_real = false
```

---

## Error Handling

### Error Type Hierarchy

```rust
// In src/fish/mod.rs
pub use crate::fish::loader::LoadError;

// All error types implement std::error::Error and Display
// Error chain: LoadError -> LibraryError -> ValidationError
```

### Error Handling Best Practices

1. **Loading Errors**: Log warnings but continue loading other files
2. **Validation Errors**: Collect all errors before failing
3. **Selection Errors**: Gracefully degrade (pick any fish if weighted selection fails)
4. **User-Facing Errors**: Convert technical errors to friendly messages

### Example Error Handling

```rust
// In game initialization code
pub fn initialize_fish_library() -> Result<FishLibrary, Box<dyn std::error::Error>> {
    let mut library = FishLibrary::new();
    
    match load_from_directory("data/fish", &mut library) {
        Ok(count) => {
            println!("Loaded {} fish successfully", count);
            
            if count == 0 {
                return Err("No fish loaded - check data directory".into());
            }
        }
        Err(e) => {
            eprintln!("Error loading fish: {}", e);
            return Err(e.into());
        }
    }
    
    Ok(library)
}
```

---

## Testing Strategy

### Unit Tests

**File:** `src/fish/mod.rs` (continued)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_fish(id: &str, rarity: Rarity) -> Fish {
        Fish {
            id: id.to_string(),
            name: "Test Fish".to_string(),
            species: "Testus fishus".to_string(),
            rarity,
            fact: "This is a test fact.".to_string(),
            image_path: PathBuf::from("test.png"),
            is_real: true,
            fact_is_real: true,
        }
    }
    
    #[test]
    fn test_fish_validation() {
        let fish = create_test_fish("test_001", Rarity::Common);
        assert!(fish.validate().is_ok());
        
        // Invalid ID
        let mut bad_fish = fish.clone();
        bad_fish.id = "invalid id!".to_string();
        assert!(bad_fish.validate().is_err());
        
        // Empty name
        let mut bad_fish = fish.clone();
        bad_fish.name = "".to_string();
        assert!(bad_fish.validate().is_err());
    }
    
    #[test]
    fn test_library_add_fish() {
        let mut library = FishLibrary::new();
        let fish = create_test_fish("test_001", Rarity::Common);
        
        assert!(library.add_fish(fish).is_ok());
        assert_eq!(library.total_count(), 1);
    }
    
    #[test]
    fn test_duplicate_prevention() {
        let mut library = FishLibrary::new();
        let fish1 = create_test_fish("test_001", Rarity::Common);
        let fish2 = create_test_fish("test_001", Rarity::Rare);
        
        assert!(library.add_fish(fish1).is_ok());
        assert!(library.add_fish(fish2).is_err());
    }
    
    #[test]
    fn test_weighted_selection() {
        use std::collections::HashMap;
        
        let mut library = FishLibrary::new();
        
        // Add multiple fish per rarity
        for i in 0..10 {
            library.add_fish(create_test_fish(&format!("common_{}", i), Rarity::Common)).unwrap();
        }
        for i in 0..5 {
            library.add_fish(create_test_fish(&format!("rare_{}", i), Rarity::Rare)).unwrap();
        }
        library.add_fish(create_test_fish("mythic_1", Rarity::Mythic)).unwrap();
        
        // Run 10000 selections and verify distribution
        let mut counts: HashMap<Rarity, usize> = HashMap::new();
        
        for _ in 0..10000 {
            let fish = library.select_random_fish().unwrap();
            *counts.entry(fish.rarity).or_insert(0) += 1;
        }
        
        let common_pct = counts.get(&Rarity::Common).unwrap_or(&0) * 100 / 10000;
        let rare_pct = counts.get(&Rarity::Rare).unwrap_or(&0) * 100 / 10000;
        let mythic_pct = counts.get(&Rarity::Mythic).unwrap_or(&0) * 100 / 10000;
        
        // Allow 5% variance
        assert!((45..=55).contains(&common_pct), "Common should be ~50%, got {}%", common_pct);
        assert!((10..=20).contains(&rare_pct), "Rare should be ~15%, got {}%", rare_pct);
        assert!(mythic_pct <= 2, "Mythic should be ~0.2%, got {}%", mythic_pct);
    }
}
```

### Integration Tests

**File:** `tests/fish_loading_test.rs`

```rust
use which_bowl::fish::{FishLibrary, loader};
use std::path::PathBuf;

#[test]
fn test_load_example_fish() {
    let mut library = FishLibrary::new();
    let path = PathBuf::from("data/fish/example_fish.toml");
    
    let count = loader::load_from_toml(&path, &mut library)
        .expect("Failed to load example fish");
    
    assert!(count > 0, "Should load at least one fish");
    assert_eq!(library.total_count(), count);
}

#[test]
fn test_invalid_toml_handling() {
    let mut library = FishLibrary::new();
    let path = PathBuf::from("tests/data/invalid_fish.toml");
    
    let result = loader::load_from_toml(&path, &mut library);
    assert!(result.is_err());
}
```

### Test Coverage Goals

- Unit tests: 80%+ coverage
- Integration tests for all file loading scenarios
- Property-based tests for weighted selection distribution
- Validation tests for all error conditions

---

## Performance Considerations

### Time Complexity

| Operation | Complexity | Notes |
|-----------|-----------|-------|
| Add fish | O(1) | HashMap insertion |
| Get fish by ID | O(1) | HashMap lookup |
| Select weighted rarity | O(1) | Fixed 6 rarities |
| Select from rarity | O(1) | Random selection from Vec |
| Total selection | O(1) | Amortized |
| Load from file | O(n) | n = number of fish |

### Memory Usage

- Base library overhead: ~100 bytes
- Per fish: ~200-500 bytes (depends on string sizes)
- Rarity index: ~48 bytes per rarity tier
- Recent history: ~40 bytes per entry (5 entries = 200 bytes)

**Estimated total for 100 fish**: ~30-50 KB

### Optimization Notes

1. Fish are stored by value in HashMap (no heap indirection for the fish themselves)
2. Rarity index stores only String IDs, not full Fish structs
3. Weighted selection uses integer arithmetic (faster than floating point)
4. Recent history uses Vec (cache-friendly, small size)

---

## Future Enhancements

### Potential Features

1. **Fish Collections**
   - Track which fish the player has seen
   - Completion percentage
   - Achievement system

2. **Dynamic Rarity Adjustment**
   - Increase rarity weights for unseen fish
   - Pity timer for mythic fish

3. **Seasonal Fish**
   - Time-based availability
   - Event-exclusive fish

4. **Fish Variants**
   - Shiny/alternate versions
   - Color variations

5. **Player Luck System**
   - Persistent luck modifier
   - Streak bonuses

---

## Appendix

### Complete Module Structure

```rust
// src/fish/mod.rs - Public API
mod rarity;
mod loader;

pub use rarity::Rarity;
pub use loader::{load_from_toml, load_from_directory, LoadError};

// Fish struct, FishLibrary, ValidationError, LibraryError
// (implementations shown above)
```

### Dependency Justification

- **serde**: Industry-standard serialization (13M+ downloads/month)
- **toml**: Human-readable config format, excellent for game data
- **rand**: Cryptographically secure RNG, battle-tested
- **thiserror**: Idiomatic error handling, zero overhead
- **egui**: Immediate-mode GUI, perfect for rapid game development

### References

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [egui Documentation](https://docs.rs/egui/)
- [Serde Data Formats](https://serde.rs/)
- [Weighted Random Selection Algorithm](https://en.wikipedia.org/wiki/Fitness_proportionate_selection)

---

**End of Technical Specification**

*This document is a living specification and should be updated as the implementation evolves.*
