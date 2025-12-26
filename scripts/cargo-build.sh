#!/bin/sh
# Create project structure
mkdir -p src
mkdir -p examples
mkdir -p man


# Build
cargo build --release

# The executable will be at: target/release/rax25kb

