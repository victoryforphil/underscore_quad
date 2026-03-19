# underscore_quad

Low-latency UVC camera renderer with gamepad input forwarding to ExpressLRS TX.

## Overview

`underscore_quad` is a Rust application that:
- Captures video from UVC cameras with minimal latency
- Renders frames fullscreen or in windowed mode
- Forwards gamepad input to connected ExpressLRS transmitters (Stage 2 feature)

## Goals

- **Latency**: Sub-50ms end-to-end display latency
- **Reliability**: Robust handling of camera disconnects and packet loss
- **Configurability**: CLI flags and optional config files for tuning

## Getting Started

```bash
# Build (development)
cargo build

# Run with logging
RUST_LOG=info cargo run

# Release build (optimizations enabled)
cargo build --release
```

## Prebuilt GUI Artifacts

- GitHub Actions workflow: `GUI Artifacts`
- macOS downloads: `underscore_quad-<target>.app.tar.gz`
- Linux downloads: `underscore_quad-<target>.tar.gz`
- Steam Deck target: use `underscore_quad-x86_64-unknown-linux-gnu.tar.gz`
- ARM Arch target: use `underscore_quad-aarch64-unknown-linux-gnu.tar.gz`

### Steam Deck notes

- Steam Deck runs SteamOS, which is Arch-based, on `x86_64` AMD hardware.
- The `x86_64-unknown-linux-gnu` artifact is the right build for Steam Deck.
- Install by extracting the tarball in Desktop Mode, marking the binary executable if needed, and launching `underscore_quad`.

## Dependencies

### System (Linux)

- UVC-compatible camera
- For ExpressLRS: USB-to-serial adapter or SPI interface

### Rust Crates

See `Cargo.toml` for current dependencies. Notable:
- `anyhow`, `thiserror` — error handling
- `clap` — CLI parsing
- `pretty_env_logger` — logging
- `serde` — serialization (config/packets)
- `tokio` — async runtime (add as needed)

## Project Structure

```
src/
├── main.rs              # Entry point, CLI setup
├── camera/              # UVC capture module (to be implemented)
│   └── mod.rs
├── render/              # Frame rendering module
│   ├── mod.rs
│   └── window.rs        # Window management
├── input/               # Gamepad handling
│   ├── mod.rs
│   └── gamepad.rs
├── expresslrs/          # TX communication (Stage 2)
│   └── mod.rs
└── config/              # Configuration loading
    └── mod.rs
```

## Development

### Code Style

- Follow Rust API Guidelines
- Run `cargo clippy` before committing
- Format with `cargo fmt`
- Document unsafe code and performance-critical sections

### Testing

```bash
# Run all tests
cargo test

# Run single test
cargo test <test_name> -- --nocapture

# Check (fast iteration)
cargo check
```

## License

GPLv3 — see `LICENSE` for details.
