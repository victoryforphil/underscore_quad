# underscore_quad Style Guide

## Rust Code Style

### General Conventions

- **Edition**: 2021 (not 2024 which doesn't exist yet)
- **Line Length**: 100 characters (max)
- **Braces**: Open on same line as statement (K&R style)

```rust
if condition {
    // ...
} else {
    // ...
}
```

### Error Handling

Use `thiserror` for domain-specific errors:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CameraError {
    #[error("camera not found: {0}")]
    NotFound(u32),
    
    #[error("failed to open camera: {source}")]
    OpenFailed { source: anyhow::Error },
}

// Usage:
fn connect_camera(id: u32) -> Result<Camera, CameraError> {
    // ...
}
```

For one-off or contextual errors, use `anyhow!()`:

```rust
bail!("camera {} not responding", id);
```

### Naming

| Type | Style | Example |
|------|-------|---------|
| Struct/Enum | PascalCase | `UvcFrame`, `RenderBackend` |
| Function/Method | snake_case | `process_frame()`, `init_window()` |
| Constant | SCREAMING_SNAKE_CASE | `MAX_LATENCY_MS` |
| Lifetime | `'a`, `'b` (short), descriptive if needed (`'frame`) |
| Module | snake_case | `camera`, `render`, `input` |

### Imports

Group and sort alphabetically:

```rust
// Standard library
use std::collections::HashMap;
use std::io::{self, Write};
use std::time::Duration;

// External crates
use anyhow::Result;
use clap::Parser;

// Crate modules
use crate::camera::{Camera, Frame};
```

### Comments & Documentation

- Use `///` for module/item docs (not `//!`)
- Explain **why** more than **what**
- Document safety invariants for unsafe code:

```rust
/// Safety: `ptr` must be valid for reads of `len` bytes until dropped.
pub unsafe fn from_raw_parts(ptr: *const u8, len: usize) -> Self {
    // ...
}
```

### Performance & Memory

**Avoid allocations in hot paths:**

```rust
// ✅ Prefer slice over cloning
fn process(frame: &[u8]) { ... }

// ❌ Avoid if not needed
fn process(frame: Vec<u8>) { ... }
```

**Prefer stack allocation when size is bounded:**

```rust
use arrayvec::ArrayVec;

// For small, fixed max sizes
let buffer: ArrayVec<u8, 1024> = ArrayVec::new();
```

### Async/Futures

- Use `tokio` for I/O-bound tasks (camera capture, network)
- Keep CPU-heavy work in blocking threads or sync code
- For channels: use `crossbeam-channel` for high-throughput or `std::sync::mpsc`

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let (tx, rx) = tokio::sync::mpsc::channel(32);
    
    // Producer (blocking camera read in async context)
    tokio::spawn(async move {
        // ...
    });
    
    // Consumer
    while let Some(frame) = rx.recv().await {
        render(&frame)?;
    }
    
    Ok(())
}
```

## Project Structure

### Module Organization

Place related functionality in submodules:

```rust
// src/camera/mod.rs
pub mod capture;
pub mod decode;

// src/render/mod.rs  
pub mod window;
pub mod presenter;
```

### Configuration

Use `serde` + `clap` for config:

```rust
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "underscore_quad")]
struct Args {
    #[arg(short, long, default_value = "1920x1080")]
    resolution: String,
    
    #[arg(short, long, default_value_t = 60.0)]
    fps: f32,
    
    #[arg(short, long)]
    fullscreen: bool,
}
```

### Logging

Use `tracing` or `log`:

```rust
use pretty_env_logger;
use tracing::{info, debug};

fn init_logging() {
    pretty_env_logger::init();
}

fn process_frame(frame: &[u8]) -> Result<()> {
    debug!("processing {} bytes", frame.len());
    // ...
}
```

## File Headers

Each source file should have:

```rust
//! Module documentation
//!
//! Details about what this module does.

// Lint suppressions (if needed)
// #![deny(clippy::all)]

use crate::prelude::*;
```

## Git & PR Guidelines

### Commit Messages

```
scope: brief description

Detailed explanation if needed.
Fixes #123.
```

Scopes: `camera`, `render`, `input`, `expresslrs`, `config`, `docs`, `ci`

### Branch Names

- `feature/camera-capture` — new feature
- `fix/render-fps` — bug fix
- `refactor/frame-pipeline` — refactoring
- `docs/readme-update` — docs

## Common Pitfalls

1. **Memory**: Avoid copying large frame buffers; use slices/arc
2. **Blocking**: Don't block the main thread with I/O
3. **Errors**: Don't swallow errors silently; log and propagate
4. **Safety**: Document all `unsafe` usage thoroughly
5. **Latency**: Measure before optimizing; focus on hot paths

## Code Review Checklist

- [ ] Clippy passes (`cargo clippy -- -D warnings`)
- [ ] Tests pass (`cargo test`)
- [ ] Performance doesn't regress (measure if non-trivial change)
- [ ] Unsafe code is documented and minimal
- [ ] Logging adds value (not noise)
- [ ] Error messages are user-actionable
