use ftfrs::{Archive, Record, Result};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

fn main() -> Result<()> {
    // Get file path from command line or use default
    let args: Vec<String> = std::env::args().collect();
    let path = if args.len() > 1 {
        PathBuf::from(&args[1])
    } else {
        PathBuf::from("trace.ftf")
    };

    println!("Reading FTF trace from {}", path.display());
    
    // Open and read the file
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    
    // Parse the archive
    let archive = Archive::read(reader)?;
    
    println!("Successfully read {} records", archive.records.len());
    
    // Display information about each record
    for (i, record) in archive.records.iter().enumerate() {
        println!("Record {}: {}", i, record_type_to_string(record));
    }
    
    Ok(())
}

// Helper function to get a descriptive string for each record type
fn record_type_to_string(record: &Record) -> String {
    match record {
        Record::Metadata(meta) => match meta {
            ftfrs::MetadataRecord::MagicNumber => "Metadata: Magic Number".to_string(),
            ftfrs::MetadataRecord::ProviderInfo(info) => format!("Metadata: Provider Info (id: {}, name: {})", info.provider_id, info.provider_name),
            ftfrs::MetadataRecord::ProviderSection(section) => format!("Metadata: Provider Section (id: {})", section.provider_id),
            ftfrs::MetadataRecord::ProviderEvent(event) => format!("Metadata: Provider Event (id: {}, event: {})", event.provider_id, event.event_id),
            ftfrs::MetadataRecord::TraceInfo(info) => format!("Metadata: Trace Info (type: {}, data: {:#x})", info.trace_info_type, info.data),
        },
        Record::Initialization(init) => format!("Initialization (ticks per second: {})", init.ticks_per_second),
        Record::String(str_rec) => format!("String (index: {}, value: \"{}\")", str_rec.index, str_rec.value),
        Record::Thread(thread) => format!("Thread (index: {}, process: {:#x}, thread: {:#x})", thread.index, thread.process_koid, thread.thread_koid),
        Record::Event(event) => match event {
            ftfrs::EventRecord::Instant(_) => "Event: Instant".to_string(),
            ftfrs::EventRecord::Counter(_) => "Event: Counter".to_string(),
            ftfrs::EventRecord::DurationBegin(_) => "Event: Duration Begin".to_string(),
            ftfrs::EventRecord::DurationEnd(_) => "Event: Duration End".to_string(),
            ftfrs::EventRecord::DurationComplete(_) => "Event: Duration Complete".to_string(),
            _ => "Event: Other".to_string(),
        },
        Record::Blob => "Blob".to_string(),
        Record::Userspace => "Userspace".to_string(),
        Record::Kernel => "Kernel".to_string(),
        Record::Scheduling => "Scheduling".to_string(),
        Record::Log => "Log".to_string(),
        Record::LargeBlob => "Large Blob".to_string(),
    }
}