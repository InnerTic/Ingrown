# Ingrown

A language-neutral AI agent runtime built around capabilities.

## Current Status

Initial Rust skeleton - establishing load-bearing walls.

## Structure

- `crates/ingrown-api/` - Core capability abstractions
- `crates/ingrown-core/` - Agent runtime and registry

## Building

```bash
cargo build
cargo test
```

## Running

```bash
cargo run -p ingrown-core
```
