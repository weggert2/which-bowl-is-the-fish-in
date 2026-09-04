#!/bin/bash
cd /home/bill/other/which-bowl
cargo build --quiet && cat <<'RUST' | cargo run --quiet --example test_fish 2>&1
// This would be a test program
RUST
