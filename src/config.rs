/// Game configuration constants
#[derive(Debug, Clone)]
pub struct GameConfig {
    pub window_width: f32,
    pub window_height: f32,
    pub bowl_size: (f32, f32),
    pub bowl_spacing: f32,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            window_width: 1024.0,
            window_height: 768.0,
            bowl_size: (150.0, 150.0),
            bowl_spacing: 40.0,
        }
    }
}
