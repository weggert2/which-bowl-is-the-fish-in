use ::rand::{seq::SliceRandom, thread_rng, Rng};
use macroquad::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use which_bowl::config::{
    GameConfig, LID_CLOSED_Y_CORRECTIONS, TABLE_CIRCLE_X_NORMALIZED, TABLE_CIRCLE_Y_NORMALIZED,
    TABLE_HEIGHT_RATIO,
};
use which_bowl::fish::{load_from_toml, Fish, FishLibrary, Rarity};

const WRONG_BOWL_MESSAGES: [&str; 5] = ["Nope", ":(", "Fish Not Found", "Try again", "So sad"];
const FAILED_REVEAL_DURATION: f32 = 0.75;
const CALM_BACKGROUND: Color = Color::from_rgba(224, 239, 244, 255);
/// The cursor PNG is a two-column sheet of 887px square fish frames.
const CURSOR_DISPLAY_SIZE: f32 = 96.0;
const CURSOR_FRAME_SIZE: f32 = 887.0;
const CURSOR_HOTSPOT_X: f32 = 80.0;
const CURSOR_HOTSPOT_Y: f32 = 48.0;
const CURSOR_BUBBLE_DISTANCE: f32 = 14.0;
const MAX_CURSOR_BUBBLES: usize = 24;
const CLOUD_CLUSTERS: [(f32, f32, f32, f32); 4] = [
    (120.0, 120.0, 1.0, 0.20),
    (460.0, 185.0, 0.72, 0.16),
    (790.0, 100.0, 0.9, 0.18),
    (255.0, 285.0, 0.58, 0.14),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GameState {
    Playing,
    FailedReveal,
    Revealing,
}

struct CursorBubble {
    position: Vec2,
    velocity: Vec2,
    radius: f32,
    age: f32,
    lifetime: f32,
}

pub struct WhichBowlApp {
    config: GameConfig,
    asset_root: PathBuf,
    correct_bowl: usize,
    message: String,
    hover_bowl: Option<usize>,
    table_texture: Option<Texture2D>,
    bowl_textures: [Option<Texture2D>; 3],
    lid_textures: [Option<Texture2D>; 3],
    cursor_texture: Option<Texture2D>,
    cursor_bubbles: Vec<CursorBubble>,
    last_cursor_position: Option<Vec2>,
    cursor_bubble_distance: f32,
    cursor_bubble_phase: u32,
    cursor_mouth_open: f32,
    fish_library: FishLibrary,
    current_fish: Option<Fish>,
    current_fact: Option<String>,
    current_rarity: Option<Rarity>,
    fish_texture_cache: HashMap<String, Texture2D>,
    current_fish_texture: Option<Texture2D>,
    game_state: GameState,
    reveal_started_at: f64,
    failed_bowl: Option<usize>,
    failed_reveal_started_at: f64,
}

impl WhichBowlApp {
    pub async fn new() -> Self {
        let asset_root = resolve_asset_root();
        let cursor_texture = load_cursor_texture_safe(&asset_root.join("cursor_fish.png")).await;
        show_mouse(cursor_texture.is_none());

        // Load fish library
        let mut fish_library = FishLibrary::new();
        match load_from_toml(asset_root.join("fish_library.toml"), &mut fish_library) {
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
            asset_root: asset_root.clone(),
            correct_bowl: 0,
            message: String::new(),
            hover_bowl: None,
            table_texture: load_texture_safe(&asset_root.join("table.png")).await,
            bowl_textures: [
                load_texture_safe(&asset_root.join("bowls/bowl2.png")).await,
                load_texture_safe(&asset_root.join("bowls/bowl1.png")).await,
                load_texture_safe(&asset_root.join("bowls/bowl3.png")).await,
            ],
            lid_textures: [
                load_lid_texture_safe(&asset_root.join("bowls/lid2_display.png")).await,
                load_lid_texture_safe(&asset_root.join("bowls/lid1_display.png")).await,
                load_lid_texture_safe(&asset_root.join("bowls/lid3_display.png")).await,
            ],
            cursor_texture,
            cursor_bubbles: Vec::with_capacity(MAX_CURSOR_BUBBLES),
            last_cursor_position: None,
            cursor_bubble_distance: 0.0,
            cursor_bubble_phase: 0,
            cursor_mouth_open: 0.0,
            fish_library,
            current_fish: None,
            current_fact: None,
            current_rarity: None,
            fish_texture_cache: HashMap::new(),
            current_fish_texture: None,
            game_state: GameState::Playing,
            reveal_started_at: 0.0,
            failed_bowl: None,
            failed_reveal_started_at: 0.0,
        };
        app.start_new_round();
        app
    }

    fn start_new_round(&mut self) {
        let mut rng = thread_rng();
        self.correct_bowl = rng.gen_range(0..3);
        self.message.clear();
        self.current_fact = None;

        // Roll rarity first
        let rarity = Rarity::roll_random();
        self.current_rarity = Some(rarity);

        // TOML image paths are relative to `assets`. Only select fish
        // whose image asset is readable.
        match self
            .fish_library
            .select_random_playtest_fish(&self.asset_root)
        {
            Ok(fish) => {
                self.current_fish = Some(fish.clone());
                println!(
                    "Selected: {} {} (rarity rolled separately)",
                    rarity, fish.name
                );
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
                self.current_fact = fish.facts.choose(&mut thread_rng()).cloned();
                if self.current_fact.is_none() {
                    eprintln!("Selected fish '{}' has no valid facts", fish.id);
                }

                // Load fish texture (check cache first)
                let image_path = self
                    .asset_root
                    .join(&fish.image_path)
                    .to_string_lossy()
                    .to_string();
                let texture = if let Some(cached) = self.fish_texture_cache.get(&image_path) {
                    Some(cached.clone())
                } else {
                    // Try to load the texture
                    match load_texture(&image_path).await {
                        Ok(tex) => {
                            self.fish_texture_cache
                                .insert(image_path.clone(), tex.clone());
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

            self.reveal_started_at = get_time();
            self.game_state = GameState::Revealing;
        } else {
            let mut rng = thread_rng();
            self.message =
                WRONG_BOWL_MESSAGES[rng.gen_range(0..WRONG_BOWL_MESSAGES.len())].to_string();
            self.failed_bowl = Some(bowl_index);
            self.failed_reveal_started_at = get_time();
            self.game_state = GameState::FailedReveal;
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
                // Keep the reveal visible long enough for the opening and fish
                // movement to read before allowing the next round.
                if self.reveal_elapsed() >= 1.0 && is_mouse_button_pressed(MouseButton::Left) {
                    self.current_fish_texture = None;
                    self.start_new_round();
                    self.game_state = GameState::Playing;
                }
            }
            GameState::FailedReveal => {
                // Failed reveals are intentionally non-interactive so repeated
                // clicks cannot interrupt the lid's open-and-close feedback.
                if self.failed_reveal_elapsed() >= FAILED_REVEAL_DURATION {
                    self.failed_bowl = None;
                    self.hover_bowl = None;
                    self.message.clear();
                    self.game_state = GameState::Playing;
                }
            }
        }

        self.update_cursor_bubbles(vec2(mouse_pos.0, mouse_pos.1));
    }

    fn draw_cursor(&self) {
        for bubble in &self.cursor_bubbles {
            let fade = (1.0 - bubble.age / bubble.lifetime).clamp(0.0, 1.0);
            let fill = Color::new(0.67, 0.91, 1.0, 0.22 * fade);
            let outline = Color::new(0.92, 0.99, 1.0, 0.76 * fade);
            draw_circle(bubble.position.x, bubble.position.y, bubble.radius, fill);
            draw_circle_lines(
                bubble.position.x,
                bubble.position.y,
                bubble.radius,
                1.2,
                outline,
            );
            draw_circle(
                bubble.position.x - bubble.radius * 0.28,
                bubble.position.y - bubble.radius * 0.28,
                bubble.radius * 0.2,
                Color::new(1.0, 1.0, 1.0, 0.85 * fade),
            );
        }

        if let Some(texture) = &self.cursor_texture {
            let (mouse_x, mouse_y) = mouse_position();
            let source_x = if self.cursor_mouth_open > 0.01 {
                CURSOR_FRAME_SIZE
            } else {
                0.0
            };
            draw_texture_ex(
                texture,
                mouse_x - CURSOR_HOTSPOT_X,
                mouse_y - CURSOR_HOTSPOT_Y,
                WHITE,
                DrawTextureParams {
                    source: Some(Rect::new(
                        source_x,
                        0.0,
                        CURSOR_FRAME_SIZE,
                        CURSOR_FRAME_SIZE,
                    )),
                    dest_size: Some(Vec2::splat(CURSOR_DISPLAY_SIZE)),
                    ..Default::default()
                },
            );
        }
    }

    fn update_cursor_bubbles(&mut self, cursor_position: Vec2) {
        if self.cursor_texture.is_none() {
            return;
        }

        let delta_time = get_frame_time().min(0.1);
        self.cursor_mouth_open = (self.cursor_mouth_open - delta_time * 7.0).max(0.0);
        for bubble in &mut self.cursor_bubbles {
            bubble.age += delta_time;
            bubble.position += bubble.velocity * delta_time;
        }
        self.cursor_bubbles
            .retain(|bubble| bubble.age < bubble.lifetime);

        let Some(previous_position) = self.last_cursor_position.replace(cursor_position) else {
            return;
        };
        let movement = cursor_position - previous_position;
        let distance = movement.length();
        if distance < 1.0 {
            return;
        }

        let speed = distance / delta_time.max(0.001);
        self.cursor_mouth_open = 0.45 + 0.55 * (get_time() as f32 * 18.0).sin().abs();
        self.cursor_bubble_distance += distance;
        while self.cursor_bubble_distance >= CURSOR_BUBBLE_DISTANCE {
            self.cursor_bubble_distance -= CURSOR_BUBBLE_DISTANCE;
            self.cursor_bubble_phase = self.cursor_bubble_phase.wrapping_add(1);

            if self.cursor_bubbles.len() == MAX_CURSOR_BUBBLES {
                self.cursor_bubbles.remove(0);
            }

            let phase = self.cursor_bubble_phase as f32 * 2.399_963_1;
            let side_offset = phase.sin() * 5.0;
            self.cursor_bubbles.push(CursorBubble {
                // cursor_position is the fish mouth hotspot/click point.
                position: cursor_position + vec2(2.0, side_offset),
                velocity: vec2(24.0 + (speed * 0.035).min(28.0), -32.0 - phase.cos() * 8.0),
                radius: 2.5 + phase.cos().abs() * 2.5,
                age: 0.0,
                lifetime: 0.55 + phase.sin().abs() * 0.25,
            });
        }
    }

    pub fn draw(&self) {
        clear_background(CALM_BACKGROUND);

        match self.game_state {
            GameState::Playing => {
                self.draw_clouds();

                // Draw table in lower portion
                if let Some(table) = &self.table_texture {
                    let table_height = screen_height() * TABLE_HEIGHT_RATIO;
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
            }
            GameState::Revealing => {
                self.draw_fish_reveal();
            }
            GameState::FailedReveal => {
                self.draw_failed_reveal();
            }
        }

        // The custom cursor must be the final thing submitted for this frame.
        // Drawing it during `update` puts it behind the next frame's clear.
        self.draw_cursor();
    }

    fn draw_failed_reveal(&self) {
        let elapsed = self.failed_reveal_elapsed();
        let lift_progress = if elapsed < 0.18 {
            ease_out_cubic(elapsed / 0.18)
        } else if elapsed < 0.48 {
            1.0
        } else {
            1.0 - ease_in_cubic(((elapsed - 0.48) / 0.27).clamp(0.0, 1.0))
        };
        let wobble = (elapsed * 42.0).sin() * 13.0 * lift_progress;

        self.draw_background_and_table();
        for bowl_idx in 0..3 {
            let lid_offset = if self.failed_bowl == Some(bowl_idx) {
                vec2(wobble, -self.config.bowl_size.1 * 0.72 * lift_progress)
            } else {
                Vec2::ZERO
            };
            self.draw_bowl_with_lid_offset(bowl_idx, lid_offset);
        }

        let message_size = 44.0;
        let message_width = measure_text(&self.message, None, message_size as u16, 1.0).width;
        draw_text(
            &self.message,
            screen_width() / 2.0 - message_width / 2.0,
            105.0,
            message_size,
            Color::from_rgba(205, 50, 65, 255),
        );
    }

    fn draw_fish_reveal(&self) {
        let elapsed = self.reveal_elapsed();
        let opening = ease_out_cubic((elapsed / 0.55).min(1.0));
        let fish_progress = ease_out_cubic(((elapsed - 0.18) / 0.82).clamp(0.0, 1.0));

        self.draw_background_and_table();
        for bowl_idx in 0..3 {
            let lid_lift = if bowl_idx == self.correct_bowl {
                -self.config.bowl_size.1 * 0.9 * opening
            } else {
                0.0
            };
            self.draw_bowl_with_lid_offset(bowl_idx, vec2(0.0, lid_lift));
        }

        // Preserve the game board beneath the celebration, but let the center
        // read as a distinct reveal space.
        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            Color::from_rgba(13, 18, 42, 115),
        );

        if let (Some(fish), Some(rarity)) = (&self.current_fish, &self.current_rarity) {
            let center_x = screen_width() / 2.0;
            let center_y = screen_height() / 2.0;
            self.draw_rotating_spotlight(center_x, center_y, elapsed, rarity.color());

            // Draw a centered title with the rarity highlighted in its tier color.
            let title_prefix = "You found a ";
            let rarity_title = rarity.display_name();
            let title_separator = " ";
            let fish_title = fish.name.as_str();
            let title_punctuation = "!";
            let title_size = 44.0;
            let prefix_width = measure_text(title_prefix, None, title_size as u16, 1.0).width;
            let rarity_width = measure_text(rarity_title, None, title_size as u16, 1.0).width;
            let separator_width = measure_text(title_separator, None, title_size as u16, 1.0).width;
            let fish_width = measure_text(fish_title, None, title_size as u16, 1.0).width;
            let punctuation_width =
                measure_text(title_punctuation, None, title_size as u16, 1.0).width;
            let title_start_x = center_x
                - (prefix_width + rarity_width + separator_width + fish_width + punctuation_width)
                    / 2.0;
            let title_alpha = ((elapsed - 0.55) / 0.35).clamp(0.0, 1.0);
            let rarity_color = rarity.color();
            draw_text(
                title_prefix,
                title_start_x,
                100.0,
                title_size,
                Color::new(1.0, 1.0, 1.0, title_alpha),
            );
            let rarity_x = title_start_x + prefix_width;
            draw_outlined_title_text(
                rarity_title,
                rarity_x,
                100.0,
                title_size,
                rarity_color,
                title_alpha,
            );
            draw_text(
                title_separator,
                rarity_x + rarity_width,
                100.0,
                title_size,
                Color::new(1.0, 1.0, 1.0, title_alpha),
            );
            let fish_x = rarity_x + rarity_width + separator_width;
            draw_outlined_title_text(
                fish_title,
                fish_x,
                100.0,
                title_size,
                rarity_color,
                title_alpha,
            );
            draw_text(
                title_punctuation,
                fish_x + fish_width,
                100.0,
                title_size,
                Color::new(1.0, 1.0, 1.0, title_alpha),
            );

            // The fish rises from the opened bowl and settles at center.
            let source = self.get_bowl_rect(self.correct_bowl);
            let source_center = vec2(source.x + source.w / 2.0, source.y + source.h / 2.0);
            let destination_center = vec2(center_x, center_y - 20.0);
            let fish_center = source_center + (destination_center - source_center) * fish_progress;
            let fish_size = 90.0 + (360.0 - 90.0) * fish_progress;
            let fish_x = fish_center.x - fish_size / 2.0;
            let fish_y = fish_center.y - fish_size / 2.0;

            // Draw fish texture or placeholder
            if let Some(texture) = &self.current_fish_texture {
                draw_texture_ex(
                    texture,
                    fish_x,
                    fish_y,
                    Color::new(1.0, 1.0, 1.0, ((elapsed - 0.12) / 0.25).clamp(0.0, 1.0)),
                    DrawTextureParams {
                        dest_size: Some(Vec2::new(fish_size, fish_size)),
                        ..Default::default()
                    },
                );
            } else {
                // Placeholder if texture failed to load
                draw_rectangle(
                    fish_x,
                    fish_y,
                    fish_size,
                    fish_size,
                    Color::from_rgba(60, 60, 80, 255),
                );
                let placeholder_text = "Fish Image";
                let placeholder_size = 24.0;
                let placeholder_width =
                    measure_text(placeholder_text, None, placeholder_size as u16, 1.0).width;
                draw_text(
                    placeholder_text,
                    center_x - placeholder_width / 2.0,
                    center_y,
                    placeholder_size,
                    Color::from_rgba(150, 150, 150, 255),
                );
            }

            // Draw fish fact after the movement completes.
            let chosen_fact = self
                .current_fact
                .as_deref()
                .unwrap_or("No fact is available for this fish.");
            let fact = format_reveal_fact(&fish.species, chosen_fact);
            let fact_size = 24.0;
            let max_fact_width = screen_width() - 200.0;

            // Word wrap the fact
            let wrapped_lines = wrap_text(&fact, max_fact_width, fact_size);
            let line_height = fact_size + 8.0;
            let total_fact_height = wrapped_lines.len() as f32 * line_height;
            let fact_padding = 20.0;
            let fact_text_width = wrapped_lines
                .iter()
                .map(|line| measure_text(line, None, fact_size as u16, 1.0).width)
                .fold(0.0_f32, f32::max);
            let fact_card_width =
                (fact_text_width + fact_padding * 2.0).clamp(160.0, screen_width() - 48.0);
            let fact_card_height = total_fact_height + fact_padding * 2.0;
            let fact_card_x = (center_x - fact_card_width / 2.0)
                .clamp(24.0, screen_width() - fact_card_width - 24.0);
            let fact_card_y =
                (fish_y + fish_size + 32.0).clamp(24.0, screen_height() - fact_card_height - 72.0);
            let fact_start_y = fact_card_y + fact_padding + fact_size;
            let fact_alpha = ((elapsed - 0.9) / 0.35).clamp(0.0, 1.0);

            draw_rounded_rectangle(
                fact_card_x,
                fact_card_y,
                fact_card_width,
                fact_card_height,
                14.0,
                Color::new(0.04, 0.08, 0.13, 0.74 * fact_alpha),
            );

            for (i, line) in wrapped_lines.iter().enumerate() {
                let line_width = measure_text(line, None, fact_size as u16, 1.0).width;
                draw_text(
                    line,
                    center_x - line_width / 2.0,
                    fact_start_y + i as f32 * line_height,
                    fact_size,
                    Color::new(220.0 / 255.0, 220.0 / 255.0, 220.0 / 255.0, fact_alpha),
                );
            }

            // Draw "Click to continue" button
            let button_text = "Click to continue";
            let button_size = 28.0;
            let button_width = measure_text(button_text, None, button_size as u16, 1.0).width;
            let button_y = fact_card_y + fact_card_height + 40.0;

            draw_text(
                button_text,
                center_x - button_width / 2.0,
                button_y,
                button_size,
                Color::new(100.0 / 255.0, 200.0 / 255.0, 1.0, fact_alpha),
            );
        }
    }

    fn reveal_elapsed(&self) -> f32 {
        (get_time() - self.reveal_started_at).max(0.0) as f32
    }

    fn failed_reveal_elapsed(&self) -> f32 {
        (get_time() - self.failed_reveal_started_at).max(0.0) as f32
    }

    fn draw_background_and_table(&self) {
        self.draw_clouds();

        if let Some(table) = &self.table_texture {
            let table_height = screen_height() * TABLE_HEIGHT_RATIO;
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
    }

    fn draw_clouds(&self) {
        for &(x, y, scale, speed) in &CLOUD_CLUSTERS {
            let drift = (get_time() as f32 * speed).sin() * 40.0;
            let cloud_color = Color::from_rgba(255, 255, 255, 96);
            let shadow_color = Color::from_rgba(175, 210, 228, 62);
            let cloud_x = x + drift;

            draw_circle(
                cloud_x - 30.0 * scale,
                y + 6.0 * scale,
                22.0 * scale,
                shadow_color,
            );
            draw_circle(
                cloud_x + 2.0 * scale,
                y - 6.0 * scale,
                29.0 * scale,
                cloud_color,
            );
            draw_circle(
                cloud_x + 33.0 * scale,
                y + 3.0 * scale,
                21.0 * scale,
                cloud_color,
            );
            draw_circle(
                cloud_x - 25.0 * scale,
                y + 5.0 * scale,
                19.0 * scale,
                cloud_color,
            );
        }
    }

    fn get_bowl_rect(&self, bowl_idx: usize) -> Rect {
        let bowl_width = self.config.bowl_size.0;
        let bowl_height = self.config.bowl_size.1;
        let table_height = screen_height() * TABLE_HEIGHT_RATIO;
        let table_top = screen_height() - table_height;
        let center_x = screen_width() * TABLE_CIRCLE_X_NORMALIZED[bowl_idx];
        let center_y = table_top + table_height * TABLE_CIRCLE_Y_NORMALIZED;

        Rect::new(
            center_x - bowl_width / 2.0,
            center_y - bowl_height / 2.0,
            bowl_width,
            bowl_height,
        )
    }

    fn draw_bowl(&self, bowl_idx: usize) {
        let lid_offset = if self.hover_bowl == Some(bowl_idx) {
            let hover_time = get_time() as f32;
            vec2(
                (hover_time * 9.0).sin() * 2.0,
                (hover_time * 7.0).cos() * 3.0,
            )
        } else {
            Vec2::ZERO
        };
        self.draw_bowl_with_lid_offset(bowl_idx, lid_offset);
    }

    fn draw_bowl_with_lid_offset(&self, bowl_idx: usize, lid_offset: Vec2) {
        let rect = self.get_bowl_rect(bowl_idx);

        // Draw bowl base
        if let Some(bowl_texture) = &self.bowl_textures[bowl_idx] {
            draw_texture_ex(
                bowl_texture,
                rect.x,
                rect.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(rect.w, rect.h)),
                    ..Default::default()
                },
            );
        } else {
            // Fallback to placeholder if texture failed to load
            draw_rectangle(
                rect.x,
                rect.y,
                rect.w,
                rect.h,
                Color::from_rgba(160, 140, 100, 255),
            );
        }

        // Draw lid on top
        if let Some(lid_texture) = &self.lid_textures[bowl_idx] {
            draw_texture_ex(
                lid_texture,
                rect.x + lid_offset.x,
                rect.y
                    + self.config.closed_lid_y_offset
                    + LID_CLOSED_Y_CORRECTIONS[bowl_idx]
                    + lid_offset.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(rect.w, rect.h)),
                    ..Default::default()
                },
            );
        }
    }

    fn draw_rotating_spotlight(&self, center_x: f32, center_y: f32, elapsed: f32, color: Color) {
        let radius = screen_width().max(screen_height()) * 0.85;
        for beam in 0..6 {
            let angle = elapsed * 0.9 + beam as f32 * std::f32::consts::TAU / 6.0;
            let spread = 0.18;
            let left = vec2(angle.cos(), angle.sin()) * radius;
            let right_angle = angle + spread;
            let right = vec2(right_angle.cos(), right_angle.sin()) * radius;
            draw_triangle(
                vec2(center_x, center_y),
                vec2(center_x, center_y) + left,
                vec2(center_x, center_y) + right,
                Color::new(color.r, color.g, color.b, 0.09),
            );
        }
    }
}

fn resolve_asset_root() -> PathBuf {
    if let Ok(executable_path) = std::env::current_exe() {
        if let Some(executable_directory) = executable_path.parent() {
            let packaged_assets = executable_directory.join("assets");
            if packaged_assets.is_dir() {
                return packaged_assets;
            }
        }
    }

    let working_directory_assets = PathBuf::from("assets");
    if working_directory_assets.is_dir() {
        return working_directory_assets;
    }

    eprintln!(
        "Could not find an assets directory beside the executable or in the working directory."
    );
    working_directory_assets
}

async fn load_texture_safe(path: &Path) -> Option<Texture2D> {
    let path_string = path.to_string_lossy();
    match load_texture(&path_string).await {
        Ok(texture) => Some(texture),
        Err(e) => {
            eprintln!("Failed to load texture {}: {}", path.display(), e);
            None
        }
    }
}

async fn load_lid_texture_safe(path: &Path) -> Option<Texture2D> {
    let texture = load_texture_safe(path).await?;
    texture.set_filter(FilterMode::Linear);
    Some(texture)
}

async fn load_cursor_texture_safe(path: &Path) -> Option<Texture2D> {
    let texture = load_texture_safe(path).await?;
    texture.set_filter(FilterMode::Nearest);
    Some(texture)
}

fn is_point_in_rect(point: (f32, f32), rect: Rect) -> bool {
    point.0 >= rect.x
        && point.0 <= rect.x + rect.w
        && point.1 >= rect.y
        && point.1 <= rect.y + rect.h
}

fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

fn ease_in_cubic(t: f32) -> f32 {
    t.powi(3)
}

fn draw_rounded_rectangle(x: f32, y: f32, width: f32, height: f32, radius: f32, color: Color) {
    let radius = radius.min(width / 2.0).min(height / 2.0);

    draw_rectangle(x + radius, y, width - radius * 2.0, height, color);
    draw_rectangle(x, y + radius, width, height - radius * 2.0, color);
    draw_circle(x + radius, y + radius, radius, color);
    draw_circle(x + width - radius, y + radius, radius, color);
    draw_circle(x + radius, y + height - radius, radius, color);
    draw_circle(x + width - radius, y + height - radius, radius, color);
}

fn draw_outlined_title_text(text: &str, x: f32, y: f32, font_size: f32, color: Color, alpha: f32) {
    let outline = Color::new(0.02, 0.03, 0.05, alpha);
    for (offset_x, offset_y) in [
        (-2.0, -2.0),
        (0.0, -2.0),
        (2.0, -2.0),
        (-2.0, 0.0),
        (2.0, 0.0),
        (-2.0, 2.0),
        (0.0, 2.0),
        (2.0, 2.0),
    ] {
        draw_text(text, x + offset_x, y + offset_y, font_size, outline);
    }
    draw_text(
        text,
        x,
        y,
        font_size,
        Color::new(color.r, color.g, color.b, alpha),
    );
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

fn format_reveal_fact(species: &str, fact: &str) -> String {
    format!("{species}: {fact}")
}

#[cfg(test)]
mod tests {
    use super::format_reveal_fact;

    #[test]
    fn reveal_fact_prefixes_the_species_and_colon() {
        assert_eq!(
            format_reveal_fact("Amphiprioninae", "Lives among sea anemones."),
            "Amphiprioninae: Lives among sea anemones."
        );
    }
}
