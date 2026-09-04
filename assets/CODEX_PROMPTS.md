# Codex Image Generation Prompts

Asset creation prompts for "Which Bowl is the Fish In" game.

---

## Prompt 1: Ornate Bowl

**Purpose:** Main bowl image (reused for all 3 bowls)

**Prompt:**
```
Create a PNG image (512x512px, transparent background) of an ornate decorative bowl viewed from a slight angle. The bowl should be ceramic or porcelain with intricate patterns - think Asian-inspired designs with blue and white glazing, or Middle Eastern patterns with gold accents. The bowl should look substantial enough to hide a fish, slightly tilted toward the viewer to show depth. Empty interior. Art style: slightly whimsical but detailed, like a high-quality game asset. Clean edges for alpha channel.
```

**Output:** `assets/bowls/bowl.png`

---

## Prompt 2: Fish Collection - Real Sea Creatures (Batch 1)

**Purpose:** Realistic fish for the game library

**Prompt:**
```
Create a collection of 5 PNG images (512x512px each, transparent background) of real sea creatures, shown from the side view, cartoonish but recognizable style. Each creature should be vibrant, colorful, and fit to hide in a decorative bowl. Include:

1. Clownfish (orange with white stripes)
2. Blue Tang (bright blue with yellow tail)
3. Octopus (red/purple with tentacles)
4. Seahorse (yellow/gold, upright pose)
5. Jellyfish (translucent with purple/pink glow)

Style: Slightly stylized but clearly identifiable, friendly appearance, clean silhouettes. Similar art style across all fish.
```

**Outputs:** 
- `assets/fish/clownfish_01.png`
- `assets/fish/blue_tang_01.png`
- `assets/fish/octopus_01.png`
- `assets/fish/seahorse_01.png`
- `assets/fish/jellyfish_01.png`

---

## Prompt 3: Fish Collection - More Real Creatures (Batch 2)

**Purpose:** Additional realistic sea creatures

**Prompt:**
```
Create 5 more PNG images (512x512px, transparent background) of sea creatures, matching the style from Batch 1. Include:

1. Angelfish (yellow and white striped)
2. Mantis Shrimp (green/orange, alien-looking)
3. Pufferfish (round, spiky, yellow)
4. Eel (long, serpentine, dark blue with spots)
5. Starfish (orange/red, five-pointed)

Style: Same as Batch 1 - cartoonish but recognizable, vibrant colors, friendly.
```

**Outputs:**
- `assets/fish/angelfish_01.png`
- `assets/fish/mantis_shrimp_01.png`
- `assets/fish/pufferfish_01.png`
- `assets/fish/eel_01.png`
- `assets/fish/starfish_01.png`

---

## Prompt 4: Fantastical Fish Collection

**Purpose:** Made-up creatures with whimsical designs

**Prompt:**
```
Create 5 PNG images (512x512px, transparent background) of FANTASTICAL sea creatures that don't exist in real life. Style matches previous batches but these are inventions:

1. Galaxy Fish - Small fish with a cosmic nebula pattern, glowing stars in its body, purple/blue gradient
2. Phoenix Koi - Koi fish with small flame-like fins, orange/red with golden accents, subtle fire aura
3. Crystal Shrimp - Translucent shrimp made of blue crystal/glass, faceted surfaces, magical sparkle
4. Void Whale - Tiny whale (bowl-sized) that's pure black with white constellation dots, ethereal
5. Clockwork Crab - Steampunk mechanical crab with visible brass gears, copper shell, tiny crank on back

Style: Whimsical fantasy, but still "cute" not scary. Should fit the game's lighthearted tone.
```

**Outputs:**
- `assets/fish/galaxy_fish_01.png`
- `assets/fish/phoenix_koi_01.png`
- `assets/fish/crystal_shrimp_01.png`
- `assets/fish/void_whale_01.png`
- `assets/fish/clockwork_crab_01.png`

---

## Prompt 5: More Fantastical Creatures

**Purpose:** Additional made-up creatures for variety

