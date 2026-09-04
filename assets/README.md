# Assets Directory

## Structure

```
assets/
├── bowls/          # Bowl images (ornate, decorative bowls)
│   └── bowl.png    # Main bowl image (will be used for all 3 bowls)
├── fish/           # Fish/sea creature images
│   ├── clownfish.png
│   ├── octopus.png
│   └── ...
└── fish_library.toml  # Fish data with facts (generated)
```

## Image Specifications

### Bowl Images
- **Format:** PNG with alpha channel
- **Size:** 512x512px (will be scaled down in-game)
- **Style:** Ornate, decorative bowls
- One image will be reused for all three bowls

### Fish Images
- **Format:** PNG with alpha channel
- **Size:** 512x512px (will be scaled to fit)
- **Naming:** Use fish ID from fish_library.toml (e.g., `clownfish_01.png`)
- **Mix:** Real sea creatures and fantastical beings

## Placeholders

The game will use colored rectangles as placeholders until actual images are added.
Images can be added incrementally - the game will fall back to placeholders for missing images.

## Contributing Images

Use your codex image generator to create:
1. Bowl images (decorative, game-appropriate)
2. Fish images (both realistic and fantastical)

Name fish images to match their ID in `fish_library.toml`.
