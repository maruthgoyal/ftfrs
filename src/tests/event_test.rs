use crate::event::EventRecord;
use crate::header::RecordHeader;
use crate::{StringOrRef, ThreadOrRef};
use anyhow::Result;
use std::io::Cursor;

#[cfg(test)]
#[allow(clippy::identity_op)]
mod tests {
    use super::*;

    #[test]
    fn test_instant_event_record_parsing() -> Result<()> {
        // Create header with:
        // - Record type: Event (bits 0-3 = 4)
        // - Size: 5 (bits 4-15) - 5 * 8 = 40 bytes
        // - Event type: Instant (bits 16-19 = 0)
        // - Number of arguments: 0 (bits 20-23 = 0)
        // - Thread ref: 5 (bits 24-31 = 5)
        // - Category ref: 10 (bits 32-47 = 10)
        // - Name ref: 15 (bits 48-63 = 15)

        let header_value: u64 = 0
            | (15 << 48)   // Name ref
            | (10 << 32)   // Category ref
            | (5 << 24)    // Thread ref
            | (0 << 20)    // Number of arguments
            | (0 << 16)    // Event type: Instant
            | (5 << 4)     // Size (5 * 8 = 40 bytes)
            | 4; // Record type: Event

        let header = RecordHeader {
            value: header_value,
        };

        // Create test data
        let timestamp: u64 = 1000000; // Example timestamp value
        let mut data = Vec::new();
        data.extend_from_slice(&timestamp.to_le_bytes());

        let mut cursor = Cursor::new(data);

        // Parse the event record
        let record = EventRecord::parse(&mut cursor, header)?;

        // Verify the record is an Instant event with expected values
        match record {
            EventRecord::Instant(instant) => {
                assert_eq!(instant.event.timestamp, 1000000);
                assert_eq!(instant.event.thread, ThreadOrRef::Ref(5));
                assert_eq!(instant.event.category, StringOrRef::Ref(10));
                assert_eq!(instant.event.name, StringOrRef::Ref(15));
                assert!(instant.event.arguments.is_empty());
            }
            _ => panic!("Expected Instant event record"),
        }

        Ok(())
    }

    #[test]
    fn test_counter_event_record_parsing() -> Result<()> {
        // Create header with:
        // - Record type: Event (bits 0-3 = 4)
        // - Size: 6 (bits 4-15) - 6 * 8 = 48 bytes
        // - Event type: Counter (bits 16-19 = 1)
        // - Number of arguments: 0 (bits 20-23 = 0)
        // - Thread ref: 1 (bits 24-31 = 1)
        // - Category ref: 2 (bits 32-47 = 2)
        // - Name ref: 3 (bits 48-63 = 3)

        let header_value: u64 = 0
            | (3 << 48)    // Name ref
            | (2 << 32)    // Category ref
            | (1 << 24)    // Thread ref
            | (0 << 20)    // Number of arguments
            | (1 << 16)    // Event type: Counter
            | (6 << 4)     // Size (6 * 8 = 48 bytes)
            | 4; // Record type: Event

        let header = RecordHeader {
            value: header_value,
        };

        // Create test data
        let timestamp: u64 = 1000000; // Example timestamp value
        let counter_id: u64 = 42; // Example counter ID

        let mut data = Vec::new();
        data.extend_from_slice(&timestamp.to_le_bytes());
        data.extend_from_slice(&counter_id.to_le_bytes());

        let mut cursor = Cursor::new(data);

        // Parse the event record
        let record = EventRecord::parse(&mut cursor, header)?;

        // Verify the record is a Counter event with expected values
        match record {
            EventRecord::Counter(counter) => {
                assert_eq!(counter.event.timestamp, 1000000);
                assert_eq!(counter.event.thread, ThreadOrRef::Ref(1));
                assert_eq!(counter.event.category, StringOrRef::Ref(2));
                assert_eq!(counter.event.name, StringOrRef::Ref(3));
                assert_eq!(counter.counter_id, 42);
                assert!(counter.event.arguments.is_empty());
            }
            _ => panic!("Expected Counter event record"),
        }

        Ok(())
    }

