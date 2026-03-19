# underscore_quad System Agent Profile

This profile is for system-level agents operating in this repository.

## Repository Overview

- **Name**: `underscore_quad`
- **Type**: Low-latency Rust application for UVC camera rendering + gamepad input forwarding
- **Primary Language**: Rust (edition 2021)
- **License**: GPLv3

## Build & Deployment

```bash
# Development build
cargo build

# Release (optimizations enabled)
cargo build --release

# Fast iteration
cargo check [--release]

# Linting (required before commit)
cargo clippy -- -D warnings

# Testing
cargo test [test_name]
```

## Code Style Reference

See `STYLE.md` for detailed Rust style guidelines. Key points:

- Use `thiserror` for domain errors, `anyhow` for contextual errors
- Prefer slices (`&[u8]`) over owned vectors in frame processing hot paths
- Document all unsafe code with safety invariants
- Log at appropriate levels using `pretty_env_logger`
- Run `cargo fmt` and `cargo clippy` before committing

## Project Architecture

### Current State (v0.1)

- Basic Cargo skeleton
- Dependencies: anyhow, clap, pretty_env_logger, serde, thiserror
- No implementation yet (main.rs just prints "Hello, world!")

### Planned Modules

| Module | Purpose |
|--------|---------|
| camera/ | UVC video capture with minimal latency |
| render/ | Fullscreen/windowed frame presentation |
| input/ | Gamepad input polling and mapping |
| expresslrs/ | TX communication (Stage 2) |

## Common Tasks

### Adding a New Dependency

1. Edit `Cargo.toml`
2. Add to imports in relevant module
3. Run `cargo update && cargo check`

### Adding a New Module

1. Create `src/<module_name>/mod.rs` with docs
2. Add `pub mod <module_name>;` in `src/main.rs`

## Related Files

| File | Purpose |
|------|---------|
| Cargo.toml | Dependencies and metadata |
| AGENTS.md | General agent guide |
| STYLE.md | Detailed coding style guidelines |
| README.md | Project overview |

