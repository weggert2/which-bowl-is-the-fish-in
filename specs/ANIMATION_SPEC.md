# Animation System Technical Specification

## Overview

This document specifies the animation system for the "Which Bowl is the Fish In?" game. The system handles smooth, timed animations for the fish reveal sequence using easing functions and interpolation utilities.

## Module Structure

```
src/
└── animation/
    ├── mod.rs          # Core animation types and phase management
    ├── easing.rs       # Easing functions for smooth motion
    └── lerp.rs         # Linear interpolation utilities
```

## 1. Core Animation Module (`animation/mod.rs`)

### 1.1 Animation Phases

The reveal animation consists of three distinct phases:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationPhase {
    FishEntry,    // 0.8 seconds - fish moves from bowl to center
    ShowFact,     // 3.0 seconds - fact fades in and displays
    FishExit,     // 1.0 seconds - fish fades out and exits
    Complete,     // Animation finished
}
```

### 1.2 Phase Duration Constants

```rust
/// Duration constants for each animation phase (in seconds)
pub mod durations {
    pub const FISH_ENTRY: f64 = 0.8;
    pub const SHOW_FACT: f64 = 3.0;
    pub const FISH_EXIT: f64 = 1.0;
    pub const TOTAL: f64 = FISH_ENTRY + SHOW_FACT + FISH_EXIT; // 4.8s
}
```

### 1.3 RevealAnimation Struct

The main animation controller that tracks timing and phase progression:

```rust
use std::time::Instant;

pub struct RevealAnimation {
    /// When the animation started
    start_time: Instant,
    
    /// Current phase of the animation
    current_phase: AnimationPhase,
    
    /// Time elapsed within the current phase (in seconds)
    phase_elapsed: f64,
}

impl RevealAnimation {
    /// Create a new animation starting now
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            current_phase: AnimationPhase::FishEntry,
            phase_elapsed: 0.0,
        }
    }
    
    /// Update animation state based on elapsed time
    /// Returns true if animation state changed (requires repaint)
    pub fn update(&mut self) -> bool {
        let total_elapsed = self.start_time.elapsed().as_secs_f64();
        let prev_phase = self.current_phase;
        
        // Determine current phase based on total elapsed time
        if total_elapsed < durations::FISH_ENTRY {
            self.current_phase = AnimationPhase::FishEntry;
            self.phase_elapsed = total_elapsed;
        } else if total_elapsed < durations::FISH_ENTRY + durations::SHOW_FACT {
            self.current_phase = AnimationPhase::ShowFact;
            self.phase_elapsed = total_elapsed - durations::FISH_ENTRY;
        } else if total_elapsed < durations::TOTAL {
            self.current_phase = AnimationPhase::FishExit;
            self.phase_elapsed = total_elapsed - durations::FISH_ENTRY - durations::SHOW_FACT;
        } else {
            self.current_phase = AnimationPhase::Complete;
            self.phase_elapsed = 0.0;
        }
        
        // Return true if phase changed or animation still in progress
        prev_phase != self.current_phase || !self.is_complete()
    }
    
    /// Get progress through current phase (0.0 to 1.0)
    pub fn phase_progress(&self) -> f64 {
        let duration = match self.current_phase {
            AnimationPhase::FishEntry => durations::FISH_ENTRY,
            AnimationPhase::ShowFact => durations::SHOW_FACT,
            AnimationPhase::FishExit => durations::FISH_EXIT,
            AnimationPhase::Complete => return 1.0,
        };
        
        (self.phase_elapsed / duration).min(1.0)
    }
    
    /// Get current phase
    pub fn current_phase(&self) -> AnimationPhase {
        self.current_phase
    }
    
    /// Check if animation is complete
    pub fn is_complete(&self) -> bool {
        self.current_phase == AnimationPhase::Complete
    }
    
    /// Get total elapsed time in seconds
    pub fn total_elapsed(&self) -> f64 {
        self.start_time.elapsed().as_secs_f64()
    }
}
```

### 1.4 Animation Parameters

Constants for visual positioning and scaling:

```rust
pub mod params {
    use egui::Pos2;
    
    /// Screen center position (relative to viewport)
    pub const CENTER_SCREEN: Pos2 = Pos2 { x: 0.5, y: 0.5 };
    
    /// Fish scale during entry animation
    pub const FISH_SCALE_START: f64 = 0.5;   // 50% size at bowl
    pub const FISH_SCALE_END: f64 = 1.5;     // 150% size at center
    
    /// Alpha values for fade effects
    pub const ALPHA_TRANSPARENT: u8 = 0;
    pub const ALPHA_OPAQUE: u8 = 255;
    
