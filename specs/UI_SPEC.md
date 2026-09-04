# UI Rendering System - Technical Specification
**Game:** Which Bowl is the Fish In?  
**Framework:** egui (Rust immediate-mode GUI)  
**Version:** 1.0  
**Last Updated:** 2026-09-04

---

## 1. Module Structure

### 1.1 Directory Layout
```
src/
├── main.rs                  # App initialization, state management, screen routing
├── state.rs                 # Game state, fish data, journal tracking
├── animation.rs             # Animation state machine for reveals
└── ui/
    ├── mod.rs              # Common UI utilities, colors, layout helpers
    ├── intro.rs            # Intro screen rendering
    ├── playing.rs          # Main game screen (bowl selection)
    ├── revealing.rs        # Fish reveal animation screen
    └── journal.rs          # Discovery journal screen
```

### 1.2 Module Responsibilities

**`ui/mod.rs`** - Exports all screen modules and provides:
- Common button builders
- Text styling constants
- Color palette
- Layout helper functions
- Shared UI components (back buttons, headers, etc.)

**Screen Modules** - Each screen module exports a single public function:
```rust
pub fn render(ctx: &egui::Context, state: &mut GameState) -> ScreenTransition
```

All screen modules return a `ScreenTransition` enum indicating next screen:
```rust
pub enum ScreenTransition {
    Stay,                    // Remain on current screen
    ToIntro,                 // Return to intro
    ToPlaying,               // Start/continue game
    ToRevealing,             // Trigger reveal animation
    ToJournal,               // Open journal
}
```

---

## 2. Intro Screen (`ui/intro.rs`)

### 2.1 Purpose
Welcome screen that introduces the game with humor and provides navigation to start playing or view the journal.

### 2.2 Layout Structure
```
┌──────────────────────────────────────────────┐
│                                              │
│                                              │
│        Which Bowl is the Fish In?            │
│                                              │
│    This game is written in Rust, it's       │
│    the most performant, memory safe fish    │
│    guessing game you'll ever see in         │
│    your life                                 │
│                                              │
│            ┌───────────────┐                 │
│            │  Start Game   │                 │
│            └───────────────┘                 │
│                                              │
│            ┌───────────────┐                 │
│            │ View Journal  │                 │
│            └───────────────┘                 │
│                                              │
└──────────────────────────────────────────────┘
```

### 2.3 Implementation

```rust
use egui::{Align, Layout, Vec2};
use crate::state::GameState;
use super::{ScreenTransition, colors, button_primary, button_secondary};

pub fn render(ctx: &egui::Context, state: &GameState) -> ScreenTransition {
    let mut transition = ScreenTransition::Stay;
    
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(60.0);
            
            // Title
            ui.heading(egui::RichText::new("Which Bowl is the Fish In?")
                .size(48.0)
                .color(colors::TITLE));
            
            ui.add_space(40.0);
            
            // The joke
            ui.label(egui::RichText::new(
                "This game is written in Rust, it's the most performant,\n\
                 memory safe fish guessing game you'll ever see in your life"
            )
                .size(18.0)
                .color(colors::SUBTITLE));
            
            ui.add_space(60.0);
            
            // Start Game button
            if button_primary(ui, "Start Game", Vec2::new(200.0, 50.0)) {
                transition = ScreenTransition::ToPlaying;
            }
            
            ui.add_space(20.0);
            
            // View Journal button
            if button_secondary(ui, "View Journal", Vec2::new(200.0, 50.0)) {
                transition = ScreenTransition::ToJournal;
            }
            
            ui.add_space(20.0);
            
            // Stats display (small, subtle)
            if state.journal.total_discovered() > 0 {
                ui.label(egui::RichText::new(
                    format!("Fish discovered: {}/{}", 
                        state.journal.total_discovered(),
                        state.journal.total_fish())
                )
                    .size(14.0)
                    .color(colors::MUTED));
            }
        });
    });
    
    transition
}
```

### 2.4 Design Requirements
- **Background**: Solid color (light blue-gray: `#E8F4F8`)
- **Title font**: 48px, bold, dark blue (`#1A3A52`)
- **Body text**: 18px, medium weight, gray-blue (`#4A6B7C`)
- **Buttons**: Primary (blue) and secondary (gray) styles
- **Spacing**: Generous vertical spacing (40-60px between elements)
- **Centering**: All content vertically and horizontally centered

---

## 3. Playing Screen (`ui/playing.rs`)

### 3.1 Purpose
Main game screen where players click on one of three bowls to guess which contains the fish.

### 3.2 Layout Structure
```
┌──────────────────────────────────────────────┐
│ [Journal]                                    │
│                                              │
│          Which bowl has the fish?            │
│                                              │
│                                              │
│     ┌─────┐    ┌─────┐    ┌─────┐          │
│     │     │    │     │    │     │          │
│     │ 🥣  │    │ 🥣  │    │ 🥣  │          │
│     │     │    │     │    │     │          │
│     └─────┘    └─────┘    └─────┘          │
│                                              │
│                                              │
│          ❌ Wrong bowl! Try again            │
│                                              │
└──────────────────────────────────────────────┘
```

### 3.3 State Machine

The playing screen tracks:
- **`current_fish`**: The fish to be found (selected at screen entry)
- **`correct_bowl`**: Index (0-2) of bowl containing fish
- **`wrong_guess_message`**: Optional error message to display
- **`hover_bowl`**: Index of currently hovered bowl (for visual feedback)

### 3.4 Implementation