    #[test]
    fn test_duration_begin_event_record_parsing() -> Result<()> {
        // Create header with:
        // - Record type: Event (bits 0-3 = 4)
        // - Size: 5 (bits 4-15) - 5 * 8 = 40 bytes
        // - Event type: DurationBegin (bits 16-19 = 2)
        // - Number of arguments: 0 (bits 20-23 = 0)
        // - Thread ref: 7 (bits 24-31 = 7)
        // - Category ref: 12 (bits 32-47 = 12)
        // - Name ref: 20 (bits 48-63 = 20)

        let header_value: u64 = 0
            | (20 << 48)   // Name ref
            | (12 << 32)   // Category ref
            | (7 << 24)    // Thread ref
            | (0 << 20)    // Number of arguments
            | (2 << 16)    // Event type: DurationBegin
            | (5 << 4)     // Size (5 * 8 = 40 bytes)
            | 4; // Record type: Event

        let header = RecordHeader {
            value: header_value,
        };

        // Create test data
        let timestamp: u64 = 2000000; // Example timestamp value
        let mut data = Vec::new();
        data.extend_from_slice(&timestamp.to_le_bytes());

        let mut cursor = Cursor::new(data);

        // Parse the event record
        let record = EventRecord::parse(&mut cursor, header)?;

        // Verify the record is a DurationBegin event with expected values
        match record {
            EventRecord::DurationBegin(begin) => {
                assert_eq!(begin.event.timestamp, 2000000);
                assert_eq!(begin.event.thread, ThreadOrRef::Ref(7));
                assert_eq!(begin.event.category, StringOrRef::Ref(12));
                assert_eq!(begin.event.name, StringOrRef::Ref(20));
                assert!(begin.event.arguments.is_empty());
            }
            _ => panic!("Expected DurationBegin event record"),
        }

        Ok(())
    }

    #[test]
    fn test_duration_end_event_record_parsing() -> Result<()> {
        // Create header with:
        // - Record type: Event (bits 0-3 = 4)
        // - Size: 5 (bits 4-15) - 5 * 8 = 40 bytes
        // - Event type: DurationEnd (bits 16-19 = 3)
        // - Number of arguments: 0 (bits 20-23 = 0)
        // - Thread ref: 7 (bits 24-31 = 7)
        // - Category ref: 12 (bits 32-47 = 12)
        // - Name ref: 20 (bits 48-63 = 20)

        let header_value: u64 = 0
            | (20 << 48)   // Name ref
            | (12 << 32)   // Category ref
            | (7 << 24)    // Thread ref
            | (0 << 20)    // Number of arguments
            | (3 << 16)    // Event type: DurationEnd
            | (5 << 4)     // Size (5 * 8 = 40 bytes)
            | 4; // Record type: Event

        let header = RecordHeader {
            value: header_value,
        };

        // Create test data
        let timestamp: u64 = 3000000; // Example timestamp value
        let mut data = Vec::new();
        data.extend_from_slice(&timestamp.to_le_bytes());

        let mut cursor = Cursor::new(data);

        // Parse the event record
        let record = EventRecord::parse(&mut cursor, header)?;

        // Verify the record is a DurationEnd event with expected values
        match record {
            EventRecord::DurationEnd(end) => {
                assert_eq!(end.event.timestamp, 3000000);
                assert_eq!(end.event.thread, ThreadOrRef::Ref(7));
                assert_eq!(end.event.category, StringOrRef::Ref(12));
                assert_eq!(end.event.name, StringOrRef::Ref(20));
                assert!(end.event.arguments.is_empty());
            }
            _ => panic!("Expected DurationEnd event record"),
        }

        Ok(())
    }