    /// Fact text fade-in timing
    pub const FACT_FADE_START: f64 = 0.2;    // Start fading at 20% of ShowFact phase
    pub const FACT_FADE_END: f64 = 0.5;      // Fully visible at 50% of ShowFact phase
}
```

## 2. Easing Functions (`animation/easing.rs`)

Easing functions transform linear progress (0.0 to 1.0) into smooth, natural motion curves.

### 2.1 Mathematical Formulas

```rust
/// Cubic ease-out: fast start, slow end
/// Formula: 1 - (1 - t)³
/// Use for: Fish entry (smooth deceleration into center)
pub fn ease_out_cubic(t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(3)
}

/// Cubic ease-in: slow start, fast end  
/// Formula: t³
/// Use for: Fish exit (smooth acceleration away)
pub fn ease_in_cubic(t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    t.powi(3)
}

/// Quadratic ease-in-out: smooth both ends
/// Formula: t < 0.5 ? 2t² : 1 - (-2t + 2)²/2
/// Use for: Fade effects (subtle alpha transitions)
pub fn ease_in_out_quad(t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    if t < 0.5 {
        2.0 * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
    }
}

/// Linear (no easing): constant speed
/// Formula: t
/// Use for: Debug or when constant speed is desired
pub fn linear(t: f64) -> f64 {
    t.clamp(0.0, 1.0)
}
```

### 2.2 Easing Function Selection Guide

| Phase | Property | Easing Function | Rationale |
|-------|----------|----------------|-----------|
| FishEntry | Position | `ease_out_cubic` | Decelerates smoothly into center |
| FishEntry | Scale | `ease_out_cubic` | Synchronized with position |
| ShowFact | Alpha (text) | `ease_in_out_quad` | Gentle fade-in |
| FishExit | Position | `ease_in_cubic` | Accelerates away from center |
| FishExit | Alpha (fish) | `ease_in_cubic` | Synchronized with exit motion |

### 2.3 Visual Reference

```
ease_out_cubic (Entry):
Progress: 0% ──────────────────────── 100%
Position: •━━━━━━━━━━━━━━━━━━━━━━━━━━━○
          ↑                            ↑
        Fast                        Slow

ease_in_cubic (Exit):
Progress: 0% ──────────────────────── 100%
Position: ○━━━━━━━━━━━━━━━━━━━━━━━━━━━•
          ↑                            ↑
        Slow                        Fast

ease_in_out_quad (Fade):
Progress: 0% ──────────────────────── 100%
Alpha:    ○━━━━━━━━━━━━━━━━━━━━━━━━━━○
          ↑                            ↑
        Slow        Fast          Slow
```

## 3. Interpolation Utilities (`animation/lerp.rs`)

Linear interpolation (lerp) functions for smooth value transitions.

### 3.1 Core Lerp Function

```rust
/// Linear interpolation between two f64 values
/// Formula: start + (end - start) * t
/// 
/// # Arguments
/// * `start` - Starting value (at t=0.0)
/// * `end` - Ending value (at t=1.0)
/// * `t` - Progress from 0.0 to 1.0
pub fn lerp(start: f64, end: f64, t: f64) -> f64 {
    start + (end - start) * t
}
```

### 3.2 Vec2/Pos2 Interpolation

```rust
use egui::Pos2;

/// Interpolate between two 2D positions
/// 
/// # Example
/// ```
/// let start = Pos2::new(100.0, 200.0);
/// let end = Pos2::new(400.0, 300.0);
/// let mid = lerp_pos2(start, end, 0.5);
/// // mid = Pos2 { x: 250.0, y: 250.0 }
/// ```
pub fn lerp_pos2(start: Pos2, end: Pos2, t: f64) -> Pos2 {
    Pos2 {
        x: lerp(start.x as f64, end.x as f64, t) as f32,
        y: lerp(start.y as f64, end.y as f64, t) as f32,
    }
}
```

### 3.3 Color Interpolation

```rust
use egui::Color32;

/// Interpolate between two colors in RGBA space
/// 
/// # Example
/// ```
/// let transparent = Color32::from_rgba_premultiplied(255, 255, 255, 0);
/// let opaque = Color32::from_rgba_premultiplied(255, 255, 255, 255);
/// let half = lerp_color(transparent, opaque, 0.5);
/// // half = Color32 { r: 255, g: 255, b: 255, a: 127 }
/// ```
pub fn lerp_color(start: Color32, end: Color32, t: f64) -> Color32 {
    Color32::from_rgba_premultiplied(
        lerp_u8(start.r(), end.r(), t),
        lerp_u8(start.g(), end.g(), t),
        lerp_u8(start.b(), end.b(), t),
        lerp_u8(start.a(), end.a(), t),
    )
}

