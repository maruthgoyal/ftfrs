# FTFRS

A Rust library for reading and writing [Fuchsia Trace Format (FTF)](https://fuchsia.dev/fuchsia-src/development/tracing/trace-format) traces.

> ⚠️ **WARNING** ⚠️  
> This is prototype, in-development software. The API may change significantly between versions and some features are not yet fully implemented. Use in production environments is not recommended at this time.

## Features

- Read and write FTF trace files
- Support for common record types:
  - Metadata (Magic Number, Provider Info, Provider Event, Provider Section, Trace Info)
  - Events (Instant, Counter, Duration Begin/End/Complete)
  - Thread Records
  - String Records
  - Initialization Records
- Support for all argument types in events (Int32, UInt32, Int64, UInt64, Float, String, Pointer, KernelObjectId, Boolean, Null)
- Ergonomic API for creating and manipulating trace records

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
ftfrs = "0.1.0"
```

## Usage Examples

### Reading an existing trace file

```rust
use ftfrs::{Archive, Result};
use std::fs::File;
use std::io::BufReader;

fn main() -> Result<()> {
    // Open the trace file
    let file = File::open("trace.ftf")?;
    let reader = BufReader::new(file);
    
    // Parse the trace archive
    let archive = Archive::read(reader)?;
    
    // Process the records in the archive
    for (i, record) in archive.records.iter().enumerate() {
        println!("Record {}: {:?}", i, record);
    }
    
    Ok(())
}
```

### Creating a new trace file

```rust
use ftfrs::{
    Archive, Record, StringRef, ThreadRef, Result
};
use std::fs::File;
use std::io::BufWriter;

fn main() -> Result<()> {
    // Create a new archive
    let mut archive = Archive {
        records: Vec::new(),
    };
    
    // Add magic number record
    archive.records.push(Record::create_magic_number());
    
    // Add provider info
    archive.records.push(Record::create_provider_info(
        1, // provider ID
        "my_provider".to_string(),
    ));
    
    // Add a string record
    archive.records.push(Record::create_string(
        1, // string index
        7, // string length
        "example".to_string(),
    ));
    
    // Add a thread record
    archive.records.push(Record::create_thread(
        1,         // thread index
        0x1234,    // process KOID
        0x5678,    // thread KOID
    ));
    
    // Add an instant event
    archive.records.push(Record::create_instant_event(
        100_000, // timestamp (100 microseconds)
        ThreadRef::Ref(1),
        StringRef::Inline("category".to_string()),
        StringRef::Inline("started".to_string()),
        Vec::new(), // arguments
    ));
    
    // Add a duration begin event
    archive.records.push(Record::create_duration_begin_event(
        200_000, // timestamp (200 microseconds)
        ThreadRef::Ref(1),
        StringRef::Inline("category".to_string()),
        StringRef::Inline("process".to_string()),
        Vec::new(), // arguments
    ));
    
    // Add a duration end event
    archive.records.push(Record::create_duration_end_event(
        300_000, // timestamp (300 microseconds)
        ThreadRef::Ref(1),
        StringRef::Inline("category".to_string()),
        StringRef::Inline("process".to_string()),
        Vec::new(), // arguments
    ));
    
    // Write the archive to a file
    let file = File::create("new_trace.ftf")?;
    let writer = BufWriter::new(file);
    archive.write(writer)?;
    
    println!("Trace file successfully written!");
    Ok(())
}
```

### Creating a Duration Event with Both Start and End

```rust
use ftfrs::{Record, StringRef, ThreadRef};

// Create a duration complete event (captures both start and end)
let duration_event = Record::create_duration_complete_event(
    100_000,  // start timestamp (100 microseconds)
    ThreadRef::Ref(1),
    StringRef::Inline("category".to_string()),
    StringRef::Inline("operation".to_string()),
    Vec::new(), // arguments
    150_000,  // end timestamp (150 microseconds)
);
```

### Creating a Counter Event

```rust
use ftfrs::{Record, StringRef, ThreadRef, Argument};

// Create a counter event
let counter_event = Record::create_counter_event(
    200_000, // timestamp
    ThreadRef::Ref(1),
    StringRef::Inline("metrics".to_string()),
    StringRef::Inline("cpu_usage".to_string()),
    Vec::new(), // arguments (would typically contain the counter value)
    42,       // counter ID
);
```

### String and Thread References

When creating events, you can use either inline strings or references to previously defined string records:

```rust
// Using a string reference (more efficient for repeated strings)
let event_with_ref = Record::create_instant_event(
    300_000,
    ThreadRef::Ref(1),
    StringRef::Ref(2), // Reference to string record with index 2
    StringRef::Ref(3), // Reference to string record with index 3
    Vec::new(),
);

// Using an inline string (simpler for one-off strings)
let event_with_inline = Record::create_instant_event(
    400_000,
    ThreadRef::Ref(1),
    StringRef::Inline("category".to_string()), // Inline string
    StringRef::Inline("event_name".to_string()), // Inline string
    Vec::new(),
);
```

### Adding Arguments to Events

Events can include arguments of various types to include additional data:

```rust
use ftfrs::{Argument, Record, StringRef, ThreadRef};

// Create a vector of arguments of different types
let args = vec![
    // Integer arguments
    Argument::Int32(StringRef::Inline("count".to_string()), 42),
    Argument::UInt64(StringRef::Inline("timestamp_ms".to_string()), 1647359412000),
    
    // Floating point argument
    Argument::Float(StringRef::Inline("value".to_string()), 3.14159),
    
    // String argument (can use inline or reference strings for both name and value)
    Argument::Str(
        StringRef::Inline("message".to_string()),
        StringRef::Inline("Operation completed successfully".to_string())
    ),
    
    // Boolean argument
    Argument::Boolean(StringRef::Inline("success".to_string()), true),
    
    // Pointer argument (memory address)
    Argument::Pointer(StringRef::Inline("address".to_string()), 0xDEADBEEF),
    
    // Kernel object ID
    Argument::KernelObjectId(StringRef::Inline("process_koid".to_string()), 0x1234)
];

// Create an event with arguments
let event_with_args = Record::create_instant_event(
    500_000,
    ThreadRef::Ref(1),
    StringRef::Inline("app".to_string()),
    StringRef::Inline("process_data".to_string()),
    args
);
```

Argument names can use string references for efficiency when used repeatedly:

```rust
// First, create a string record for the argument name
let string_record = Record::create_string(
    10, // string index
    5,  // string length
    "name".to_string()
);

// Then use a reference to this string in arguments
let args = vec![
    Argument::Int32(StringRef::Ref(10), 42) // Reference to "name" string
];
```

## Benchmarks 📊

The library includes comprehensive benchmarks to measure performance of various operations:

```bash
# Run all benchmarks
cargo bench

# Run a specific benchmark group
cargo bench -- string_handling

# Run a specific benchmark
cargo bench -- archive_read/10
```

Key benchmarking categories:

1. **Read Performance**
   - Reading archives of various sizes
   - Parsing individual record types

2. **Write Performance**
   - Writing archives of various sizes
   - Creating and writing span events

3. **String Handling Performance**
   - Comparing inline strings vs. string references
   - Performance with different string sizes

4. **Mixed Workloads**
   - Different ratios of interned vs. inline strings
   - Performance with varying numbers of arguments
   - Overall throughput (events per second)

Benchmark results help identify performance characteristics and guide optimization decisions.

## Example Tool 🛠️

The repository includes an example tool that demonstrates reading and writing trace files.

### Running the Example

```bash
# Create a sample trace file
cargo run --example trace_tool write [output_file.ftf]

# Read and display a trace file
cargo run --example trace_tool read <trace_file.ftf>
```

The example tool:
- Creates a sample trace with various record types (metadata, events, strings, etc.)
- Demonstrates string references and inline strings
- Shows how to read trace files and analyze their contents
- Provides a practical example of a complete trace structure

## Roadmap 🚀

The following items are planned for future development:

- ✅ Argument support for events (completed!)
- 🔄 Performance optimizations:
  - ✅ Add benchmarks (completed!)
  - Memory usage improvements (in-memory layout mirroring disk format to reduce memory footprint)
  - Write optimizations (avoid unnecessary string copies for padding)
- 🔮 Support for remaining record types:
  - AsyncBegin/AsyncInstant/AsyncEnd events
  - FlowBegin/FlowStep/FlowEnd events
  - Blob records
  - Userspace records
  - Kernel records
  - Scheduling records
  - Log records
  - LargeBlob records

## Contributing

Contributions are welcome! Feel free to open issues or submit pull requests.

## License

This project is licensed under the [MIT License](LICENSE).