**Prompt:**
```
Create 5 more fantastical sea creatures (512x512px PNG, transparent background), matching previous style:

1. Rainbow Squid - Squid with tentacles that fade through rainbow colors, bioluminescent spots
2. Cloud Pufferfish - Pufferfish made of fluffy white cloud material, tiny lightning bolts
3. Temporal Tetra - Small fish with a clock face pattern on its side, ticking hands visible
4. Origami Octopus - Octopus that looks like it's folded from paper, crisp edges, pastel colors
5. Plasma Eel - Eel made of glowing electric plasma, neon blue/purple energy tendrils

Style: Fantasy but friendly, slightly magical/sci-fi elements, whimsical not creepy.
```

**Outputs:**
- `assets/fish/rainbow_squid_01.png`
- `assets/fish/cloud_pufferfish_01.png`
- `assets/fish/temporal_tetra_01.png`
- `assets/fish/origami_octopus_01.png`
- `assets/fish/plasma_eel_01.png`

---

## Prompt 6: UI Background

**Purpose:** Subtle background for the game

**Prompt:**
```
Create a PNG image (1024x768px) for a game background. Subtle, calming underwater scene - think blurred bokeh of blue-green water with gentle light rays from above, very soft and out of focus. Should not distract from UI elements. Color palette: muted blues, teals, subtle greens. Could include very faint suggestions of bubbles or light caustics on the "sea floor". Peaceful, zen-like atmosphere. This is the canvas behind the bowls and UI.
```

**Output:** `assets/background.png`

---

## Prompt 7: UI Effects - Success Celebration

**Purpose:** Particle effects for correct guess

**Prompt:**
```
Create a sprite sheet (1024x1024 PNG, transparent background) of celebratory particle effects for when the player guesses correctly. Include:

- Gold/yellow sparkles of various sizes
- Confetti pieces (small rectangles/triangles in festive colors)
- Star burst effects
- Small bubble particles
- Glitter/shimmer effects

Arrange as individual elements that can be scattered/animated. Bright, cheerful colors. Should feel rewarding without being overwhelming.
```

**Output:** `assets/effects/success_particles.png`

---

## Prompt 8: UI Elements - Buttons and Panels

**Purpose:** UI chrome for menus and dialogs

**Prompt:**
```
Create a 1024x1024 PNG sprite sheet (transparent background) with UI panel elements:

- Button background (3 states: normal, hover, pressed) - rounded rectangles with subtle gradient
- Dialog box / panel background - semi-transparent rounded rectangle with border
- Small decorative corners for panels (ornate, matching bowl theme)
- Separator lines (decorative)

Style: Matches the game's whimsical aesthetic, slight ornamental flair without being too busy. Colors: neutral tans/browns/creams with gold accents, similar to the bowl palette.
```

**Output:** `assets/ui/ui_elements.png`

---

## Prompt 9: Icon Set

**Purpose:** Small icons for UI (journal, settings, etc.)

**Prompt:**
```
Create a 512x512 PNG sprite sheet with small icons (64x64 each) on transparent background:

- Book icon (for Journal)
- Star icon (for rarity indicator)
- Checkmark icon (for discovered)
- Question mark icon (for undiscovered)
- Gear icon (for settings, if needed)
- Home icon (for back/return)

Style: Simple, clean line art with slight color fill, matches game aesthetic. Clear silhouettes.
```

**Output:** `assets/ui/icons.png`

---

## Notes:

- All images should have **transparent backgrounds** (PNG with alpha channel)
- **512x512px** for bowls and fish (will be scaled down in-game)
- **Consistent art style** across all assets
- **Clean edges** for proper alpha blending
- Save fish images with the naming convention: `{fish_id}.png` matching the IDs in `fish_library.toml`

## Priority Order:

1. **Bowl** (Prompt 1) - Needed immediately
2. **Fish Batch 1** (Prompt 2) - Core gameplay
3. **Fantastical Fish** (Prompt 4) - Adds variety
4. **Background** (Prompt 6) - Polish
5. **UI Elements** (Prompt 8) - For journal/menus
6. Everything else - Nice to have

You can generate these in batches as time allows!