/// Helper: interpolate between two u8 values
fn lerp_u8(start: u8, end: u8, t: f64) -> u8 {
    lerp(start as f64, end as f64, t).round() as u8
}
```

### 3.4 Alpha Channel Utilities

```rust
/// Apply alpha to a color, preserving RGB
/// 
/// # Arguments
/// * `color` - Base color
/// * `alpha` - Alpha value 0-255 (or 0.0-1.0 normalized)
pub fn with_alpha(color: Color32, alpha: u8) -> Color32 {
    Color32::from_rgba_premultiplied(
        color.r(),
        color.g(),
        color.b(),
        alpha,
    )
}

/// Normalize alpha from 0.0-1.0 to 0-255
pub fn alpha_to_u8(alpha: f64) -> u8 {
    (alpha.clamp(0.0, 1.0) * 255.0).round() as u8
}
```

## 4. Animation Phase Implementation

### 4.1 Fish Entry Phase

**Duration:** 0.8 seconds  
**Purpose:** Animate fish from bowl position to screen center with scale-up

```rust
use crate::animation::{RevealAnimation, AnimationPhase, easing, lerp, params};
use egui::{Pos2, Response, Ui};

fn render_fish_entry(
    ui: &mut Ui,
    animation: &RevealAnimation,
    bowl_pos: Pos2,
    fish_image: &egui::TextureHandle,
) {
    if animation.current_phase() != AnimationPhase::FishEntry {
        return;
    }
    
    // Get linear progress and apply easing
    let progress = animation.phase_progress();
    let eased_progress = easing::ease_out_cubic(progress);
    
    // Interpolate position from bowl to screen center
    let screen_center = ui.ctx().screen_rect().center();
    let current_pos = lerp::lerp_pos2(bowl_pos, screen_center, eased_progress);
    
    // Interpolate scale
    let scale = lerp::lerp(
        params::FISH_SCALE_START,
        params::FISH_SCALE_END,
        eased_progress
    );
    
    // Render fish at interpolated position and scale
    let fish_size = egui::vec2(
        fish_image.size_vec2().x * scale as f32,
        fish_image.size_vec2().y * scale as f32,
    );
    
    let fish_rect = egui::Rect::from_center_size(current_pos, fish_size);
    ui.painter().image(
        fish_image.id(),
        fish_rect,
        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
        egui::Color32::WHITE,
    );
}
```

### 4.2 Show Fact Phase

**Duration:** 3.0 seconds  
**Purpose:** Display fact text with fade-in, hold for reading

```rust
fn render_show_fact(
    ui: &mut Ui,
    animation: &RevealAnimation,
    fact_text: &str,
    fish_image: &egui::TextureHandle,
) {
    if animation.current_phase() != AnimationPhase::ShowFact {
        return;
    }
    
    let progress = animation.phase_progress();
    
    // Calculate fade-in alpha for fact text
    let fade_progress = if progress < params::FACT_FADE_START {
        0.0
    } else if progress < params::FACT_FADE_END {
        (progress - params::FACT_FADE_START) / (params::FACT_FADE_END - params::FACT_FADE_START)
    } else {
        1.0
    };
    
    let text_alpha = alpha_to_u8(easing::ease_in_out_quad(fade_progress));
    
    // Render fish at center (full scale, full opacity)
    let screen_center = ui.ctx().screen_rect().center();
    let fish_size = fish_image.size_vec2() * params::FISH_SCALE_END as f32;
    let fish_rect = egui::Rect::from_center_size(screen_center, fish_size);
    
    ui.painter().image(
        fish_image.id(),
        fish_rect,
        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
        egui::Color32::WHITE,
    );
    
    // Render fact text below fish with fade-in
    let text_pos = egui::pos2(
        screen_center.x,
        screen_center.y + fish_size.y * 0.6,
    );
    
    let text_color = egui::Color32::from_rgba_premultiplied(255, 255, 255, text_alpha);
    ui.painter().text(
        text_pos,
        egui::Align2::CENTER_CENTER,
        fact_text,
        egui::FontId::proportional(24.0),
        text_color,
    );
}
```

### 4.3 Fish Exit Phase

**Duration:** 1.0 seconds  
**Purpose:** Fade out and move fish away from center

```rust
fn render_fish_exit(
    ui: &mut Ui,
    animation: &RevealAnimation,
    fish_image: &egui::TextureHandle,
) {
    if animation.current_phase() != AnimationPhase::FishExit {
        return;
    }
    
    let progress = animation.phase_progress();
    let eased_progress = easing::ease_in_cubic(progress);
    
    // Fade out alpha
    let alpha = alpha_to_u8(1.0 - eased_progress);
    
    // Optional: Move fish slightly upward during exit
    let screen_center = ui.ctx().screen_rect().center();
    let exit_offset = lerp::lerp(0.0, -100.0, eased_progress);
    let current_pos = egui::pos2(screen_center.x, screen_center.y + exit_offset as f32);
    
    // Render fish with fade-out
    let fish_size = fish_image.size_vec2() * params::FISH_SCALE_END as f32;
    let fish_rect = egui::Rect::from_center_size(current_pos, fish_size);
    
    let tint_color = egui::Color32::from_rgba_premultiplied(255, 255, 255, alpha);
    ui.painter().image(
        fish_image.id(),
        fish_rect,
        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
        tint_color,
    );
}
```

## 5. Game Loop Integration

### 5.1 Animation Update Pattern

```rust
use egui::{Context, CentralPanel};

