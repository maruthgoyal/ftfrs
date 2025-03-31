# FTFRS Development Guidelines

## Build & Test Commands
- Build: `cargo build`
- Run: `cargo run`
- Test all: `cargo test`
- Test single: `cargo test test_name`
- Lint: `cargo clippy -- -D warnings`
- Format: `cargo fmt`

## Code Style
- Use `thiserror` for error enums with `#[derive(Error)]` and `#[error("message")]`
- Use `anyhow::Result` for functions that can fail
- Implement `TryFrom` for type conversions that can fail
- Follow Rust naming: snake_case for variables/functions, CamelCase for types
- Document public interfaces with `///` doc comments
- Use the `extract_bits!` macro for bit manipulation
- Error handling: no panics in production code (replace current panic in lib.rs)
- Create specialized structs for different record types
- Keep error definitions close to the types they validate