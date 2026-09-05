use macroquad::prelude::Color;
use rand::Rng;
use serde::{Deserialize, Serialize};

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
            Rarity::Common => 5000,   // 50.0%
            Rarity::Uncommon => 3000, // 30.0%
            Rarity::Rare => 1500,     // 15.0%
            Rarity::Epic => 400,      //  4.0%
            Rarity::Legendary => 80,  //  0.8%
            Rarity::Mythic => 20,     //  0.2%
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

    /// Returns the display color for UI rendering (macroquad Color)
    pub fn color(&self) -> Color {
        match self {
            Rarity::Common => Color::from_rgba(128, 128, 128, 255), // Gray
            Rarity::Uncommon => Color::from_rgba(50, 200, 50, 255), // Green
            Rarity::Rare => Color::from_rgba(50, 100, 255, 255),    // Blue
            Rarity::Epic => Color::from_rgba(150, 50, 255, 255),    // Purple
            Rarity::Legendary => Color::from_rgba(255, 150, 50, 255), // Orange
            Rarity::Mythic => Color::from_rgba(255, 215, 0, 255),   // Gold
        }
    }

    /// Returns the display name for this rarity
    pub fn display_name(&self) -> &'static str {
        match self {
            Rarity::Common => "Common",
            Rarity::Uncommon => "Uncommon",
            Rarity::Rare => "Rare",
            Rarity::Epic => "Epic",
            Rarity::Legendary => "Legendary",
            Rarity::Mythic => "Mythic",
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

    /// Rolls a random rarity based on the weighted probabilities
    pub fn roll_random() -> Self {
        let mut rng = rand::thread_rng();
        let total_weight = Self::total_weight();
        let mut random_weight = rng.gen_range(0..total_weight);

        for rarity in Self::all() {
            let weight = rarity.weight();
            if random_weight < weight {
                return rarity;
            }
            random_weight -= weight;
        }

        // Fallback (should never reach here due to weight calculation)
        Rarity::Common
    }
}

impl std::fmt::Display for Rarity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}
