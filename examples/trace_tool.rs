use ftfrs::{Archive, Argument, Record, Result, StringRef, ThreadRef};
use std::env;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::process;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    match args[1].as_str() {
        "read" => {
            if args.len() < 3 {
                println!("Error: Missing trace file path for read operation");
                print_usage();
                process::exit(1);
            }
            read_trace(&args[2])?;
        }
        "write" => {
            let output_path = if args.len() >= 3 {
                &args[2]
            } else {
                "sample_trace.ftf"
            };
            write_sample_trace(output_path)?;
        }
        _ => {
            println!("Error: Unknown command '{}'", args[1]);
            print_usage();
            process::exit(1);
        }
    }

    Ok(())
}

fn print_usage() {
    println!("Usage:");
    println!("  trace_tool read <trace_file>     - Read and display trace file contents");
    println!("  trace_tool write [output_file]   - Create a sample trace file (default: sample_trace.ftf)");
}

fn read_trace(file_path: &str) -> Result<()> {
    println!("Reading trace file: {}", file_path);

    let file = match File::open(file_path) {
        Ok(f) => f,
        Err(e) => {
            println!("Error opening file: {}", e);
            process::exit(1);
        }
    };

    let reader = BufReader::new(file);
    let archive = Archive::read(reader)?;

    println!("Successfully read {} records", archive.records.len());

    // Print summary of records by type
    let mut metadata_count = 0;
    let mut initialization_count = 0;
    let mut string_count = 0;
    let mut thread_count = 0;
    let mut event_count = 0;
    let mut other_count = 0;

    for record in &archive.records {
        match record {
            Record::Metadata(_) => metadata_count += 1,
            Record::Initialization(_) => initialization_count += 1,
            Record::String(_) => string_count += 1,
            Record::Thread(_) => thread_count += 1,
            Record::Event(_) => event_count += 1,
            _ => other_count += 1,
        }
    }

    println!("\nRecord Type Summary:");
    println!("--------------------");
    println!("Metadata Records:      {}", metadata_count);
    println!("Initialization Records: {}", initialization_count);
    println!("String Records:        {}", string_count);
    println!("Thread Records:        {}", thread_count);
    println!("Event Records:         {}", event_count);
    println!("Other Records:         {}", other_count);

    // Print first 10 records in detail
    let display_count = std::cmp::min(10, archive.records.len());

    if display_count > 0 {
        println!("\nFirst {} Records:", display_count);
        println!("--------------------");

        for (i, record) in archive.records.iter().take(display_count).enumerate() {
            println!("Record {}: {:?}", i, record);
        }

        if archive.records.len() > display_count {
            println!(
                "... and {} more records",
                archive.records.len() - display_count
            );
        }
    }

    Ok(())
}

