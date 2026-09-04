use std::fs;
use std::path::{Path, PathBuf};
use serde::Deserialize;
use crate::fish::{Fish, FishLibrary};

/// Container for deserializing TOML fish data
#[derive(Debug, Deserialize)]
struct FishData {
    fish: Vec<Fish>,
}

/// Error types for fish loading
#[derive(Debug)]
pub enum LoadError {
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    Parse {
        path: PathBuf,
        source: toml::de::Error,
    },
    PartialLoad {
        loaded: usize,
        error_count: usize,
    },
    NotADirectory(PathBuf),
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::Io { path, source } => {
                write!(f, "IO error reading {}: {}", path.display(), source)
            }
            LoadError::Parse { path, source } => {
                write!(f, "Parse error in {}: {}", path.display(), source)
            }
            LoadError::PartialLoad { loaded, error_count } => {
                write!(f, "Partial load: {} fish loaded, {} errors", loaded, error_count)
            }
            LoadError::NotADirectory(path) => {
                write!(f, "Not a directory: {}", path.display())
            }
        }
    }
}

impl std::error::Error for LoadError {}

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
    let mut error_count = 0;

    for fish in fish_data.fish {
        match library.add_fish(fish.clone()) {
            Ok(()) => loaded_count += 1,
            Err(e) => {
                eprintln!("Error adding fish '{}': {}", fish.id, e);
                error_count += 1;
            }
        }
    }

    // Report errors if any
    if error_count > 0 {
        return Err(LoadError::PartialLoad {
            loaded: loaded_count,
            error_count,
        });
    }

    Ok(loaded_count)
}

/// Loads fish from multiple TOML files in a directory
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
