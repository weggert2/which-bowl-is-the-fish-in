use crate::config::GameConfig;
use crate::fish::{FishLibrary, Fish, Rarity, load_from_toml};
use macroquad::prelude::*;
use ::rand::{thread_rng, Rng};

pub struct WhichBowlApp {
    config: GameConfig,
    correct_bowl: usize,
    message: String,
    hover_bowl: Option<usize>,
    background_texture: Option<Texture2D>,
    table_texture: Option<Texture2D>,
    bowl_textures: [Option<Texture2D>; 3],
    lid_textures: [Option<Texture2D>; 3],
    fish_library: FishLibrary,
    current_fish: Option<Fish>,
    current_rarity: Option<Rarity>,
}

impl WhichBowlApp {
    pub async fn new() -> Self {
        // Load fish library
        let mut fish_library = FishLibrary::new();
        match load_from_toml("assets/fish_library.toml", &mut fish_library) {
            Ok(count) => {
                println!("Loaded {} fish successfully", count);
            }
            Err(e) => {
                eprintln!("Error loading fish library: {}", e);
                eprintln!("Continuing with empty library...");
            }
        }

        let mut app = Self {
            config: GameConfig::default(),
            correct_bowl: 0,
            message: String::new(),
            hover_bowl: None,
            background_texture: load_texture_safe("assets/background.png").await,
            table_texture: load_texture_safe("assets/table.png").await,
            bowl_textures: [
                load_texture_safe("assets/bowls/bowl1.png").await,
                load_texture_safe("assets/bowls/bowl2.png").await,
                load_texture_safe("assets/bowls/bowl3.png").await,
            ],
            lid_textures: [
                load_texture_safe("assets/bowls/lid1.png").await,
                load_texture_safe("assets/bowls/lid2.png").await,
                load_texture_safe("assets/bowls/lid3.png").await,
            ],
            fish_library,
            current_fish: None,
            current_rarity: None,
        };
        app.start_new_round();
        app
    }

    fn start_new_round(&mut self) {
        let mut rng = thread_rng();
        self.correct_bowl = rng.gen_range(0..3);
        self.message.clear();

        // Roll rarity first
        let rarity = Rarity::roll_random();
        self.current_rarity = Some(rarity);

        // Then select a random fish
        match self.fish_library.select_random_fish() {
            Ok(fish) => {
                self.current_fish = Some(fish.clone());
                println!("Selected: {} {} (rarity rolled separately)", rarity, fish.name);
            }
            Err(e) => {
                eprintln!("Error selecting fish: {}", e);
                self.current_fish = None;
                self.current_rarity = None;
            }
        }
    }

    fn on_bowl_clicked(&mut self, bowl_index: usize) {
        if bowl_index == self.correct_bowl {
            // Show what was found with rarity
            if let (Some(fish), Some(rarity)) = (&self.current_fish, &self.current_rarity) {
                self.message = format!("You found a {} {}!", rarity, fish.name);
            } else {
                self.message = "✓ Correct! Well done!".to_string();
            }
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

        // Draw bowl base
        if let Some(bowl_texture) = &self.bowl_textures[bowl_idx] {
            let tint = if is_hovered {
                Color::from_rgba(255, 255, 200, 255) // Slight yellow tint on hover
            } else {
                WHITE
            };

            draw_texture_ex(
                bowl_texture,
                rect.x,
                rect.y,
                tint,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(rect.w, rect.h)),
                    ..Default::default()
                },
            );
        } else {
            // Fallback to placeholder if texture failed to load
            draw_rectangle(rect.x, rect.y, rect.w, rect.h, Color::from_rgba(160, 140, 100, 255));
        }

        // Draw lid on top
        if let Some(lid_texture) = &self.lid_textures[bowl_idx] {
            let tint = if is_hovered {
                Color::from_rgba(255, 255, 200, 255)
            } else {
                WHITE
            };

            draw_texture_ex(
                lid_texture,
                rect.x,
                rect.y,
                tint,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(rect.w, rect.h)),
                    ..Default::default()
                },
            );
        }

        // Draw hover border if hovered
        if is_hovered {
            draw_rectangle_lines(
                rect.x,
                rect.y,
                rect.w,
                rect.h,
                3.0,
                Color::from_rgba(255, 255, 100, 255),
            );
        }
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
