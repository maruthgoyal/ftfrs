use ftfrs::{
    Archive, Counter, DurationBegin, DurationComplete, DurationEnd, Event, EventRecord, InitializationRecord, Instant, MetadataRecord, ProviderEvent, ProviderInfo, ProviderSection, Record, Result, StringOrRef, StringRecord, ThreadOrRef, ThreadRecord, TraceInfo
};
use std::fs::File;
use std::io::BufWriter;

fn main() -> Result<()> {
    // Create a sample archive
    let archive = create_sample_archive()?;
    
    // Write it to a file
    let file = File::create("trace.ftf")?;
    let writer = BufWriter::new(file);
    
    println!("Writing archive with {} records to trace.ftf", archive.records.len());
    archive.write(writer)?;
    println!("Successfully wrote trace file");
    
    Ok(())
}

#[allow(clippy::vec_init_then_push)]
fn create_sample_archive() -> Result<Archive> {
    // Create various records
    let mut records = Vec::new();
    
    // Standard header: Magic number record
    records.push(Record::Metadata(MetadataRecord::MagicNumber));
    
    // Add an initialization record with 1 billion ticks per second
    // records.push(Record::Initialization(InitializationRecord {
    //     ticks_per_second: 1_000_000_000,
    // }));
    
    // Add provider information
    let provider_id = 42;
    records.push(Record::Metadata(MetadataRecord::ProviderInfo(ProviderInfo {
        provider_id,
        provider_name: "SampleProvider".to_string(),
    })));
    
    // Add provider section
    records.push(Record::Metadata(MetadataRecord::ProviderSection(ProviderSection {
        provider_id,
    })));
    
    // Add provider event
    records.push(Record::Metadata(MetadataRecord::ProviderEvent(ProviderEvent {
        provider_id,
        event_id: 1,
    })));
    
    // Add trace info
    records.push(Record::Metadata(MetadataRecord::TraceInfo(TraceInfo {
        trace_info_type: 1, // This could be a timestamp, process ID, etc.
        data: 0x123456789ABCDEF,
    })));
    
    // // Add string records for the string table
    records.push(Record::String(StringRecord {
        index: 1,
        length: 8,
        value: "Category".to_string(),
    }));
    
    records.push(Record::String(StringRecord {
        index: 2,
        length: 14,
        value: "SampleFunction".to_string(),
    }));
    
    records.push(Record::String(StringRecord {
        index: 3,
        length: 10,
        value: "EventName".to_string(),
    }));
    
    // Add thread record
    records.push(Record::Thread(ThreadRecord {
        index: 1,
        process_koid: 0x1234,
        thread_koid: 0x5678,
    }));
    
    // // Base time for events
    let base_time = 1_000_000;
    
    // Create an Instant event using string references
    let instant_event = Event {
        timestamp: base_time,
        thread: ThreadOrRef::Ref(1), // Reference the thread we defined above
        category: StringOrRef::Ref(1), // Reference "Category"
        name: StringOrRef::Ref(3),    // Reference "EventName"
        arguments: Vec::new(),
    };
    records.push(Record::Event(EventRecord::Instant(Instant {
        event: instant_event,
    })));
    
    // Create a Duration Begin event with inline strings
    let begin_event = Event {
        timestamp: base_time + 100,
        thread: ThreadOrRef::Ref(1),
        category: StringOrRef::String("TestCategory".to_string()),
        name: StringOrRef::Ref(2), // Reference "SampleFunction"
        arguments: Vec::new(),
    };
    records.push(Record::Event(EventRecord::DurationBegin(DurationBegin {
        event: begin_event,
    })));
    
    // Create a Duration End event
    let end_event = Event {
        timestamp: base_time + 500,
        thread: ThreadOrRef::Ref(1),
        category: StringOrRef::String("TestCategory".to_string()),
        name: StringOrRef::Ref(2), // Reference "SampleFunction"
        arguments: Vec::new(),
    };
    records.push(Record::Event(EventRecord::DurationEnd(DurationEnd {
        event: end_event,
    })));
    
    // Create a Duration Complete event (alternative to Begin/End pair)
    let complete_event = Event {
        timestamp: base_time + 800,
        thread: ThreadOrRef::ProcessAndThread(0x9ABC, 0xDEF0), // Inline thread information
        category: StringOrRef::Ref(1), // Reference "Category"
        name: StringOrRef::String("CompleteDuration".to_string()),
        arguments: Vec::new(),
    };
    records.push(Record::Event(EventRecord::DurationComplete(DurationComplete {
        event: complete_event,
        end_ts: base_time + 900, // Duration in ticks
    })));
    
    Ok(Archive { records })
}