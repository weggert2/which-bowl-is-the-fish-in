use crate::config::GameConfig;
use macroquad::prelude::*;
use ::rand::{thread_rng, Rng};

pub struct WhichBowlApp {
    config: GameConfig,
    correct_bowl: usize,
    message: String,
    hover_bowl: Option<usize>,
    background_texture: Option<Texture2D>,
    table_texture: Option<Texture2D>,
}

impl WhichBowlApp {
    pub async fn new() -> Self {
        let mut app = Self {
            config: GameConfig::default(),
            correct_bowl: 0,
            message: String::new(),
            hover_bowl: None,
            background_texture: load_texture_safe("assets/background.png").await,
            table_texture: load_texture_safe("assets/table.png").await,
        };
        app.start_new_round();
        app
    }

    fn start_new_round(&mut self) {
        let mut rng = thread_rng();
        self.correct_bowl = rng.gen_range(0..3);
        self.message.clear();
    }

    fn on_bowl_clicked(&mut self, bowl_index: usize) {
        if bowl_index == self.correct_bowl {
            self.message = "✓ Correct! Well done!".to_string();
            self.start_new_round();
        } else {
            self.message = "✗ Wrong bowl! Try again.".to_string();
        }
    }

    pub fn update(&mut self) {
        let mouse_pos = mouse_position();

        // Check bowl hover and clicks
        self.hover_bowl = None;
        for bowl_idx in 0..3 {
            let bowl_rect = self.get_bowl_rect(bowl_idx);
            if is_point_in_rect(mouse_pos, bowl_rect) {
                self.hover_bowl = Some(bowl_idx);
                if is_mouse_button_pressed(MouseButton::Left) {
                    self.on_bowl_clicked(bowl_idx);
                }
            }
        }
    }

    pub fn draw(&self) {
        clear_background(BLACK);

        // Draw background
        if let Some(bg) = &self.background_texture {
            draw_texture_ex(
                bg,
                0.0,
                0.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(screen_width(), screen_height())),
                    ..Default::default()
                },
            );
        }

        // Draw table in lower portion
        if let Some(table) = &self.table_texture {
            let table_height = screen_height() * 0.6;
            draw_texture_ex(
                table,
                0.0,
                screen_height() - table_height,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(screen_width(), table_height)),
                    ..Default::default()
                },
            );
        }

        // Draw title
        let title = "Which bowl is the fish in?";
        let title_size = 48.0;
        let title_width = measure_text(title, None, title_size as u16, 1.0).width;
        draw_text(
            title,
            screen_width() / 2.0 - title_width / 2.0,
            80.0,
            title_size,
            Color::from_rgba(30, 60, 90, 255),
        );

        // Draw bowls
        for bowl_idx in 0..3 {
            self.draw_bowl(bowl_idx);
        }

        // Draw message
        if !self.message.is_empty() {
            let msg_color = if self.message.contains("Correct") {
                Color::from_rgba(50, 180, 50, 255)
            } else {
                Color::from_rgba(220, 50, 50, 255)
            };
            let msg_size = 32.0;
            let msg_width = measure_text(&self.message, None, msg_size as u16, 1.0).width;
            draw_text(
                &self.message,
                screen_width() / 2.0 - msg_width / 2.0,
                screen_height() - 150.0,
                msg_size,
                msg_color,
            );
        }
    }

    fn get_bowl_rect(&self, bowl_idx: usize) -> Rect {
        let bowl_width = self.config.bowl_size.0;
        let bowl_height = self.config.bowl_size.1;
        let total_width = 3.0 * bowl_width + 2.0 * self.config.bowl_spacing;
        let start_x = (screen_width() - total_width) / 2.0;
        let y = screen_height() / 2.0 - bowl_height / 2.0;

        let x = start_x + bowl_idx as f32 * (bowl_width + self.config.bowl_spacing);

        Rect::new(x, y, bowl_width, bowl_height)
    }

    fn draw_bowl(&self, bowl_idx: usize) {
        let rect = self.get_bowl_rect(bowl_idx);
        let is_hovered = self.hover_bowl == Some(bowl_idx);

        let fill_color = if is_hovered {
            Color::from_rgba(200, 180, 140, 255)
        } else {
            Color::from_rgba(160, 140, 100, 255)
        };

        let border_color = if is_hovered {
            Color::from_rgba(100, 80, 60, 255)
        } else {
            Color::from_rgba(120, 100, 80, 255)
        };

        // Draw bowl rectangle
        draw_rectangle(rect.x, rect.y, rect.w, rect.h, fill_color);
        draw_rectangle_lines(
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            if is_hovered { 3.0 } else { 2.0 },
            border_color,
        );

        // Draw bowl emoji
        let emoji_size = 50.0;
        let emoji_width = measure_text("🥣", None, emoji_size as u16, 1.0).width;
        draw_text(
            "🥣",
            rect.x + rect.w / 2.0 - emoji_width / 2.0,
            rect.y + rect.h / 2.0 + emoji_size / 3.0,
            emoji_size,
            Color::from_rgba(80, 60, 40, 255),
        );
    }
}

async fn load_texture_safe(path: &str) -> Option<Texture2D> {
    match load_texture(path).await {
        Ok(texture) => Some(texture),
        Err(e) => {
            eprintln!("Failed to load texture {}: {}", path, e);
            None
        }
    }
}

fn is_point_in_rect(point: (f32, f32), rect: Rect) -> bool {
    point.0 >= rect.x
        && point.0 <= rect.x + rect.w
        && point.1 >= rect.y
        && point.1 <= rect.y + rect.h
}
