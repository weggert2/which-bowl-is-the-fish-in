use crate::config::GameConfig;
use egui::{Color32, Pos2, Rect, Rounding, Sense, Stroke, Vec2};
use rand::Rng;

/// Main application state
pub struct WhichBowlApp {
    config: GameConfig,
    correct_bowl: usize,
    message: String,
    hover_bowl: Option<usize>,
}

impl WhichBowlApp {
    /// Create a new instance of the app
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self {
            config: GameConfig::default(),
            correct_bowl: 0,
            message: String::new(),
            hover_bowl: None,
        };
        app.start_new_round();
        app
    }

    /// Start a new round by selecting a random bowl
    fn start_new_round(&mut self) {
        let mut rng = rand::thread_rng();
        self.correct_bowl = rng.gen_range(0..3);
        self.message.clear();
    }

    /// Handle a bowl click
    fn on_bowl_clicked(&mut self, bowl_index: usize) {
        if bowl_index == self.correct_bowl {
            self.message = "✓ Correct! Well done!".to_string();
            // Start new round after correct guess
            self.start_new_round();
        } else {
            self.message = "✗ Wrong bowl! Try again.".to_string();
        }
    }

    /// Render a single bowl
    fn render_bowl(
        &mut self,
        ui: &mut egui::Ui,
        bowl_index: usize,
        position: Pos2,
    ) -> bool {
        let bowl_size = Vec2::new(self.config.bowl_size.0, self.config.bowl_size.1);
        let rect = Rect::from_min_size(position, bowl_size);

        let is_hovered = self.hover_bowl == Some(bowl_index);

        // Allocate space and detect interactions
        let response = ui.allocate_rect(rect, Sense::click());

        // Update hover state
        if response.hovered() {
            self.hover_bowl = Some(bowl_index);
        } else if self.hover_bowl == Some(bowl_index) {
            self.hover_bowl = None;
        }

        // Bowl colors
        let fill_color = if is_hovered {
            Color32::from_rgb(200, 180, 140) // Lighter brown on hover
        } else {
            Color32::from_rgb(160, 140, 100) // Default brown/tan
        };

        let stroke = if is_hovered {
            Stroke::new(3.0, Color32::from_rgb(100, 80, 60))
        } else {
            Stroke::new(2.0, Color32::from_rgb(120, 100, 80))
        };

        // Draw bowl rectangle
        ui.painter().rect(
            rect,
            Rounding::same(12.0),
            fill_color,
            stroke,
        );

        // Draw bowl emoji in center
        let text_pos = rect.center();
        ui.painter().text(
            text_pos,
            egui::Align2::CENTER_CENTER,
            "🥣",
            egui::FontId::proportional(50.0),
            Color32::from_rgb(80, 60, 40),
        );

        response.clicked()
    }
}

impl eframe::App for WhichBowlApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(40.0);

                // Title
                ui.heading(
                    egui::RichText::new("Which bowl is the fish in?")
                        .size(36.0)
                        .color(Color32::from_rgb(30, 60, 90))
                );

                ui.add_space(80.0);

                // Calculate bowl positions
                let available_width = ui.available_width();
                let bowl_width = self.config.bowl_size.0;
                let total_bowl_width = 3.0 * bowl_width + 2.0 * self.config.bowl_spacing;
                let start_x = (available_width - total_bowl_width) / 2.0;

                // Get current cursor position for vertical alignment
                let cursor_pos = ui.cursor().min;

                // Render three bowls horizontally
                let mut clicked_bowl = None;
                for bowl_idx in 0..3 {
                    let x_offset = start_x + bowl_idx as f32 * (bowl_width + self.config.bowl_spacing);
                    let bowl_pos = Pos2::new(
                        cursor_pos.x + x_offset,
                        cursor_pos.y,
                    );

                    if self.render_bowl(ui, bowl_idx, bowl_pos) {
                        clicked_bowl = Some(bowl_idx);
                    }
                }

                // Handle bowl click
                if let Some(bowl_idx) = clicked_bowl {
                    self.on_bowl_clicked(bowl_idx);
                }

                // Advance cursor past bowls
                ui.add_space(self.config.bowl_size.1 + 60.0);

                // Display message
                if !self.message.is_empty() {
                    let message_color = if self.message.contains("Correct") {
                        Color32::from_rgb(50, 180, 50) // Green for correct
                    } else {
                        Color32::from_rgb(220, 50, 50) // Red for wrong
                    };

                    ui.label(
                        egui::RichText::new(&self.message)
                            .size(24.0)
                            .color(message_color)
                    );
                }
            });
        });
    }
}
