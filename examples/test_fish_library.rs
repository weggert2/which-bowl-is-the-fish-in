use which_bowl::fish::{FishLibrary, load_from_toml};

fn main() {
    println!("Testing Fish Library System");
    println!("===========================\n");

    // Create and load fish library
    let mut fish_library = FishLibrary::new();

    match load_from_toml("assets/fish_library.toml", &mut fish_library) {
        Ok(count) => {
            println!("✓ Successfully loaded {} fish", count);
        }
        Err(e) => {
            eprintln!("✗ Error loading fish library: {}", e);
            return;
        }
    }

    // Display library stats
    let stats = fish_library.stats();
    println!("\nLibrary Statistics:");
    println!("  Total fish: {}", stats.total_fish);
    for (rarity, count) in &stats.by_rarity {
        println!("  {}: {} fish", rarity, count);
    }

    // Test random selection
    println!("\nTesting random fish selection (10 picks):");
    for i in 1..=10 {
        match fish_library.select_random_fish() {
            Ok(fish) => {
                println!("  {}. {} - {} ({})", i, fish.name, fish.species, fish.rarity);
            }
            Err(e) => {
                eprintln!("  Error selecting fish: {}", e);
            }
        }
    }

    println!("\n✓ All tests completed successfully!");
}