fn write_sample_trace(file_path: &str) -> Result<()> {
    println!("Creating sample trace file: {}", file_path);

    // Create a new archive
    let mut archive = Archive {
        records: Vec::new(),
    };

    // Add magic number record
    archive.records.push(Record::create_magic_number());

    // Add initialization record
    archive
        .records
        .push(Record::create_initialization(1_000_000)); // 1M ticks per second

    // Add provider info
    archive.records.push(Record::create_provider_info(
        1, // provider ID
        "sample_provider".to_string(),
    ));

    // Add some string records for reuse
    archive.records.push(Record::create_string(
        1, // index
        9, // length
        "rendering".to_string(),
    ));

    archive.records.push(Record::create_string(
        2, // index
        8, // length
        "database".to_string(),
    ));

    archive.records.push(Record::create_string(
        3, // index
        7, // length
        "network".to_string(),
    ));
    
    // Add string records for argument names
    archive.records.push(Record::create_string(
        4, // index
        5, // length
        "count".to_string(),
    ));
    
    archive.records.push(Record::create_string(
        5, // index
        8, // length
        "duration".to_string(),
    ));
    
    archive.records.push(Record::create_string(
        6, // index
        4, // length
        "size".to_string(),
    ));

    // Add thread record
    archive.records.push(Record::create_thread(
        1,      // thread index
        0x1234, // process KOID
        0x5678, // thread KOID
    ));

    // Add some events with references to the strings

    // Instant event with inline argument names
    let frame_args = vec![
        Argument::Int32(StringRef::Inline("frame_number".to_string()), 42),
        Argument::Float(StringRef::Inline("render_time_ms".to_string()), 16.7),
    ];
    
    archive.records.push(Record::create_instant_event(
        100_000, // timestamp
        ThreadRef::Ref(1),
        StringRef::Ref(1), // "rendering"
        StringRef::Inline("frame_presented".to_string()),
        frame_args,
    ));

    // Counter event with string reference argument names
    let fps_args = vec![
        Argument::UInt32(StringRef::Ref(4), 60), // "count" = 60
    ];
    
    archive.records.push(Record::create_counter_event(
        150_000, // timestamp
        ThreadRef::Ref(1),
        StringRef::Ref(1), // "rendering"
        StringRef::Inline("fps".to_string()),
        fps_args,
        60, // counter ID (60 fps)
    ));

    // Duration begin with multiple argument types
    let query_begin_args = vec![
        Argument::Str(
            StringRef::Ref(6), // "size"
            StringRef::Inline("large".to_string()),
        ),
        Argument::UInt64(
            StringRef::Inline("timeout_ms".to_string()),
            5000,
        ),
        Argument::Str(
            StringRef::Inline("query_type".to_string()),
            StringRef::Inline("SELECT".to_string()),
        ),
    ];
    
    archive.records.push(Record::create_duration_begin_event(
        200_000, // timestamp
        ThreadRef::Ref(1),
        StringRef::Ref(2), // "database"
        StringRef::Inline("query".to_string()),
        query_begin_args,
    ));

    // Duration end with result arguments
    let query_end_args = vec![
        Argument::Int32(StringRef::Ref(4), 157),  // "count" = 157 (rows returned)
        Argument::Int64(StringRef::Ref(5), 42),   // "duration" = 42 (milliseconds)
        Argument::Boolean(StringRef::Inline("success".to_string()), true),
    ];
    
    archive.records.push(Record::create_duration_end_event(
        250_000, // timestamp
        ThreadRef::Ref(1),
        StringRef::Ref(2), // "database"
        StringRef::Inline("query".to_string()),
        query_end_args,
    ));

    // A duration complete event with pointer and kernel object ID arguments
    let http_args = vec![
        Argument::Pointer(StringRef::Inline("buffer_ptr".to_string()), 0xDEADBEEF),
        Argument::KernelObjectId(StringRef::Inline("connection_id".to_string()), 0x12345678),
        Argument::UInt64(StringRef::Ref(6), 1024), // "size" = 1024 bytes
        Argument::Int32(StringRef::Inline("status_code".to_string()), 200),
    ];
    
    archive.records.push(Record::create_duration_complete_event(
        300_000, // start timestamp
        ThreadRef::Ref(1),
        StringRef::Ref(3), // "network"
        StringRef::Inline("http_request".to_string()),
        http_args,
        350_000, // end timestamp
    ));

    // An event with process and thread IDs and Null argument
    let boot_args = vec![
        Argument::Null(StringRef::Inline("initialized".to_string())),
        Argument::UInt64(StringRef::Inline("boot_time_ms".to_string()), 1253),
    ];
    
    archive.records.push(Record::create_instant_event(
        400_000, // timestamp
        ThreadRef::Inline {
            process_koid: 0x9ABC,
            thread_koid: 0xDEF0,
        },
        StringRef::Inline("system".to_string()),
        StringRef::Inline("boot_complete".to_string()),
        boot_args,
    ));

    // Write the archive to a file
    let file = match File::create(file_path) {
        Ok(f) => f,
        Err(e) => {
            println!("Error creating file: {}", e);
            process::exit(1);
        }
    };

    let writer = BufWriter::new(file);
    archive.write(writer)?;

    println!(
        "Successfully wrote {} records to {}",
        archive.records.len(),
        file_path
    );

    // Print summary of what was written
    println!("\nWrote the following records:");
    println!("- 1 Magic Number record");
    println!("- 1 Initialization record (1M ticks per second)");
    println!("- 1 Provider Info record (id: 1, name: 'sample_provider')");
    println!("- 6 String records (indices 1-6)");
    println!("- 1 Thread record (thread index: 1)");
    println!("- 2 Instant events with arguments (frame data, boot info)");
    println!("- 1 Counter event with fps arguments");
    println!("- 1 Duration Begin event with query parameters");
    println!("- 1 Duration End event with query results");
    println!("- 1 Duration Complete event with network request details");
    println!("- Various argument types demonstrated (Int32, UInt32, Int64, UInt64, Float, String, Boolean, Pointer, KernelObjectId, Null)");

    println!(
        "\nYou can view this trace with: ./trace_tool read {}",
        file_path
    );

    Ok(())
}
