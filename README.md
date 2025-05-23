# FTFRS

A Rust library for reading and writing [Fuchsia Trace Format (FTF)](https://fuchsia.dev/fuchsia-src/development/tracing/trace-format) traces.

> ⚠️ **WARNING** ⚠️  
> This is prototype, in-development software. The API may change significantly between versions and some features are not yet fully implemented. Use in production environments is not recommended at this time.

*Note*: This is intended to be a low-level library to help build tools using this format. If you would like to use Fuchsia Trace Format, you may be better served by crates and tools built on top of it like [ftfrs-tracing](https://github.com/maruthgoyal/ftfrs-tracing) (also WIP)

## Features

- Read and write FTF trace files
- Support for common record types:
  - Metadata (Magic Number, Provider Info, Provider Event, Provider Section, Trace Info)
  - Events (Instant, Counter, Duration Begin/End/Complete)
  - Thread Records
  - String Records
  - Initialization Records
- Support for all argument types in events (Int32, UInt32, Int64, UInt64, Float, String, Pointer, KernelObjectId, Boolean, Null)

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
ftfrs = "0.1.1"
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
    let mut archive = Archive {
        records: Vec::new(),
    };
    
    archive.records.push(Record::create_magic_number());
    
    archive.records.push(Record::create_provider_info(
        1, // provider ID
        "my_provider".to_string(),
    ));
    
    archive.records.push(Record::create_string(
        1, // string index
        "example".to_string(),
    ));
    
    archive.records.push(Record::create_thread(
        1,         // thread index
        0x1234,    // process KOID
        0x5678,    // thread KOID
    ));
    
    archive.records.push(Record::create_instant_event(
        100_000, 
        ThreadRef::Ref(1),
        StringRef::Inline("category".to_string()),
        StringRef::Inline("started".to_string()),
        Vec::new(), 
    ));
    
    archive.records.push(Record::create_duration_begin_event(
        200_000, // timestamp (200 microseconds)
        ThreadRef::Ref(1),
        StringRef::Inline("category".to_string()),
        StringRef::Inline("process".to_string()),
        Vec::new(), 
    ));
    
    archive.records.push(Record::create_duration_end_event(
        300_000, // timestamp (300 microseconds)
        ThreadRef::Ref(1),
        StringRef::Inline("category".to_string()),
        StringRef::Inline("process".to_string()),
        Vec::new(), 
    ));
    
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

let duration_event = Record::create_duration_complete_event(
    100_000,  // start timestamp (100 microseconds)
    ThreadRef::Ref(1),
    StringRef::Inline("category".to_string()),
    StringRef::Inline("operation".to_string()),
    Vec::new(), 
    150_000,  // end timestamp (150 microseconds)
);
```

### Creating a Counter Event

```rust
use ftfrs::{Record, StringRef, ThreadRef, Argument};

let counter_event = Record::create_counter_event(
    200_000, 
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
let event_with_ref = Record::create_instant_event(
    300_000,
    ThreadRef::Ref(1),
    StringRef::Ref(2), // Reference to string record with index 2
    StringRef::Ref(3), 
    Vec::new(),
);

let event_with_inline = Record::create_instant_event(
    400_000,
    ThreadRef::Ref(1),
    StringRef::Inline("category".to_string()), 
    StringRef::Inline("event_name".to_string()), 
    Vec::new(),
);
```

### Adding Arguments to Events

Events can include arguments of various types to include additional data:

```rust
use ftfrs::{Argument, Record, StringRef, ThreadRef};

let args = vec![
    Argument::Int32(StringRef::Inline("count".to_string()), 42),
    Argument::UInt64(StringRef::Inline("timestamp_ms".to_string()), 1647359412000),
    
    Argument::Float(StringRef::Inline("value".to_string()), 3.14159),
    
    Argument::Str(
        StringRef::Inline("message".to_string()),
        StringRef::Inline("Operation completed successfully".to_string())
    ),
    
    Argument::Boolean(StringRef::Inline("success".to_string()), true),
    
    Argument::Pointer(StringRef::Inline("address".to_string()), 0xDEADBEEF),
    
    Argument::KernelObjectId(StringRef::Inline("process_koid".to_string()), 0x1234)
];

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
let string_record = Record::create_string(
    10, 
    "name".to_string()
);

let args = vec![
    Argument::Int32(StringRef::Ref(10), 42) 
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
- Shows how to read trace files and analyze their contents

## Roadmap 🚀

The following items are planned for future development:

- 🔄 Performance optimizations:
- 🔮 Support for remaining record types:
  - AsyncBegin/AsyncInstant/AsyncEnd events
  - FlowBegin/FlowStep/FlowEnd events
  - Blob records
  - Userspace records
  - Kernel records
  - Scheduling records
  - Log records
  - LargeBlob records

## Related Projects
- [ftfrs-tracing](https://github.com/maruthgoyal/ftfrs-tracing)

## Contributing

Contributions are welcome! Feel free to open issues or submit pull requests.

## License

This project is licensed under the [MIT License](LICENSE).