use which_bowl::fish::{load_from_toml, FishLibrary};

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
    println!(
        "  Playtest-eligible fish: {}",
        fish_library.playtest_eligible_count("assets")
    );

    // Test random selection
    println!("\nTesting random fish selection (10 picks):");
    for i in 1..=10 {
        match fish_library.select_random_playtest_fish("assets") {
            Ok(fish) => {
                println!("  {}. {} - {}", i, fish.name, fish.species);
            }
            Err(e) => {
                eprintln!("  Error selecting fish: {}", e);
            }
        }
    }

    println!("\n✓ All tests completed successfully!");
}
