use crate::{
    event::{Event, EventRecord, Instant},
    initialization::InitializationRecord,
    metadata::{MetadataRecord, ProviderInfo, TraceInfo},
    string_rec::StringRecord,
    thread_rec::ThreadRecord,
    Archive, Record, Result, StringOrRef, ThreadOrRef,
};
use std::io::Cursor;

#[cfg(test)]
mod tests {
    use super::*;

    /// Creates a ProviderInfo metadata record for testing
    fn create_provider_info() -> MetadataRecord {
        MetadataRecord::ProviderInfo(ProviderInfo {
            provider_id: 42,
            provider_name: "test_provider".to_string(),
        })
    }

    /// Creates a TraceInfo metadata record for testing
    fn create_trace_info() -> MetadataRecord {
        MetadataRecord::TraceInfo(TraceInfo {
            trace_info_type: 1,
            data: 0x12345678,
        })
    }

    /// Creates a StringRecord for testing
    fn create_string_record() -> StringRecord {
        StringRecord {
            index: 1,
            length: 11,
            value: "test_string".to_string(),
        }
    }

    /// Creates a ThreadRecord for testing
    fn create_thread_record() -> ThreadRecord {
        ThreadRecord {
            index: 1,
            process_koid: 0x1234,
            thread_koid: 0x5678,
        }
    }

    /// Creates an InitializationRecord for testing
    fn create_initialization_record() -> InitializationRecord {
        InitializationRecord {
            ticks_per_second: 1000000,
        }
    }

    /// Creates an Instant event for testing
    fn create_instant_event() -> EventRecord {
        let event = Event {
            timestamp: 100,
            thread: ThreadOrRef::Ref(1),
            category: StringOrRef::String("category".to_string()),
            name: StringOrRef::String("event_name".to_string()),
            arguments: Vec::new(),
        };

        EventRecord::Instant(Instant { event })
    }

    /// Creates a sample archive with various record types
    fn create_sample_archive() -> Archive {
        let records = vec![
            // Magic number record
            Record::Metadata(MetadataRecord::MagicNumber),
            // Provider info record
            Record::Metadata(create_provider_info()),
            // Trace info record
            Record::Metadata(create_trace_info()),
            // String record
            Record::String(create_string_record()),
            // Thread record
            Record::Thread(create_thread_record()),
            // Initialization record
            Record::Initialization(create_initialization_record()),
            // Event record (Instant event)
            Record::Event(create_instant_event()),
        ];

        Archive { records }
    }

    #[test]
    fn test_empty_archive() -> Result<()> {
        // Create an empty archive
        let archive = Archive {
            records: Vec::new(),
        };

        // Serialize to buffer
        let mut buffer = Vec::new();
        archive.write(&mut buffer)?;

        // Verify empty buffer
        assert!(buffer.is_empty());

        // Deserialize back
        let mut cursor = Cursor::new(&buffer);
        let deserialized = Archive::read(&mut cursor)?;

        // Verify empty archive
        assert!(deserialized.records.is_empty());

        Ok(())
    }

    #[test]
    fn test_single_record_archive() -> Result<()> {
        // Create archive with single record (Magic Number)
        let archive = Archive {
            records: vec![Record::Metadata(MetadataRecord::MagicNumber)],
        };

        // Serialize to buffer
        let mut buffer = Vec::new();
        archive.write(&mut buffer)?;

        // Verify non-empty buffer
        assert!(!buffer.is_empty());

        // Deserialize back
        let mut cursor = Cursor::new(&buffer);
        let deserialized = Archive::read(&mut cursor)?;

        // Verify single record
        assert_eq!(deserialized.records.len(), 1);

        // Verify record type
        match &deserialized.records[0] {
            Record::Metadata(MetadataRecord::MagicNumber) => (),
            _ => panic!(
                "Expected MagicNumber record, got {:?}",
                deserialized.records[0]
            ),
        }

        Ok(())
    }

    #[test]
    fn test_multi_record_archive() -> Result<()> {
        // Create a sample archive with multiple records
        let archive = create_sample_archive();
        let original_len = archive.records.len();

        // Serialize to buffer
        let mut buffer = Vec::new();
        archive.write(&mut buffer)?;

        // Verify buffer has content
        assert!(!buffer.is_empty());

        // Deserialize back
        let mut cursor = Cursor::new(&buffer);
        let deserialized = Archive::read(&mut cursor)?;

        // Verify record count
        assert_eq!(deserialized.records.len(), original_len);

        // Verify records match
        for (i, (original, deserialized)) in archive
            .records
            .iter()
            .zip(deserialized.records.iter())
            .enumerate()
        {
            assert_eq!(
                original, deserialized,
                "Record at index {} doesn't match",
                i
            );
        }

        Ok(())
    }

