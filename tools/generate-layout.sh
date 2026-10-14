#!/bin/sh
# Regenerates the launcher app's copy of the default touch-control layout from the engine's own
# table (`src/OpenGMK/gm8emulator/src/touch_controls.rs`), so the editor preview and the game can
# never disagree about where a button sits by default.
set -e
root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root/src/OpenGMK"
mkdir -p "$root/android/app/src/main/assets"
cargo run --quiet --release -p gm8emulator --example dump_layout > "$root/android/app/src/main/assets/controls-layout.json"
echo "Escrito android/app/src/main/assets/controls-layout.json"
