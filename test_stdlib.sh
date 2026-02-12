#!/usr/bin/env bash
# Test script to generate stdlib JSON

STDLIB_SRC="$HOME/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library"
echo "Attempting to generate stdlib JSON from: $STDLIB_SRC"

if [ -f "$STDLIB_SRC/Cargo.toml" ]; then
    echo "✓ Cargo.toml exists in stdlib source directory"
    
    # Try to generate JSON for std package
    cd "$STDLIB_SRC"
    cargo +nightly doc --package std --no-deps --lib --output-format json 2>&1 | grep -E "json|error|warning" | head -20
else
    echo "✗ Cargo.toml not found in stdlib source directory"
fi