    #[test]
    fn test_archive_with_unsupported_record_types() -> Result<()> {
        // Create an archive with a mixture of supported and unsupported record types
        let mut archive = create_sample_archive();

        // Add unsupported record types
        archive.records.push(Record::Blob);
        archive.records.push(Record::Userspace);

        // Serialize to buffer (should skip unsupported records)
        let mut buffer = Vec::new();

        // This should return an error for unsupported record types
        let write_result = archive.write(&mut buffer);
        assert!(write_result.is_err());

        // But the buffer should still contain some supported records that were written before the error
        assert!(!buffer.is_empty());

        // Deserializing should give us just the records that were successfully written
        let mut cursor = Cursor::new(&buffer);
        let deserialized = Archive::read(&mut cursor)?;

        // Verify all records are supported types
        for record in &deserialized.records {
            match record {
                Record::Metadata(_)
                | Record::Initialization(_)
                | Record::String(_)
                | Record::Thread(_)
                | Record::Event(_) => (),
                _ => panic!("Unexpected unsupported record type: {:?}", record),
            }
        }

        Ok(())
    }

    #[test]
    fn test_archive_handles_incomplete_read() -> Result<()> {
        // Create a sample archive
        let archive = create_sample_archive();

        // Serialize to buffer
        let mut buffer = Vec::new();
        archive.write(&mut buffer)?;

        // Take just part of the buffer (first 16 bytes)
        let partial_buffer = buffer[0..16].to_vec();

        // Deserialize the partial buffer - should handle EOF gracefully
        let mut cursor = Cursor::new(&partial_buffer);
        let deserialized = Archive::read(&mut cursor)?;

        // Should have parsed records up to the truncation
        assert!(!deserialized.records.is_empty());
        assert!(deserialized.records.len() <= archive.records.len());

        Ok(())
    }

    #[test]
    fn test_archive_read_write_read_cycle() -> Result<()> {
        // Create a sample archive
        let original_archive = create_sample_archive();

        // Write to buffer
        let mut buffer1 = Vec::new();
        original_archive.write(&mut buffer1)?;

        // Read back
        let mut cursor1 = Cursor::new(&buffer1);
        let intermediate_archive = Archive::read(&mut cursor1)?;

        // Write again
        let mut buffer2 = Vec::new();
        intermediate_archive.write(&mut buffer2)?;

        // Verify the buffers are identical (round-trip serialization is consistent)
        assert_eq!(buffer1.len(), buffer2.len());
        assert_eq!(buffer1, buffer2);

        // Read again
        let mut cursor2 = Cursor::new(&buffer2);
        let final_archive = Archive::read(&mut cursor2)?;

        // Verify the original and final archives are the same
        assert_eq!(original_archive.records.len(), final_archive.records.len());

        for (i, (original, final_rec)) in original_archive
            .records
            .iter()
            .zip(final_archive.records.iter())
            .enumerate()
        {
            assert_eq!(
                original, final_rec,
                "Record at index {} doesn't match after cycle",
                i
            );
        }

        Ok(())
    }

    #[test]
    fn test_archive_appending() -> Result<()> {
        // Create two separate archives
        let archive1 = Archive {
            records: vec![
                Record::Metadata(MetadataRecord::MagicNumber),
                Record::String(create_string_record()),
            ],
        };

        let thread_record = create_thread_record();
        let archive2 = Archive {
            records: vec![Record::Thread(thread_record)],
        };

        // Serialize both to the same buffer (appending)
        let mut buffer = Vec::new();
        archive1.write(&mut buffer)?;
        archive2.write(&mut buffer)?;

        // Deserialize combined buffer
        let mut cursor = Cursor::new(&buffer);
        let combined = Archive::read(&mut cursor)?;

        // Should have all 3 records
        assert_eq!(combined.records.len(), 3);

        // Verify order and content
        match &combined.records[0] {
            Record::Metadata(MetadataRecord::MagicNumber) => (),
            _ => panic!("Expected MagicNumber, got {:?}", combined.records[0]),
        }

        match &combined.records[1] {
            Record::String(sr) => {
                assert_eq!(sr.index, create_string_record().index);
                assert_eq!(sr.value, create_string_record().value);
            }
            _ => panic!("Expected StringRecord, got {:?}", combined.records[1]),
        }

        match &combined.records[2] {
            Record::Thread(tr) => {
                assert_eq!(tr.index, thread_record.index);
                assert_eq!(tr.process_koid, thread_record.process_koid);
                assert_eq!(tr.thread_koid, thread_record.thread_koid);
            }
            _ => panic!("Expected ThreadRecord, got {:?}", combined.records[2]),
        }

        Ok(())
    }
}