    #[test]
    fn test_duration_complete_event_record_parsing() -> Result<()> {
        // Create header with:
        // - Record type: Event (bits 0-3 = 4)
        // - Size: 6 (bits 4-15) - 6 * 8 = 48 bytes
        // - Event type: DurationComplete (bits 16-19 = 4)
        // - Number of arguments: 0 (bits 20-23 = 0)
        // - Thread ref: 8 (bits 24-31 = 8)
        // - Category ref: 15 (bits 32-47 = 15)
        // - Name ref: 22 (bits 48-63 = 22)

        let header_value: u64 = 0
            | (22 << 48)   // Name ref
            | (15 << 32)   // Category ref
            | (8 << 24)    // Thread ref
            | (0 << 20)    // Number of arguments
            | (4 << 16)    // Event type: DurationComplete
            | (6 << 4)     // Size (6 * 8 = 48 bytes)
            | 4; // Record type: Event

        let header = RecordHeader {
            value: header_value,
        };

        // Create test data
        let timestamp: u64 = 4000000; // Example timestamp value
        let duration_ticks: u64 = 500000; // Example duration in ticks

        let mut data = Vec::new();
        data.extend_from_slice(&timestamp.to_le_bytes());
        data.extend_from_slice(&duration_ticks.to_le_bytes());

        let mut cursor = Cursor::new(data);

        // Parse the event record
        let record = EventRecord::parse(&mut cursor, header)?;

        // Verify the record is a DurationComplete event with expected values
        match record {
            EventRecord::DurationComplete(complete) => {
                assert_eq!(complete.event.timestamp, 4000000);
                assert_eq!(complete.event.thread, ThreadOrRef::Ref(8));
                assert_eq!(complete.event.category, StringOrRef::Ref(15));
                assert_eq!(complete.event.name, StringOrRef::Ref(22));
                assert_eq!(complete.duration_ticks, 500000);
                assert!(complete.event.arguments.is_empty());
            }
            _ => panic!("Expected DurationComplete event record"),
        }

        Ok(())
    }

    #[test]
    fn test_event_type_parsing_error() -> Result<()> {
        // Create header with an invalid event type (11)
        // - Record type: Event (bits 0-3 = 4)
        // - Size: 5 (bits 4-15) - 5 * 8 = 40 bytes
        // - Event type: Invalid (bits 16-19 = 11)
        // - Number of arguments: 0 (bits 20-23 = 0)
        // - Thread ref: 1 (bits 24-31 = 1)
        // - Category ref: 2 (bits 32-47 = 2)
        // - Name ref: 3 (bits 48-63 = 3)

        let header_value: u64 = 0
            | (3 << 48)    // Name ref
            | (2 << 32)    // Category ref
            | (1 << 24)    // Thread ref
            | (0 << 20)    // Number of arguments
            | (11 << 16)   // Event type: Invalid (11)
            | (5 << 4)     // Size (5 * 8 = 40 bytes)
            | 4; // Record type: Event

        let header = RecordHeader {
            value: header_value,
        };

        // Create test data
        let timestamp: u64 = 1000000; // Example timestamp value
        let mut data = Vec::new();
        data.extend_from_slice(&timestamp.to_le_bytes());

        let mut cursor = Cursor::new(data);

        // Parse the event record should fail with an EventTypeParseError
        let result = EventRecord::parse(&mut cursor, header);
        assert!(result.is_err());

        // Ideally we would check the specific error type, but we'd need to expose it more fully

        Ok(())
    }

    #[test]
    fn test_event_with_inline_thread() -> Result<()> {
        // Create header with:
        // - Record type: Event (bits 0-3 = 4)
        // - Size: 7 (bits 4-15) - 7 * 8 = 56 bytes
        // - Event type: Instant (bits 16-19 = 0)
        // - Number of arguments: 0 (bits 20-23 = 0)
        // - Thread inline: 0 (bits 24-31 = 0) - This indicates inline thread
        // - Category ref: 2 (bits 32-47 = 2)
        // - Name ref: 3 (bits 48-63 = 3)

        let header_value: u64 = 0
            | (3 << 48)    // Name ref
            | (2 << 32)    // Category ref
            | (0 << 24)    // Thread inline (0 means inline)
            | (0 << 20)    // Number of arguments
            | (0 << 16)    // Event type: Instant
            | (7 << 4)     // Size (7 * 8 = 56 bytes)
            | 4; // Record type: Event

        let header = RecordHeader {
            value: header_value,
        };

        // Create test data
        let timestamp: u64 = 1000000; // Example timestamp value
        let process_id: u64 = 12345; // Example process ID
        let thread_id: u64 = 67890; // Example thread ID

        let mut data = Vec::new();
        data.extend_from_slice(&timestamp.to_le_bytes());
        data.extend_from_slice(&process_id.to_le_bytes());
        data.extend_from_slice(&thread_id.to_le_bytes());

        let mut cursor = Cursor::new(data);

        // Parse the event record
        let record = EventRecord::parse(&mut cursor, header)?;

        // Verify the record is an Instant event with inline thread values
        match record {
            EventRecord::Instant(instant) => {
                assert_eq!(instant.event.timestamp, 1000000);
                assert_eq!(
                    instant.event.thread,
                    ThreadOrRef::ProcessAndThread(12345, 67890)
                );
                assert_eq!(instant.event.category, StringOrRef::Ref(2));
                assert_eq!(instant.event.name, StringOrRef::Ref(3));
                assert!(instant.event.arguments.is_empty());
            }
            _ => panic!("Expected Instant event record"),
        }

        Ok(())
    }