```rust
use egui::{Vec2, Sense, Color32, Stroke, Rounding};
use crate::state::{GameState, BowlState};
use super::{ScreenTransition, colors};

pub fn render(ctx: &egui::Context, state: &mut GameState) -> ScreenTransition {
    let mut transition = ScreenTransition::Stay;
    
    egui::CentralPanel::default().show(ctx, |ui| {
        // Top-right journal button
        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
            if ui.button("📖 Journal").clicked() {
                transition = ScreenTransition::ToJournal;
            }
        });
        
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);
            
            // Question text
            ui.heading(egui::RichText::new("Which bowl has the fish?")
                .size(32.0)
                .color(colors::TITLE));
            
            ui.add_space(60.0);
            
            // Three bowls in horizontal layout
            ui.horizontal(|ui| {
                ui.add_space((ui.available_width() - 3.0 * 150.0 - 2.0 * 40.0) / 2.0);
                
                for bowl_idx in 0..3 {
                    let clicked = render_bowl(ui, bowl_idx, state.playing.hover_bowl);
                    
                    if clicked {
                        if bowl_idx == state.playing.correct_bowl {
                            // Correct guess!
                            state.journal.discover_fish(&state.playing.current_fish);
                            state.stats.record_correct_guess();
                            transition = ScreenTransition::ToRevealing;
                        } else {
                            // Wrong guess
                            state.playing.wrong_guess_message = Some(
                                format!("❌ Wrong bowl! The fish isn't here.")
                            );
                            state.stats.record_wrong_guess();
                        }
                    }
                    
                    if bowl_idx < 2 {
                        ui.add_space(40.0);
                    }
                }
            });
            
            ui.add_space(60.0);
            
            // Error message display
            if let Some(ref message) = state.playing.wrong_guess_message {
                ui.label(egui::RichText::new(message)
                    .size(20.0)
                    .color(colors::ERROR));
            }
        });
    });
    
    transition
}

fn render_bowl(
    ui: &mut egui::Ui, 
    bowl_idx: usize, 
    hover_bowl: Option<usize>
) -> bool {
    let bowl_size = Vec2::new(150.0, 150.0);
    let is_hovered = hover_bowl == Some(bowl_idx);
    
    let (rect, response) = ui.allocate_exact_size(bowl_size, Sense::click());
    
    // Visual state based on hover
    let fill_color = if is_hovered {
        colors::BOWL_HOVER
    } else {
        colors::BOWL_DEFAULT
    };
    
    let stroke = if is_hovered {
        Stroke::new(3.0, colors::BOWL_BORDER_HOVER)
    } else {
        Stroke::new(2.0, colors::BOWL_BORDER)
    };
    
    // Draw bowl
    ui.painter().rect(
        rect,
        Rounding::same(12.0),
        fill_color,
        stroke,
    );
    
    // Bowl emoji/icon in center
    let text_pos = rect.center() - Vec2::new(20.0, 20.0);
    ui.painter().text(
        text_pos,
        egui::Align2::CENTER_CENTER,
        "🥣",
        egui::FontId::proportional(40.0),
        colors::BOWL_ICON,
    );
    
    response.clicked()
}
```

### 3.5 Bowl Rendering Details

