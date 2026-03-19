# underscore_quad Agent Guide

## Build & Test Commands

```bash
# Build (debug)
cargo build

# Build (release with optimizations for low-latency)
cargo build --release

# Run with logging enabled
RUST_LOG=info cargo run [--release]

# Run a single test
cargo test <test_name> -- --nocapture

# Lint with clippy (recommended before commit)
cargo clippy -- -D warnings

# Format code
cargo fmt

# Check without building (fast iteration)
cargo check [--release]
```

For running benchmarks or profiling, use `cargo bench` and `flamegraph` or `perf`.

## Code Style Guidelines

### Rust Conventions (1.75+)

- **Edition**: Use edition 2021 (not 2024 which doesn't exist yet)
- **Error Handling**:
  - Return `Result<T, E>` with custom error types using `thiserror`
  - Avoid `expect()` in production code; use `?` operator
  - For unrecoverable errors: `panic!()` only in tests/dev; use `bail!()` from anyhow elsewhere
- **Logging**: Use `tracing` (preferred) or `log` with `pretty_env_logger`; log at appropriate levels (trace/debug/info/warn/error)
- **Naming**:
  - Types/Structs: PascalCase (`UvcCamera`, `GamepadInput`)
  - Functions/Methods: snake_case(`process_frame()`, `connect_tx()`)
  - Constants: SCREAMING_SNAKE_CASE
  - Lifetimes: short, descriptive (`'a`, `'frame`)
- **Imports**: Group std > external crates > crate modules; each group sorted alphabetically

### Low-Latency Best Practices

- Use `&[u8]` slices instead of owned `Vec<u8>` where possible to avoid allocations
- Prefer zero-copy architectures for frame data passing
- Consider `std::sync::mpsc` or `crossbeam-channel` for high-throughput pipelines
- Use `#[inline(always)]` sparingly—only after profiling shows benefit
- Avoid heap allocations in hot paths (frame processing loop)
- For lock-free rings, consider `arrayvec` or `slab` for preallocated buffers

### Architecture Guidelines

- **Core Pipeline**:
  ```
  Camera Capture → Decode → Preprocess → Render → Output
  ```
  Keep stages separated; use channels between async blocks

- **Async/Futures**: Use `tokio` for I/O (camera, network to ExpressLRS); keep CPU-bound work in blocking threads

- **Memory Safety**:
  - Mark unsafe code with detailed comments explaining safety invariants
  - Prefer safe abstractions over raw pointers
  - Use `std::ptr::NonNull` when null is invalid

### Frame Rendering

- Target: Use `winit` for windowing, `vulkan`, `opengl`, or `gpu-rs` for rendering (pending implementation)
- Fullscreen: Default to exclusive fullscreen for lowest latency on target hardware
- vsync: Consider disabling in release builds for minimal latency (test tradeoff)

### Gamepad/ExpressLRS Integration

- **Input Mapping**: Define a configuration struct using `serde` for remapping
- **Polling**: Use non-blocking gamepad API; buffer last N inputs
- **Protocol**: ExpressLRS uses UART/SPI; implement in separate module with retry logic
- **Latency Tracking**: Add timestamps to frames and telemetry for end-to-end latency measurement

### Testing Strategy

- Unit tests: Test individual components (frame processing, packet encoding)
- Integration tests: Test full pipeline end-to-end
- Mock I/O: Use `mockall` or custom fakes for camera/ExpressLRS in tests
- CI: Run clippy and fmt on PR; test with `--release` flags

### Configuration

- CLI args via `clap` (already in dependencies)
- Optional config file (TOML) for persistent settings
- Environment variables: `UQ_LOG_LEVEL`, `UQ_RENDER_API`, `UQ_TARGET_FPS`

### Git Workflow

- Branch naming: feature/fix/test/refactor/docs-<description>
- Commit messages: `scope: brief description` (e.g., `camera: add uvc capture loop`)
- Tag releases: `v0.x.y` for stable; `v0.x.y-alpha`/`beta` for pre-release

### Performance Targets

- End-to-end latency: <50ms from camera to display (target)
- Frame jitter: <2ms variance
- Memory usage: <256MB resident on typical hardware

## Existing Dependencies Summary

| Crate | Purpose |
|-------|---------|
| `anyhow` | Error handling with backtrace support |
| `clap` | CLI argument parsing with env var support |
| `pretty_env_logger` | Human-readable logging |
| `serde` | Serialization (likely for config/packets) |
| `thiserror` | Custom error types |

## Notes

- This is a systems project—prefer explicit over implicit behavior
- Profile before optimizing; measure impact of changes
- Document unsafe code rigorously
- Keep platform-specific code in feature-gated modules (`#[cfg(target_os = "linux")]`