    #[test]
    fn test_event_with_inline_category() -> Result<()> {
        // Create header with:
        // - Record type: Event (bits 0-3 = 4)
        // - Size: 6 (bits 4-15) - 6 * 8 = 48 bytes
        // - Event type: Instant (bits 16-19 = 0)
        // - Number of arguments: 0 (bits 20-23 = 0)
        // - Thread ref: 1 (bits 24-31 = 1)
        // - Category inline: 0x8003 (bits 32-47 = 0x8003) - High bit set means inline
        // - Name ref: 3 (bits 48-63 = 3)

        // The value 0x8003 means inline with length 3 (0x8000 | 0x0003)
        let header_value: u64 = 0
            | (3 << 48)        // Name ref
            | (0x8003 << 32)   // Category inline with length 3
            | (1 << 24)        // Thread ref
            | (0 << 20)        // Number of arguments
            | (0 << 16)        // Event type: Instant
            | (6 << 4)         // Size (6 * 8 = 48 bytes)
            | 4; // Record type: Event

        let header = RecordHeader {
            value: header_value,
        };

        // Create test data
        let timestamp: u64 = 1000000; // Example timestamp value

        let mut data = Vec::new();
        data.extend_from_slice(&timestamp.to_le_bytes());
        data.extend_from_slice(b"cat\0\0\0\0\0"); // "cat" padded to 8 bytes

        let mut cursor = Cursor::new(data);

        // Parse the event record
        let record = EventRecord::parse(&mut cursor, header)?;

        // Verify the record is an Instant event with inline category
        match record {
            EventRecord::Instant(instant) => {
                assert_eq!(instant.event.timestamp, 1000000);
                assert_eq!(instant.event.thread, ThreadOrRef::Ref(1));
                assert_eq!(
                    instant.event.category,
                    StringOrRef::String("cat".to_string())
                );
                assert_eq!(instant.event.name, StringOrRef::Ref(3));
                assert!(instant.event.arguments.is_empty());
            }
            _ => panic!("Expected Instant event record"),
        }

        Ok(())
    }

    #[test]
    fn test_event_with_inline_name() -> Result<()> {
        // Create header with:
        // - Record type: Event (bits 0-3 = 4)
        // - Size: 6 (bits 4-15) - 6 * 8 = 48 bytes
        // - Event type: Instant (bits 16-19 = 0)
        // - Number of arguments: 0 (bits 20-23 = 0)
        // - Thread ref: 1 (bits 24-31 = 1)
        // - Category ref: 2 (bits 32-47 = 2)
        // - Name inline: 0x8004 (bits 48-63 = 0x8004) - High bit set means inline

        // The value 0x8004 means inline with length 4 (0x8000 | 0x0004)
        let header_value: u64 = 0
            | (0x8004 << 48)   // Name inline with length 4
            | (2 << 32)        // Category ref
            | (1 << 24)        // Thread ref
            | (0 << 20)        // Number of arguments
            | (0 << 16)        // Event type: Instant
            | (6 << 4)         // Size (6 * 8 = 48 bytes)
            | 4; // Record type: Event

        let header = RecordHeader {
            value: header_value,
        };

        // Create test data
        let timestamp: u64 = 1000000; // Example timestamp value

        let mut data = Vec::new();
        data.extend_from_slice(&timestamp.to_le_bytes());
        data.extend_from_slice(b"test\0\0\0\0"); // "test" padded to 8 bytes

        let mut cursor = Cursor::new(data);

        // Parse the event record
        let record = EventRecord::parse(&mut cursor, header)?;

        // Verify the record is an Instant event with inline name
        match record {
            EventRecord::Instant(instant) => {
                assert_eq!(instant.event.timestamp, 1000000);
                assert_eq!(instant.event.thread, ThreadOrRef::Ref(1));
                assert_eq!(instant.event.category, StringOrRef::Ref(2));
                assert_eq!(instant.event.name, StringOrRef::String("test".to_string()));
                assert!(instant.event.arguments.is_empty());
            }
            _ => panic!("Expected Instant event record"),
        }

        Ok(())
    }
}