struct GameState {
    reveal_animation: Option<RevealAnimation>,
    // ... other game state
}

impl GameState {
    fn update(&mut self, ctx: &Context) {
        // Update animation if active
        if let Some(animation) = &mut self.reveal_animation {
            let needs_repaint = animation.update();
            
            if animation.is_complete() {
                // Clean up completed animation
                self.reveal_animation = None;
            } else if needs_repaint {
                // Request repaint for next frame
                ctx.request_repaint();
            }
        }
        
        // Main UI rendering
        CentralPanel::default().show(ctx, |ui| {
            self.render_game(ui);
            
            // Render animation overlay if active
            if let Some(animation) = &self.reveal_animation {
                self.render_animation_overlay(ui, animation);
            }
        });
    }
    
    fn render_animation_overlay(&self, ui: &mut Ui, animation: &RevealAnimation) {
        match animation.current_phase() {
            AnimationPhase::FishEntry => {
                render_fish_entry(ui, animation, self.selected_bowl_pos, &self.fish_image);
            }
            AnimationPhase::ShowFact => {
                render_show_fact(ui, animation, &self.current_fact, &self.fish_image);
            }
            AnimationPhase::FishExit => {
                render_fish_exit(ui, animation);
            }
            AnimationPhase::Complete => {}
        }
    }
    
    fn start_reveal_animation(&mut self) {
        self.reveal_animation = Some(RevealAnimation::new());
    }
}
```

### 5.2 Frame Rate Considerations

```rust
// In your main.rs eframe setup
let options = eframe::NativeOptions {
    // Limit frame rate during animations to save CPU
    vsync: true,
    ..Default::default()
};

// Request repaint in update loop only when animation is active
// egui will automatically throttle based on vsync
```

### 5.3 Animation Lifecycle

```
User clicks bowl
       ↓
Start RevealAnimation::new()
       ↓
┌──────────────────────────────────┐
│  Game Loop (60 FPS)              │
│  ├─ animation.update()           │
│  ├─ Check phase                  │
│  ├─ Render current phase         │
│  └─ ctx.request_repaint()        │
└──────────────────────────────────┘
       ↓
Animation completes (4.8s elapsed)
       ↓
animation.is_complete() == true
       ↓
Clean up: self.reveal_animation = None
```

## 6. Testing and Debugging

### 6.1 Debug Visualization

```rust
impl RevealAnimation {
    /// Render debug overlay showing animation state
    #[cfg(debug_assertions)]
    pub fn debug_render(&self, ui: &mut Ui) {
        egui::Window::new("Animation Debug")
            .default_pos(egui::pos2(10.0, 10.0))
            .show(ui.ctx(), |ui| {
                ui.label(format!("Phase: {:?}", self.current_phase));
                ui.label(format!("Phase Progress: {:.2}%", self.phase_progress() * 100.0));
                ui.label(format!("Total Elapsed: {:.2}s", self.total_elapsed()));
                
                // Progress bar for current phase
                ui.add(egui::ProgressBar::new(self.phase_progress() as f32)
                    .text(format!("{:.0}%", self.phase_progress() * 100.0)));
            });
    }
}
```

### 6.2 Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ease_out_cubic_bounds() {
        assert_eq!(easing::ease_out_cubic(0.0), 0.0);
        assert_eq!(easing::ease_out_cubic(1.0), 1.0);
        assert!(easing::ease_out_cubic(0.5) > 0.5); // Should be ahead at midpoint
    }
    
    #[test]
    fn test_lerp_midpoint() {
        assert_eq!(lerp::lerp(0.0, 100.0, 0.5), 50.0);
        assert_eq!(lerp::lerp(-50.0, 50.0, 0.5), 0.0);
    }
    
    #[test]
    fn test_lerp_color() {
        let black = Color32::BLACK;
        let white = Color32::WHITE;
        let gray = lerp::lerp_color(black, white, 0.5);
        
        assert_eq!(gray.r(), 127);
        assert_eq!(gray.g(), 127);
        assert_eq!(gray.b(), 127);
    }
    
    #[test]
    fn test_animation_phase_progression() {
        let mut anim = RevealAnimation::new();
        
        // Mock time advancement (in real code, use Instant::now())
        // This test would need a way to inject time for testing
        assert_eq!(anim.current_phase(), AnimationPhase::FishEntry);
    }
}
```