**Bowl Visual States:**
1. **Default**: Light background, subtle border
2. **Hover**: Slightly darker background, thicker border, subtle scale (can add transform)
3. **Clicked**: Brief highlight animation (optional, can use egui's animation API)

**Bowl Dimensions:**
- Size: 150x150 pixels
- Border radius: 12px
- Spacing between bowls: 40px
- Icon size: 40px emoji

**Colors:**
```rust
pub mod colors {
    use egui::Color32;
    
    // Bowl states
    pub const BOWL_DEFAULT: Color32 = Color32::from_rgb(220, 235, 245);
    pub const BOWL_HOVER: Color32 = Color32::from_rgb(180, 215, 235);
    pub const BOWL_BORDER: Color32 = Color32::from_rgb(120, 180, 210);
    pub const BOWL_BORDER_HOVER: Color32 = Color32::from_rgb(80, 140, 180);
    pub const BOWL_ICON: Color32 = Color32::from_rgb(100, 100, 100);
}
```

### 3.6 Interaction Flow

1. **Screen Entry**:
   - Select random fish from undiscovered fish pool
   - Randomly assign fish to one of three bowls
   - Clear wrong_guess_message
   - Initialize hover state

2. **During Gameplay**:
   - Track cursor position to highlight hovered bowl
   - On bowl click:
     - If correct → transition to Revealing screen
     - If wrong → display error message, increment wrong guess counter

3. **Journal Button**:
   - Always accessible in top-right corner
   - Preserves game state when returning

---

## 4. Revealing Screen (`ui/revealing.rs`)

### 4.1 Purpose
Animated reveal screen that shows the discovered fish, displays a fun fact, indicates rarity, and provides a continue button.

### 4.2 Layout Structure
```
┌──────────────────────────────────────────────┐
│                                              │
│              You found a...                  │
│                                              │
│                  ┌─────┐                     │
│                  │     │                     │
│                  │ 🐠  │  (fish image)       │
│                  │     │                     │
│                  └─────┘                     │
│                                              │
│              Clownfish  ⭐⭐⭐               │
│                                              │
│  ┌────────────────────────────────────────┐  │
│  │ Did you know?                          │  │
│  │                                        │  │
│  │ Clownfish live in anemones and are    │  │
│  │ immune to their stings!                │  │
│  └────────────────────────────────────────┘  │
│                                              │
│            ┌───────────────┐                 │
│            │   Continue    │                 │
│            └───────────────┘                 │
│                                              │
└──────────────────────────────────────────────┘
```

### 4.3 Animation Phases

The revealing screen uses a state machine with the following phases:

1. **FadeIn** (0.0 - 0.5s): Title and fish image fade in
2. **ScaleIn** (0.5 - 1.0s): Fish image scales from 0.5x to 1.0x with bounce easing
3. **ShowName** (1.0 - 1.5s): Fish name and rarity fade in
4. **ShowFact** (1.5 - 2.0s): Fact card slides up and fades in
5. **ShowButton** (2.0s+): Continue button appears, user can proceed

### 4.4 Implementation

```rust
use egui::{Vec2, Align, Rounding, Color32};
use crate::state::{GameState, Fish};
use crate::animation::{AnimationState, easing};
use super::{ScreenTransition, colors, button_primary};

pub fn render(ctx: &egui::Context, state: &mut GameState) -> ScreenTransition {
    let mut transition = ScreenTransition::Stay;
    
    // Update animation timer
    let delta_time = ctx.input(|i| i.stable_dt);
    state.revealing.animation_time += delta_time;
    
    let anim_time = state.revealing.animation_time;
    let fish = &state.revealing.revealed_fish;
    
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);
            
            // Title with fade-in (Phase 1: 0.0 - 0.5s)
            let title_alpha = (anim_time / 0.5).min(1.0);
            ui.label(
                egui::RichText::new("You found a...")
                    .size(28.0)
                    .color(colors::TITLE.linear_multiply(title_alpha))
            );
            
            ui.add_space(40.0);
            
            // Fish image with fade + scale (Phase 1 & 2: 0.0 - 1.0s)
            if anim_time >= 0.0 {
                let fade_alpha = ((anim_time - 0.0) / 0.5).min(1.0);
                let scale_progress = ((anim_time - 0.5) / 0.5).clamp(0.0, 1.0);
                let scale = 0.5 + 0.5 * easing::ease_out_back(scale_progress);
                
                render_fish_image(ui, fish, fade_alpha, scale);
            }
            
            ui.add_space(30.0);
            
            // Fish name and rarity (Phase 3: 1.0 - 1.5s)
            if anim_time >= 1.0 {
                let name_alpha = ((anim_time - 1.0) / 0.5).min(1.0);
                
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(&fish.name)
                            .size(32.0)
                            .color(colors::FISH_NAME.linear_multiply(name_alpha))
                    );
                    
                    ui.add_space(10.0);
                    
                    // Rarity stars
                    let stars = "⭐".repeat(fish.rarity as usize);
                    ui.label(
                        egui::RichText::new(stars)
                            .size(24.0)
                            .color(Color32::GOLD.linear_multiply(name_alpha))
                    );
                });
            }
            
            ui.add_space(40.0);
            
            // Fact card (Phase 4: 1.5 - 2.0s)
            if anim_time >= 1.5 {
                let fact_alpha = ((anim_time - 1.5) / 0.5).min(1.0);
                let fact_offset = 20.0 * (1.0 - fact_alpha); // Slide up
                
                ui.add_space(fact_offset);
                
                render_fact_card(ui, &fish.fact, fact_alpha);
            }
            
            ui.add_space(40.0);
            
            // Continue button (Phase 5: 2.0s+)
            if anim_time >= 2.0 {
                let button_alpha = ((anim_time - 2.0) / 0.3).min(1.0);
                
                ui.add_enabled_ui(button_alpha >= 0.99, |ui| {
                    if button_primary(ui, "Continue", Vec2::new(200.0, 50.0)) {
                        transition = ScreenTransition::ToPlaying;
                        state.revealing.reset();
                    }
                });
            }
        });
    });
    
    // Request repaint for smooth animation
    if anim_time < 2.3 {
        ctx.request_repaint();
    }
    
    transition
}

fn render_fish_image(ui: &mut egui::Ui, fish: &Fish, alpha: f32, scale: f32) {
    let image_size = Vec2::new(200.0 * scale, 200.0 * scale);
    
    // Placeholder: In production, load actual fish image
    let (rect, _) = ui.allocate_exact_size(image_size, egui::Sense::hover());
    
    ui.painter().rect(
        rect,
        Rounding::same(12.0),
        Color32::from_rgba_premultiplied(200, 220, 255, (255.0 * alpha) as u8),
        egui::Stroke::new(2.0, colors::FISH_IMAGE_BORDER.linear_multiply(alpha)),
    );
    
    // Fish emoji placeholder
    let emoji_pos = rect.center();
    ui.painter().text(
        emoji_pos,
        egui::Align2::CENTER_CENTER,
        &fish.emoji,
        egui::FontId::proportional(80.0 * scale),
        Color32::WHITE.linear_multiply(alpha),
    );
}

fn render_fact_card(ui: &mut egui::Ui, fact: &str, alpha: f32) {
    let card_width = 500.0;
    
    egui::Frame::none()
        .fill(colors::FACT_CARD_BG.linear_multiply(alpha))
        .stroke(egui::Stroke::new(
            1.0, 
            colors::FACT_CARD_BORDER.linear_multiply(alpha)
        ))
        .rounding(Rounding::same(8.0))
        .inner_margin(egui::Margin::same(20.0))
        .show(ui, |ui| {
            ui.set_max_width(card_width);
            
            ui.label(
                egui::RichText::new("Did you know?")
                    .size(18.0)
                    .color(colors::FACT_TITLE.linear_multiply(alpha))
                    .strong()
            );
            
            ui.add_space(8.0);
            
            ui.label(
                egui::RichText::new(fact)
                    .size(16.0)
                    .color(colors::FACT_TEXT.linear_multiply(alpha))
            );
        });
}
```

### 4.5 Animation Integration

The revealing screen coordinates with the animation system (`src/animation.rs`):

```rust
// In src/animation.rs
pub struct RevealingState {
    pub revealed_fish: Fish,
    pub animation_time: f32,
}

impl RevealingState {
    pub fn new(fish: Fish) -> Self {
        Self {
            revealed_fish: fish,
            animation_time: 0.0,
        }
    }
    
    pub fn reset(&mut self) {
        self.animation_time = 0.0;
    }
}

pub mod easing {
    pub fn ease_out_back(t: f32) -> f32 {
        let c1 = 1.70158;
        let c3 = c1 + 1.0;
        1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
    }
}
```

### 4.6 Visual Design

**Colors:**
```rust
pub const FISH_NAME: Color32 = Color32::from_rgb(30, 60, 90);
pub const FISH_IMAGE_BORDER: Color32 = Color32::from_rgb(100, 150, 200);
pub const FACT_CARD_BG: Color32 = Color32::from_rgb(250, 250, 250);
pub const FACT_CARD_BORDER: Color32 = Color32::from_rgb(200, 200, 200);
pub const FACT_TITLE: Color32 = Color32::from_rgb(60, 100, 140);
pub const FACT_TEXT: Color32 = Color32::from_rgb(80, 80, 80);
```

**Rarity Display:**
- Common (1★): Single star
- Uncommon (2★): Two stars
- Rare (3★): Three stars
- Epic (4★): Four stars, purple color
- Legendary (5★): Five stars, gold color, glow effect

---

## 5. Journal Screen (`ui/journal.rs`)

### 5.1 Purpose
Display all discovered fish in a scrollable grid, show undiscovered fish as "???", and provide game statistics.

### 5.2 Layout Structure
```
┌──────────────────────────────────────────────┐
│  [← Back]          Fish Journal              │
├──────────────────────────────────────────────┤
│                                              │
│  Completion: 12/50 (24%)                     │
│  Total Guesses: 48    Accuracy: 75%          │
│                                              │
├──────────────────────────────────────────────┤
│  ┌────┐ ┌────┐ ┌────┐ ┌────┐ ┌────┐        │
│  │ 🐠 │ │ 🐡 │ │ 🦈 │ │ ??? │ │ ??? │       │
│  │Clow│ │Puff│ │Shar│ │ ??? │ │ ??? │       │
│  └────┘ └────┘ └────┘ └────┘ └────┘        │
│                                              │
│  ┌────┐ ┌────┐ ┌────┐ ┌────┐ ┌────┐        │
│  │ 🐟 │ │ ??? │ │ ??? │ │ ??? │ │ ??? │      │
│  │Tuna│ │ ??? │ │ ??? │ │ ??? │ │ ??? │      │
│  └────┘ └────┘ └────┘ └────┘ └────┘        │
│                                              │
│  (scrollable grid continues...)              │
│                                              │
└──────────────────────────────────────────────┘
```

### 5.3 Implementation

```rust
use egui::{Vec2, Rounding, Color32, ScrollArea, Grid};
use crate::state::{GameState, Fish, JournalEntry};
use super::{ScreenTransition, colors};

pub fn render(ctx: &egui::Context, state: &GameState) -> ScreenTransition {
    let mut transition = ScreenTransition::Stay;
    
    egui::CentralPanel::default().show(ctx, |ui| {
        // Header with back button
        ui.horizontal(|ui| {
            if ui.button("← Back").clicked() {
                transition = ScreenTransition::ToIntro;
            }
            
            ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::LeftToRight), |ui| {
                ui.heading(egui::RichText::new("Fish Journal")
                    .size(32.0)
                    .color(colors::TITLE));
            });
            
            // Invisible spacer to balance layout
            ui.add_space(ui.available_width() - 60.0);
        });
        
        ui.separator();
        ui.add_space(20.0);
        
        // Statistics panel
        render_stats(ui, state);
        
        ui.add_space(20.0);
        ui.separator();
        ui.add_space(20.0);
        
        // Scrollable fish grid
        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                render_fish_grid(ui, state);
            });
    });
    
    transition
}

fn render_stats(ui: &mut egui::Ui, state: &GameState) {
    let discovered = state.journal.total_discovered();
    let total = state.journal.total_fish();
    let completion_pct = (discovered as f32 / total as f32 * 100.0) as u32;
    
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(format!("Completion: {}/{} ({}%)", discovered, total, completion_pct))
                .size(18.0)
                .color(colors::STAT_TEXT)
        );
        
        ui.add_space(40.0);
        
        ui.label(
            egui::RichText::new(format!("Total Guesses: {}", state.stats.total_guesses()))
                .size(18.0)
                .color(colors::STAT_TEXT)
        );
        
        ui.add_space(40.0);
        
        let accuracy = if state.stats.total_guesses() > 0 {
            (state.stats.correct_guesses() as f32 / state.stats.total_guesses() as f32 * 100.0) as u32
        } else {
            0
        };
        
        ui.label(
            egui::RichText::new(format!("Accuracy: {}%", accuracy))
                .size(18.0)
                .color(if accuracy >= 75 {
                    colors::STAT_GOOD
                } else if accuracy >= 50 {
                    colors::STAT_MEDIUM
                } else {
                    colors::STAT_BAD
                })
        );
    });
}

fn render_fish_grid(ui: &mut egui::Ui, state: &GameState) {
    const CARD_SIZE: Vec2 = Vec2::new(120.0, 140.0);
    const COLUMNS: usize = 5;
    
    Grid::new("fish_grid")
        .spacing([20.0, 20.0])
        .show(ui, |ui| {
            for (idx, entry) in state.journal.entries().iter().enumerate() {
                render_fish_card(ui, entry, CARD_SIZE);
                
                if (idx + 1) % COLUMNS == 0 {
                    ui.end_row();
                }
            }
        });
}

fn render_fish_card(ui: &mut egui::Ui, entry: &JournalEntry, size: Vec2) {
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::hover());
    
    let (bg_color, border_color) = if entry.discovered {
        (colors::CARD_DISCOVERED_BG, colors::CARD_DISCOVERED_BORDER)
    } else {
        (colors::CARD_UNKNOWN_BG, colors::CARD_UNKNOWN_BORDER)
    };
    
    // Card background
    ui.painter().rect(
        rect,
        Rounding::same(8.0),
        bg_color,
        egui::Stroke::new(2.0, border_color),
    );
    
    if entry.discovered {
        // Show fish emoji
        let emoji_pos = rect.center_top() + Vec2::new(0.0, 30.0);
        ui.painter().text(
            emoji_pos,
            egui::Align2::CENTER_TOP,
            &entry.fish.emoji,
            egui::FontId::proportional(50.0),
            colors::FISH_EMOJI,
        );
        
        // Show fish name (truncated if needed)
        let name_pos = rect.center_bottom() - Vec2::new(0.0, 25.0);
        let name = if entry.fish.name.len() > 12 {
            format!("{}...", &entry.fish.name[..9])
        } else {
            entry.fish.name.clone()
        };
        
        ui.painter().text(
            name_pos,
            egui::Align2::CENTER_TOP,
            &name,
            egui::FontId::proportional(14.0),
            colors::FISH_NAME_CARD,
        );
        
        // Show rarity stars
        let stars_pos = rect.center_bottom() - Vec2::new(0.0, 10.0);
        let stars = "⭐".repeat(entry.fish.rarity as usize);
        ui.painter().text(
            stars_pos,
            egui::Align2::CENTER_TOP,
            &stars,
            egui::FontId::proportional(12.0),
            Color32::GOLD,
        );
    } else {
        // Show ??? for undiscovered
        let mystery_pos = rect.center();
        ui.painter().text(
            mystery_pos,
            egui::Align2::CENTER_CENTER,
            "???",
            egui::FontId::proportional(40.0),
            colors::MYSTERY_TEXT,
        );
    }
    
    // Tooltip on hover (discovered fish only)
    if entry.discovered && response.hovered() {
        egui::show_tooltip(ui.ctx(), ui.layer_id(), response.id, |ui| {
            ui.label(&entry.fish.name);
            ui.label(&entry.fish.fact);
        });
    }
}
```

### 5.4 Grid Layout

**Grid Configuration:**
- Columns: 5 cards per row
- Card size: 120x140 pixels
- Spacing: 20px horizontal, 20px vertical
- Responsive: Adjust columns based on window width

**Card States:**
1. **Discovered**: Full color, shows fish emoji, name, and rarity
2. **Undiscovered**: Grayed out, shows "???"
3. **Hovered (discovered)**: Slight highlight, shows tooltip with full name and fact

### 5.5 Statistics Panel

**Displayed Stats:**
- **Completion**: `X/Y (Z%)` - discovered fish count and percentage
- **Total Guesses**: Total number of bowl clicks
- **Accuracy**: Percentage of correct first guesses

**Color Coding:**
- Accuracy ≥ 75%: Green
- Accuracy 50-74%: Yellow
- Accuracy < 50%: Red

---

## 6. Common UI Utilities (`ui/mod.rs`)

### 6.1 Module Exports

```rust
pub mod intro;
pub mod playing;
pub mod revealing;
pub mod journal;

pub use intro::render as render_intro;
pub use playing::render as render_playing;
pub use revealing::render as render_revealing;
pub use journal::render as render_journal;

pub enum ScreenTransition {
    Stay,
    ToIntro,
    ToPlaying,
    ToRevealing,
    ToJournal,
}

pub mod colors {
    use egui::Color32;
    
    // Global colors
    pub const TITLE: Color32 = Color32::from_rgb(26, 58, 82);
    pub const SUBTITLE: Color32 = Color32::from_rgb(74, 107, 124);
    pub const MUTED: Color32 = Color32::from_rgb(140, 160, 170);
    pub const ERROR: Color32 = Color32::from_rgb(220, 50, 50);
    
    // Bowl colors (Playing screen)
    pub const BOWL_DEFAULT: Color32 = Color32::from_rgb(220, 235, 245);
    pub const BOWL_HOVER: Color32 = Color32::from_rgb(180, 215, 235);
    pub const BOWL_BORDER: Color32 = Color32::from_rgb(120, 180, 210);
    pub const BOWL_BORDER_HOVER: Color32 = Color32::from_rgb(80, 140, 180);
    pub const BOWL_ICON: Color32 = Color32::from_rgb(100, 100, 100);
    
    // Revealing screen colors
    pub const FISH_NAME: Color32 = Color32::from_rgb(30, 60, 90);
    pub const FISH_IMAGE_BORDER: Color32 = Color32::from_rgb(100, 150, 200);
    pub const FACT_CARD_BG: Color32 = Color32::from_rgb(250, 250, 250);
    pub const FACT_CARD_BORDER: Color32 = Color32::from_rgb(200, 200, 200);
    pub const FACT_TITLE: Color32 = Color32::from_rgb(60, 100, 140);
    pub const FACT_TEXT: Color32 = Color32::from_rgb(80, 80, 80);
    
    // Journal screen colors
    pub const STAT_TEXT: Color32 = Color32::from_rgb(60, 60, 60);
    pub const STAT_GOOD: Color32 = Color32::from_rgb(50, 180, 50);
    pub const STAT_MEDIUM: Color32 = Color32::from_rgb(220, 180, 50);
    pub const STAT_BAD: Color32 = Color32::from_rgb(220, 80, 80);
    
    pub const CARD_DISCOVERED_BG: Color32 = Color32::from_rgb(255, 255, 255);
    pub const CARD_DISCOVERED_BORDER: Color32 = Color32::from_rgb(180, 200, 220);
    pub const CARD_UNKNOWN_BG: Color32 = Color32::from_rgb(240, 240, 240);
    pub const CARD_UNKNOWN_BORDER: Color32 = Color32::from_rgb(200, 200, 200);
    pub const FISH_EMOJI: Color32 = Color32::from_rgb(50, 50, 50);
    pub const FISH_NAME_CARD: Color32 = Color32::from_rgb(60, 60, 60);
    pub const MYSTERY_TEXT: Color32 = Color32::from_rgb(180, 180, 180);
}
```

### 6.2 Button Helpers

```rust
use egui::{Button, Vec2, RichText, Color32, Rounding};

pub fn button_primary(ui: &mut egui::Ui, text: &str, size: Vec2) -> bool {
    let button = Button::new(
        RichText::new(text)
            .size(18.0)
            .color(Color32::WHITE)
    )
    .min_size(size)
    .fill(Color32::from_rgb(60, 130, 200))
    .rounding(Rounding::same(8.0));
    
    ui.add(button).clicked()
}

pub fn button_secondary(ui: &mut egui::Ui, text: &str, size: Vec2) -> bool {
    let button = Button::new(
        RichText::new(text)
            .size(18.0)
            .color(Color32::from_rgb(60, 60, 60))
    )
    .min_size(size)
    .fill(Color32::from_rgb(220, 220, 220))
    .rounding(Rounding::same(8.0));
    
    ui.add(button).clicked()
}

pub fn button_small(ui: &mut egui::Ui, text: &str) -> bool {
    let button = Button::new(
        RichText::new(text)
            .size(14.0)
    )
    .rounding(Rounding::same(4.0));
    
    ui.add(button).clicked()
}
```

### 6.3 Layout Helpers

```rust
use egui::{Vec2, Ui, Response};

/// Centers content vertically and horizontally
pub fn centered_content<R>(
    ui: &mut Ui,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> R {
    ui.with_layout(
        egui::Layout::centered_and_justified(egui::Direction::TopDown),
        add_contents,
    )
    .inner
}

/// Creates a card-style frame
pub fn card_frame<R>(
    ui: &mut Ui,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> R {
    egui::Frame::none()
        .fill(colors::CARD_DISCOVERED_BG)
        .stroke(egui::Stroke::new(2.0, colors::CARD_DISCOVERED_BORDER))
        .rounding(Rounding::same(12.0))
        .inner_margin(egui::Margin::same(16.0))
        .show(ui, add_contents)
        .inner
}

/// Creates a panel with header and content
pub fn panel_with_header<R>(
    ui: &mut Ui,
    header: &str,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> R {
    ui.vertical(|ui| {
        ui.heading(RichText::new(header).size(24.0).color(colors::TITLE));
        ui.separator();
        ui.add_space(10.0);
        add_contents(ui)
    })
    .inner
}
```

### 6.4 Text Styling Constants

```rust
pub mod text_styles {
    use egui::FontId;
    
    pub const HEADING_LARGE: f32 = 48.0;
    pub const HEADING_MEDIUM: f32 = 32.0;
    pub const HEADING_SMALL: f32 = 24.0;
    pub const BODY_LARGE: f32 = 18.0;
    pub const BODY_MEDIUM: f32 = 16.0;
    pub const BODY_SMALL: f32 = 14.0;
    pub const CAPTION: f32 = 12.0;
    
    pub fn heading_large() -> FontId {
        FontId::proportional(HEADING_LARGE)
    }
    
    pub fn heading_medium() -> FontId {
        FontId::proportional(HEADING_MEDIUM)
    }
    
    pub fn body_large() -> FontId {
        FontId::proportional(BODY_LARGE)
    }
}
```

### 6.5 Animation Helpers

```rust
use egui::Color32;

/// Linear interpolation between two values
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Linear interpolation between two colors
pub fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    Color32::from_rgb(
        lerp(a.r() as f32, b.r() as f32, t) as u8,
        lerp(a.g() as f32, b.g() as f32, t) as u8,
        lerp(a.b() as f32, b.b() as f32, t) as u8,
    )
}

/// Fade a color by alpha multiplier
pub fn fade_color(color: Color32, alpha: f32) -> Color32 {
    color.linear_multiply(alpha)
}
```

---

## 7. Integration with Main App

### 7.1 Screen Enum

```rust
// In src/main.rs or src/state.rs
pub enum Screen {
    Intro,
    Playing,
    Revealing,
    Journal,
}
```

### 7.2 Main Render Loop

```rust
use eframe::egui;
use crate::ui::{render_intro, render_playing, render_revealing, render_journal, ScreenTransition};

impl eframe::App for FishGame {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let transition = match self.current_screen {
            Screen::Intro => render_intro(ctx, &self.state),
            Screen::Playing => render_playing(ctx, &mut self.state),
            Screen::Revealing => render_revealing(ctx, &mut self.state),
            Screen::Journal => render_journal(ctx, &self.state),
        };
        
        // Handle screen transitions
        match transition {
            ScreenTransition::ToIntro => {
                self.current_screen = Screen::Intro;
            }
            ScreenTransition::ToPlaying => {
                self.state.start_new_round();
                self.current_screen = Screen::Playing;
            }
            ScreenTransition::ToRevealing => {
                self.current_screen = Screen::Revealing;
            }
            ScreenTransition::ToJournal => {
                self.current_screen = Screen::Journal;
            }
            ScreenTransition::Stay => {
                // No transition
            }
        }
    }
}
```

---

## 8. Visual Design System

### 8.1 Color Palette

```rust
// Primary brand colors
const BRAND_PRIMARY: Color32 = Color32::from_rgb(60, 130, 200);    // Blue
const BRAND_SECONDARY: Color32 = Color32::from_rgb(100, 180, 160); // Teal
const BRAND_ACCENT: Color32 = Color32::from_rgb(255, 200, 50);     // Gold

// Semantic colors
const SUCCESS: Color32 = Color32::from_rgb(50, 180, 50);
const WARNING: Color32 = Color32::from_rgb(220, 180, 50);
const ERROR: Color32 = Color32::from_rgb(220, 50, 50);
const INFO: Color32 = Color32::from_rgb(60, 130, 200);

// Neutral colors
const GRAY_900: Color32 = Color32::from_rgb(30, 30, 30);
const GRAY_700: Color32 = Color32::from_rgb(80, 80, 80);
const GRAY_500: Color32 = Color32::from_rgb(140, 140, 140);
const GRAY_300: Color32 = Color32::from_rgb(200, 200, 200);
const GRAY_100: Color32 = Color32::from_rgb(240, 240, 240);

// Background colors
const BG_PRIMARY: Color32 = Color32::from_rgb(232, 244, 248);
const BG_SECONDARY: Color32 = Color32::from_rgb(255, 255, 255);
```

### 8.2 Typography Scale

- **Heading Large**: 48px, bold - Used for main titles
- **Heading Medium**: 32px, bold - Used for section headers
- **Heading Small**: 24px, semibold - Used for card titles
- **Body Large**: 18px, regular - Used for primary body text
- **Body Medium**: 16px, regular - Used for secondary text
- **Body Small**: 14px, regular - Used for captions
- **Caption**: 12px, regular - Used for metadata

### 8.3 Spacing Scale

```rust
pub mod spacing {
    pub const XS: f32 = 4.0;
    pub const SM: f32 = 8.0;
    pub const MD: f32 = 16.0;
    pub const LG: f32 = 24.0;
    pub const XL: f32 = 32.0;
    pub const XXL: f32 = 48.0;
}
```

### 8.4 Border Radius

```rust
pub mod radius {
    pub const SM: f32 = 4.0;   // Small elements (tags, badges)
    pub const MD: f32 = 8.0;   // Buttons, cards
    pub const LG: f32 = 12.0;  // Panels, modals
    pub const FULL: f32 = 999.0; // Circular elements
}
```

---

## 9. Responsive Design Considerations

### 9.1 Window Size Breakpoints

```rust
pub fn window_size_category(width: f32) -> WindowSize {
    match width {
        w if w < 600.0 => WindowSize::Small,
        w if w < 1000.0 => WindowSize::Medium,
        _ => WindowSize::Large,
    }
}

pub enum WindowSize {
    Small,   // Mobile-like
    Medium,  // Tablet-like
    Large,   // Desktop
}
```

### 9.2 Adaptive Layouts

- **Small windows**: Single column, reduced font sizes, smaller bowls
- **Medium windows**: Standard layout, optimal for 800-1000px width
- **Large windows**: Centered content with max width constraint

### 9.3 Minimum Window Size

- Minimum width: 600px
- Minimum height: 400px
- Recommended: 1000x700px

---

## 10. Performance Considerations

### 10.1 Rendering Optimization

1. **Minimize Repaints**: Only request repaints during animations
2. **Lazy Loading**: Load fish images on demand
3. **Texture Caching**: Cache loaded images in TextureHandle
4. **Grid Virtualization**: For journal with 100+ fish, implement virtual scrolling

### 10.2 Animation Performance

```rust
// Only repaint while animating
if state.revealing.is_animating() {
    ctx.request_repaint();
} else {
    // Let egui handle repaints on user interaction
}
```

### 10.3 Image Loading Strategy

```rust
use egui::TextureHandle;
use std::collections::HashMap;

pub struct ImageCache {
    textures: HashMap<String, TextureHandle>,
}

impl ImageCache {
    pub fn load_fish_image(
        &mut self,
        ctx: &egui::Context,
        fish_id: &str,
    ) -> Option<&TextureHandle> {
        if !self.textures.contains_key(fish_id) {
            // Load image asynchronously
            let image = load_image_from_disk(fish_id)?;
            let texture = ctx.load_texture(
                fish_id,
                image,
                egui::TextureOptions::default(),
            );
            self.textures.insert(fish_id.to_string(), texture);
        }
        self.textures.get(fish_id)
    }
}
```

---

## 11. Testing Checklist

### 11.1 Visual Tests

- [ ] All screens render correctly at 1000x700px
- [ ] All screens render correctly at minimum size (600x400px)
- [ ] Text is readable on all backgrounds
- [ ] Colors are consistent across screens
- [ ] Hover states are visible and responsive
- [ ] Animations are smooth (60fps target)

### 11.2 Interaction Tests

- [ ] All buttons are clickable and responsive
- [ ] Bowl hover states update correctly
- [ ] Journal scrolling works smoothly
- [ ] Screen transitions are instantaneous
- [ ] Continue button only appears after animation completes
- [ ] Journal button accessible from Playing screen

### 11.3 Edge Cases

- [ ] Journal displays correctly with 0 discovered fish
- [ ] Journal displays correctly with all fish discovered
- [ ] Long fish names are truncated appropriately
- [ ] Long facts wrap correctly in fact card
- [ ] Stats display correctly with 0 guesses
- [ ] Division by zero handled in accuracy calculation

---

## 12. Future Enhancements

### 12.1 Potential Features

1. **Sound Effects**: Add audio feedback for clicks, reveals, discoveries
2. **Particle Effects**: Confetti on rare fish discovery
3. **Fish Details Modal**: Click journal card to see full details
4. **Filters/Search**: Filter journal by rarity, search by name
5. **Achievements**: Special badges for milestones
6. **Dark Mode**: Alternative color scheme
7. **Customization**: Player-selectable bowl styles

### 12.2 Accessibility Improvements

1. **Keyboard Navigation**: Full keyboard support
2. **Screen Reader Support**: ARIA labels (if egui supports)
3. **High Contrast Mode**: Enhanced contrast option
4. **Font Size Scaling**: User-adjustable text size
5. **Reduced Motion**: Option to disable animations

---

## 13. Code Organization Best Practices

### 13.1 File Size Guidelines

- Keep screen modules under 400 lines
- Extract complex widgets to separate files
- Use submodules for large screens

### 13.2 Naming Conventions

- **Functions**: `snake_case` (e.g., `render_bowl`, `button_primary`)
- **Constants**: `SCREAMING_SNAKE_CASE` (e.g., `BOWL_SIZE`, `MAX_FISH`)
- **Types**: `PascalCase` (e.g., `ScreenTransition`, `FishCard`)
- **Modules**: `snake_case` (e.g., `ui/playing.rs`)

### 13.3 Documentation

- Document all public functions with doc comments
- Include usage examples for complex helpers
- Document color choices and design rationale
- Maintain this spec as source of truth

---

## Appendix A: Complete Color Reference

```rust
pub mod colors {
    use egui::Color32;
    
    // === GLOBAL COLORS ===
    pub const TITLE: Color32 = Color32::from_rgb(26, 58, 82);
    pub const SUBTITLE: Color32 = Color32::from_rgb(74, 107, 124);
    pub const MUTED: Color32 = Color32::from_rgb(140, 160, 170);
    pub const ERROR: Color32 = Color32::from_rgb(220, 50, 50);
    pub const SUCCESS: Color32 = Color32::from_rgb(50, 180, 50);
    pub const WARNING: Color32 = Color32::from_rgb(220, 180, 50);
    
    // === BOWL COLORS (Playing Screen) ===
    pub const BOWL_DEFAULT: Color32 = Color32::from_rgb(220, 235, 245);
    pub const BOWL_HOVER: Color32 = Color32::from_rgb(180, 215, 235);
    pub const BOWL_BORDER: Color32 = Color32::from_rgb(120, 180, 210);
    pub const BOWL_BORDER_HOVER: Color32 = Color32::from_rgb(80, 140, 180);
    pub const BOWL_ICON: Color32 = Color32::from_rgb(100, 100, 100);
    
    // === REVEALING SCREEN ===
    pub const FISH_NAME: Color32 = Color32::from_rgb(30, 60, 90);
    pub const FISH_IMAGE_BORDER: Color32 = Color32::from_rgb(100, 150, 200);
    pub const FACT_CARD_BG: Color32 = Color32::from_rgb(250, 250, 250);
    pub const FACT_CARD_BORDER: Color32 = Color32::from_rgb(200, 200, 200);
    pub const FACT_TITLE: Color32 = Color32::from_rgb(60, 100, 140);
    pub const FACT_TEXT: Color32 = Color32::from_rgb(80, 80, 80);
    
    // === JOURNAL SCREEN ===
    pub const STAT_TEXT: Color32 = Color32::from_rgb(60, 60, 60);
    pub const STAT_GOOD: Color32 = Color32::from_rgb(50, 180, 50);
    pub const STAT_MEDIUM: Color32 = Color32::from_rgb(220, 180, 50);
    pub const STAT_BAD: Color32 = Color32::from_rgb(220, 80, 80);
    pub const CARD_DISCOVERED_BG: Color32 = Color32::from_rgb(255, 255, 255);
    pub const CARD_DISCOVERED_BORDER: Color32 = Color32::from_rgb(180, 200, 220);
    pub const CARD_UNKNOWN_BG: Color32 = Color32::from_rgb(240, 240, 240);
    pub const CARD_UNKNOWN_BORDER: Color32 = Color32::from_rgb(200, 200, 200);
    pub const FISH_EMOJI: Color32 = Color32::from_rgb(50, 50, 50);
    pub const FISH_NAME_CARD: Color32 = Color32::from_rgb(60, 60, 60);
    pub const MYSTERY_TEXT: Color32 = Color32::from_rgb(180, 180, 180);
    
    // === BUTTONS ===
    pub const BUTTON_PRIMARY_BG: Color32 = Color32::from_rgb(60, 130, 200);
    pub const BUTTON_PRIMARY_TEXT: Color32 = Color32::WHITE;
    pub const BUTTON_SECONDARY_BG: Color32 = Color32::from_rgb(220, 220, 220);
    pub const BUTTON_SECONDARY_TEXT: Color32 = Color32::from_rgb(60, 60, 60);
    
    // === BACKGROUNDS ===
    pub const BG_PRIMARY: Color32 = Color32::from_rgb(232, 244, 248);
    pub const BG_SECONDARY: Color32 = Color32::WHITE;
}
```

---

## Appendix B: Example State Structures

```rust
// In src/state.rs

pub struct GameState {
    pub current_screen: Screen,
    pub journal: Journal,
    pub stats: GameStats,
    pub playing: PlayingState,
    pub revealing: RevealingState,
}

pub struct PlayingState {
    pub current_fish: Fish,
    pub correct_bowl: usize,
    pub wrong_guess_message: Option<String>,
    pub hover_bowl: Option<usize>,
}

pub struct RevealingState {
    pub revealed_fish: Fish,
    pub animation_time: f32,
}

pub struct Journal {
    entries: Vec<JournalEntry>,
}

pub struct JournalEntry {
    pub fish: Fish,
    pub discovered: bool,
}

pub struct Fish {
    pub id: String,
    pub name: String,
    pub emoji: String,
    pub fact: String,
    pub rarity: u8, // 1-5
}

pub struct GameStats {
    correct_guesses: u32,
    wrong_guesses: u32,
}

impl GameStats {
    pub fn total_guesses(&self) -> u32 {
        self.correct_guesses + self.wrong_guesses
    }
    
    pub fn correct_guesses(&self) -> u32 {
        self.correct_guesses
    }
    
    pub fn record_correct_guess(&mut self) {
        self.correct_guesses += 1;
    }
    
    pub fn record_wrong_guess(&mut self) {
        self.wrong_guesses += 1;
    }
}
```

---

**End of Specification**

This technical specification provides a complete blueprint for implementing the UI rendering system for "Which Bowl is the Fish In?" game using Rust and egui. Follow this spec to ensure consistency, maintainability, and excellent user experience.
