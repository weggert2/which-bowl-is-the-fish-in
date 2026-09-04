use crate::config::GameConfig;
use crate::fish::{FishLibrary, Fish, Rarity, load_from_toml};
use macroquad::prelude::*;
use ::rand::{thread_rng, Rng};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GameState {
    Playing,
    Revealing,
}

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
    fish_texture_cache: HashMap<String, Texture2D>,
    current_fish_texture: Option<Texture2D>,
    game_state: GameState,
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
            fish_texture_cache: HashMap::new(),
            current_fish_texture: None,
            game_state: GameState::Playing,
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

    async fn on_bowl_clicked(&mut self, bowl_index: usize) {
        if bowl_index == self.correct_bowl {
            // Correct guess - load fish texture and transition to Revealing state
            if let Some(fish) = &self.current_fish {
                // Load fish texture (check cache first)
                let image_path = fish.image_path.to_string_lossy().to_string();
                let texture = if let Some(cached) = self.fish_texture_cache.get(&image_path) {
                    Some(cached.clone())
                } else {
                    // Try to load the texture
                    match load_texture(&image_path).await {
                        Ok(tex) => {
                            self.fish_texture_cache.insert(image_path.clone(), tex.clone());
                            Some(tex)
                        }
                        Err(e) => {
                            eprintln!("Failed to load fish texture {}: {}", image_path, e);
                            None
                        }
                    }
                };

                self.current_fish_texture = texture;
            }

            self.game_state = GameState::Revealing;
        } else {
            self.message = "Wrong bowl! Try again.".to_string();
        }
    }

    pub async fn update(&mut self) {
        let mouse_pos = mouse_position();

        match self.game_state {
            GameState::Playing => {
                // Check bowl hover and clicks
                self.hover_bowl = None;
                for bowl_idx in 0..3 {
                    let bowl_rect = self.get_bowl_rect(bowl_idx);
                    if is_point_in_rect(mouse_pos, bowl_rect) {
                        self.hover_bowl = Some(bowl_idx);
                        if is_mouse_button_pressed(MouseButton::Left) {
                            self.on_bowl_clicked(bowl_idx).await;
                        }
                    }
                }
            }
            GameState::Revealing => {
                // Check for click to continue
                if is_mouse_button_pressed(MouseButton::Left) {
                    self.current_fish_texture = None;
                    self.start_new_round();
                    self.game_state = GameState::Playing;
                }
            }
        }
    }

    pub fn draw(&self) {
        clear_background(BLACK);

        match self.game_state {
            GameState::Playing => {
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
                    let msg_color = Color::from_rgba(220, 50, 50, 255);
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
            GameState::Revealing => {
                self.draw_fish_reveal();
            }
        }
    }

    fn draw_fish_reveal(&self) {
        // Semi-transparent dark overlay
        draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::from_rgba(20, 20, 40, 230));

        if let (Some(fish), Some(rarity)) = (&self.current_fish, &self.current_rarity) {
            let center_x = screen_width() / 2.0;
            let center_y = screen_height() / 2.0;

            // Draw title: "You found a {Rarity} {Fish Name}!"
            let title = format!("You found a {} {}!", rarity, fish.name);
            let title_size = 44.0;
            let title_width = measure_text(&title, None, title_size as u16, 1.0).width;
            draw_text(
                &title,
                center_x - title_width / 2.0,
                100.0,
                title_size,
                WHITE,
            );

            // Draw fish image with rarity-colored outline
            let fish_size = 400.0;
            let fish_x = center_x - fish_size / 2.0;
            let fish_y = center_y - fish_size / 2.0 - 20.0;

            // Draw rarity-colored border
            let border_thickness = 8.0;
            let rarity_color = rarity.color();
            draw_rectangle_lines(
                fish_x - border_thickness,
                fish_y - border_thickness,
                fish_size + border_thickness * 2.0,
                fish_size + border_thickness * 2.0,
                border_thickness,
                rarity_color,
            );

            // Draw fish texture or placeholder
            if let Some(texture) = &self.current_fish_texture {
                draw_texture_ex(
                    texture,
                    fish_x,
                    fish_y,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(Vec2::new(fish_size, fish_size)),
                        ..Default::default()
                    },
                );
            } else {
                // Placeholder if texture failed to load
                draw_rectangle(fish_x, fish_y, fish_size, fish_size, Color::from_rgba(60, 60, 80, 255));
                let placeholder_text = "Fish Image";
                let placeholder_size = 24.0;
                let placeholder_width = measure_text(placeholder_text, None, placeholder_size as u16, 1.0).width;
                draw_text(
                    placeholder_text,
                    center_x - placeholder_width / 2.0,
                    center_y,
                    placeholder_size,
                    Color::from_rgba(150, 150, 150, 255),
                );
            }

            // Draw fish fact
            let fact = &fish.fact;
            let fact_size = 24.0;
            let max_fact_width = screen_width() - 200.0;

            // Word wrap the fact
            let wrapped_lines = wrap_text(fact, max_fact_width, fact_size);
            let line_height = fact_size + 8.0;
            let total_fact_height = wrapped_lines.len() as f32 * line_height;
            let fact_start_y = fish_y + fish_size + 60.0;

            for (i, line) in wrapped_lines.iter().enumerate() {
                let line_width = measure_text(line, None, fact_size as u16, 1.0).width;
                draw_text(
                    line,
                    center_x - line_width / 2.0,
                    fact_start_y + i as f32 * line_height,
                    fact_size,
                    Color::from_rgba(220, 220, 220, 255),
                );
            }

            // Draw "Click to continue" button
            let button_text = "Click to continue";
            let button_size = 28.0;
            let button_width = measure_text(button_text, None, button_size as u16, 1.0).width;
            let button_y = fact_start_y + total_fact_height + 40.0;

            draw_text(
                button_text,
                center_x - button_width / 2.0,
                button_y,
                button_size,
                Color::from_rgba(100, 200, 255, 255),
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

fn wrap_text(text: &str, max_width: f32, font_size: f32) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        let test_line = if current_line.is_empty() {
            word.to_string()
        } else {
            format!("{} {}", current_line, word)
        };

        let test_width = measure_text(&test_line, None, font_size as u16, 1.0).width;

        if test_width <= max_width {
            current_line = test_line;
        } else {
            if !current_line.is_empty() {
                lines.push(current_line);
            }
            current_line = word.to_string();
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
}