### 6.3 Manual Test Cases

1. **Smooth Entry**: Fish should decelerate smoothly into center, not pop or jerk
2. **Readable Fact**: Fact text should be visible for full 3 seconds after fade-in
3. **Clean Exit**: Fish should accelerate away and fade simultaneously
4. **No Flicker**: No frame drops or visual glitches during transitions
5. **Timing Accuracy**: Total animation should complete in ~4.8 seconds

## 7. Performance Considerations

### 7.1 Optimization Guidelines

- Use `Instant` for high-precision timing (already nanosecond-accurate)
- Clamp all interpolation values to prevent overshooting
- Request repaint only when animation is active
- Avoid allocations in hot path (render loop)
- Pre-load fish images and textures before starting animation

### 7.2 Memory Footprint

```rust
// RevealAnimation is small and cheap to create
std::mem::size_of::<RevealAnimation>() // ~24 bytes
std::mem::size_of::<AnimationPhase>()   // 1 byte (enum discriminant)
```

## 8. Future Enhancements

### 8.1 Possible Extensions

- **Configurable Durations**: Make phase durations adjustable via config
- **Custom Easing**: Add bezier curve easing for more complex motion
- **Animation Events**: Callbacks at phase transitions (e.g., play sound effects)
- **Particle Effects**: Add splash/bubble particles during entry/exit
- **Keyframe System**: Support arbitrary keyframe animations beyond 3 phases

### 8.2 Advanced Features

```rust
// Example: Animation event system
pub enum AnimationEvent {
    PhaseStarted(AnimationPhase),
    PhaseEnded(AnimationPhase),
    ProgressMilestone(f64),
}

impl RevealAnimation {
    pub fn poll_events(&mut self) -> Vec<AnimationEvent> {
        // Return events that occurred since last poll
        // Useful for triggering sound effects, particle spawns, etc.
    }
}
```

## 9. Dependencies

Required crates (add to `Cargo.toml`):

```toml
[dependencies]
egui = "0.29"
eframe = "0.29"  # For running the application
```

## 10. File Checklist

Implementation checklist:

- [ ] `src/animation/mod.rs` - Core types, RevealAnimation, constants
- [ ] `src/animation/easing.rs` - Easing functions (4 functions)
- [ ] `src/animation/lerp.rs` - Interpolation utilities (4 functions)
- [ ] Integration in `src/main.rs` or `src/game.rs`
- [ ] Unit tests for easing and lerp
- [ ] Manual testing of full animation sequence

## Appendix A: Complete Module Template

```rust
// src/animation/mod.rs
pub mod easing;
pub mod lerp;

use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationPhase {
    FishEntry,
    ShowFact,
    FishExit,
    Complete,
}

pub mod durations {
    pub const FISH_ENTRY: f64 = 0.8;
    pub const SHOW_FACT: f64 = 3.0;
    pub const FISH_EXIT: f64 = 1.0;
    pub const TOTAL: f64 = FISH_ENTRY + SHOW_FACT + FISH_EXIT;
}

pub mod params {
    pub const FISH_SCALE_START: f64 = 0.5;
    pub const FISH_SCALE_END: f64 = 1.5;
    pub const ALPHA_TRANSPARENT: u8 = 0;
    pub const ALPHA_OPAQUE: u8 = 255;
    pub const FACT_FADE_START: f64 = 0.2;
    pub const FACT_FADE_END: f64 = 0.5;
}

pub struct RevealAnimation {
    start_time: Instant,
    current_phase: AnimationPhase,
    phase_elapsed: f64,
}

// ... implementation from section 1.3
```

---

**Document Version:** 1.0  
**Last Updated:** 2026-09-04  
**Author:** Technical Specification for Which Bowl is the Fish In?
