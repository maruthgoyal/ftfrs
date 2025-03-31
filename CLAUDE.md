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
- Use custom `Result<T>` type for functions that can fail (we've replaced anyhow::Result)
- Implement `TryFrom` for type conversions that can fail
- Follow Rust naming: snake_case for variables/functions, CamelCase for types
- Document public interfaces with `///` doc comments
- Use the `extract_bits!` macro for bit manipulation
- Error handling: no panics in production code
- Create specialized structs for different record types
- Keep error definitions close to the types they validate

## Project Structure
- `lib.rs`: Core types, error definitions
- `header.rs`: Record header implementation
- `event.rs`: Event record implementation with various event types
- `metadata.rs`: Metadata record implementation
- `bitutils.rs`: Bit manipulation macros
- `wordutils.rs`: Word and string handling utilities

## String Handling
- Strings can be inline or references
- Inline strings are stored directly in the record
- String references point to a string record
- Inline string flag is set with bit 0x1000
- Strings must be padded to multiples of 8 bytes

## Known Issues
- String padding issue: Strings just over multiples of 8 bytes may be truncated
  - Example: "operation" (9 bytes) gets truncated to "operatio" (8 bytes)
  - Related test: `test_event_record_write_with_multiple_inline_fields`