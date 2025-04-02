# FTFRS Benchmarks

These benchmarks measure the performance of the FTFRS library for reading and writing trace files.

## Running the benchmarks

```bash
# Run all benchmarks
cargo bench

# Run a specific benchmark group
cargo bench -- string_handling

# Run a specific benchmark
cargo bench -- archive_read/10
```

## Benchmark Categories

1. **Read Performance**
   - `archive_read`: Reading archives of various sizes
   - `record_parsing`: Parsing individual record types

2. **Write Performance**
   - `archive_write`: Writing archives of various sizes  
   - `record_writing`: Writing individual record types
   - `span_creation`: Creating and writing span events

3. **String Handling**
   - `string_handling`: Comparing inline strings vs. string references
   - `string_record_parsing`: Parsing string records of different sizes

4. **Mixed Workloads**
   - `string_interning_ratio`: Testing different ratios of interned vs. inline strings
   - `event_argument_count`: Testing with different numbers of arguments per event
   - `throughput`: Overall events-per-second measurements

## Analyzing Results

Benchmark results are stored in `target/criterion`. HTML reports can be viewed in your browser.