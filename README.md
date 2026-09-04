# Which Bowl is the Fish In?

A silly fish-guessing game written in Rust (because why not use a systems programming language for a toy game?).

## The Game

- Three ornate bowls appear on screen
- Click which bowl you think the fish is in (it's random)
- Wrong? Get a humorous "nope" message
- Correct? Watch an animation as the fish reveals itself with a fun fact
- Collect fish in your journal (various rarities from Common to Mythic)
- Mix of real sea creatures and fantastical beings with real/fake facts

## Tech Stack

- **Rust** - For memory-safe bowl clicking
- **egui** - Immediate-mode GUI
- Single standalone executable

## Status

🚧 **In Development** - Spec-driven design phase

## Design Documents

See the design doc for full architecture details (coming soon).

## Development

```bash
# Run in development
cargo run

# Build release
cargo build --release
```

## Contributing

This is a toy project, but contributions welcome! Especially:
- Fish images (use placeholder initially)
- Fish facts (real and hilariously fake ones)
- Humorous "wrong guess" messages
