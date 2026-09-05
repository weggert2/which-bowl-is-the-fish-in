/// Game configuration constants
/// The table texture is drawn across the full window width in its lower area.
pub const TABLE_HEIGHT_RATIO: f32 = 0.6;

/// Centers of the three painted table circles, normalized to the table texture.
pub const TABLE_CIRCLE_X_NORMALIZED: [f32; 3] = [0.256, 0.5, 0.741];
pub const TABLE_CIRCLE_Y_NORMALIZED: f32 = 0.288;

/// Per-lid downward corrections for differences in the lid source art. These
/// are applied on top of `closed_lid_y_offset`, not to the bowl positions.
pub const LID_CLOSED_Y_CORRECTIONS: [f32; 3] = [36.0, 32.0, 20.0];

#[derive(Debug, Clone)]
pub struct GameConfig {
    pub window_width: f32,
    pub window_height: f32,
    pub bowl_size: (f32, f32),
    /// The source lid art's contact edge is near its lower edge, while the
    /// bowl rim is near the top of its image. Lift the closed lid to align
    /// those contact edges at the default 150px render height.
    pub closed_lid_y_offset: f32,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            window_width: 1024.0,
            window_height: 768.0,
            bowl_size: (150.0, 150.0),
            closed_lid_y_offset: -70.0,
        }
    }
}
