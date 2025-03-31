use crate::event::{
    Counter, DurationBegin, DurationComplete, DurationEnd, Event, EventRecord, Instant,
};
use crate::header::RecordHeader;
use crate::{Result, StringOrRef, ThreadOrRef};
use std::io::{Cursor, Read};

#[cfg(test)]
#[allow(clippy::identity_op)]
mod tests {
    use super::*;
    use crate::Record;

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

    // New tests for the write functionality

    #[test]
    fn test_instant_event_record_write() -> Result<()> {
        // Create an instant event with reference thread, category, and name
        let event = Event {
            timestamp: 1000000,
            thread: ThreadOrRef::Ref(5),
            category: StringOrRef::Ref(10),
            name: StringOrRef::Ref(15),
            arguments: Vec::new(),
        };

        let instant_record = EventRecord::Instant(Instant { event });

        // Write it to a buffer
        let mut buffer = Vec::new();
        instant_record.write(&mut buffer)?;

        // Verify the length - should be 16 bytes (8 for header + 8 for timestamp)
        assert_eq!(buffer.len(), 16);

        // Verify the header
        let header_value = u64::from_le_bytes([
            buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6], buffer[7],
        ]);
        let header = RecordHeader {
            value: header_value,
        };

        assert_eq!(header.record_type()?, crate::header::RecordType::Event);

        // Check event type (bits 16-19 = 0 for Instant)
        let event_type = (header_value >> 16) & 0xF;
        assert_eq!(event_type, 0);

        // Check thread ref (bits 24-31 = 5)
        let thread_ref = (header_value >> 24) & 0xFF;
        assert_eq!(thread_ref, 5);

        // Check category ref (bits 32-47 = 10)
        let category_ref = (header_value >> 32) & 0xFFFF;
        assert_eq!(category_ref, 10);

        // Check name ref (bits 48-63 = 15)
        let name_ref = (header_value >> 48) & 0xFFFF;
        assert_eq!(name_ref, 15);

        // Verify the timestamp
        let timestamp = u64::from_le_bytes([
            buffer[8], buffer[9], buffer[10], buffer[11], buffer[12], buffer[13], buffer[14],
            buffer[15],
        ]);
        assert_eq!(timestamp, 1000000);

        Ok(())
    }

    #[test]
    fn test_counter_event_record_write() -> Result<()> {
        // Create a counter event
        let event = Event {
            timestamp: 1000000,
            thread: ThreadOrRef::Ref(1),
            category: StringOrRef::Ref(2),
            name: StringOrRef::Ref(3),
            arguments: Vec::new(),
        };

        let counter_record = EventRecord::Counter(Counter {
            event,
            counter_id: 42,
        });

        // Write it to a buffer
        let mut buffer = Vec::new();
        counter_record.write(&mut buffer)?;

        // Verify the length - should be 24 bytes (8 for header + 8 for timestamp + 8 for counter_id)
        assert_eq!(buffer.len(), 24);

        // Verify the header
        let header_value = u64::from_le_bytes([
            buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6], buffer[7],
        ]);

        // Check event type (bits 16-19 = 1 for Counter)
        let event_type = (header_value >> 16) & 0xF;
        assert_eq!(event_type, 1);

        // Verify the counter_id
        let counter_id = u64::from_le_bytes([
            buffer[16], buffer[17], buffer[18], buffer[19], buffer[20], buffer[21], buffer[22],
            buffer[23],
        ]);
        assert_eq!(counter_id, 42);

        Ok(())
    }

    #[test]
    fn test_duration_begin_event_record_write() -> Result<()> {
        // Create a duration begin event
        let event = Event {
            timestamp: 2000000,
            thread: ThreadOrRef::Ref(7),
            category: StringOrRef::Ref(12),
            name: StringOrRef::Ref(20),
            arguments: Vec::new(),
        };

        let duration_begin_record = EventRecord::DurationBegin(DurationBegin { event });

        // Write it to a buffer
        let mut buffer = Vec::new();
        duration_begin_record.write(&mut buffer)?;

        // Verify the length - should be 16 bytes (8 for header + 8 for timestamp)
        assert_eq!(buffer.len(), 16);

        // Verify the header
        let header_value = u64::from_le_bytes([
            buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6], buffer[7],
        ]);

        // Check event type (bits 16-19 = 2 for DurationBegin)
        let event_type = (header_value >> 16) & 0xF;
        assert_eq!(event_type, 2);

        // Verify the timestamp
        let timestamp = u64::from_le_bytes([
            buffer[8], buffer[9], buffer[10], buffer[11], buffer[12], buffer[13], buffer[14],
            buffer[15],
        ]);
        assert_eq!(timestamp, 2000000);

        Ok(())
    }

    #[test]
    fn test_duration_end_event_record_write() -> Result<()> {
        // Create a duration end event
        let event = Event {
            timestamp: 3000000,
            thread: ThreadOrRef::Ref(7),
            category: StringOrRef::Ref(12),
            name: StringOrRef::Ref(20),
            arguments: Vec::new(),
        };

        let duration_end_record = EventRecord::DurationEnd(DurationEnd { event });

        // Write it to a buffer
        let mut buffer = Vec::new();
        duration_end_record.write(&mut buffer)?;

        // Verify the length - should be 16 bytes (8 for header + 8 for timestamp)
        assert_eq!(buffer.len(), 16);

        // Verify the header
        let header_value = u64::from_le_bytes([
            buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6], buffer[7],
        ]);

        // Check event type (bits 16-19 = 3 for DurationEnd)
        let event_type = (header_value >> 16) & 0xF;
        assert_eq!(event_type, 3);

        // Verify the timestamp
        let timestamp = u64::from_le_bytes([
            buffer[8], buffer[9], buffer[10], buffer[11], buffer[12], buffer[13], buffer[14],
            buffer[15],
        ]);
        assert_eq!(timestamp, 3000000);

        Ok(())
    }

    #[test]
    fn test_duration_complete_event_record_write() -> Result<()> {
        // Create a duration complete event
        let event = Event {
            timestamp: 4000000,
            thread: ThreadOrRef::Ref(8),
            category: StringOrRef::Ref(15),
            name: StringOrRef::Ref(22),
            arguments: Vec::new(),
        };

        let duration_complete_record = EventRecord::DurationComplete(DurationComplete {
            event,
            duration_ticks: 500000,
        });

        // Write it to a buffer
        let mut buffer = Vec::new();
        duration_complete_record.write(&mut buffer)?;

        // Verify the length - should be 24 bytes (8 for header + 8 for timestamp + 8 for duration_ticks)
        assert_eq!(buffer.len(), 24);

        // Verify the header
        let header_value = u64::from_le_bytes([
            buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6], buffer[7],
        ]);

        // Check event type (bits 16-19 = 4 for DurationComplete)
        let event_type = (header_value >> 16) & 0xF;
        assert_eq!(event_type, 4);

        // Verify the duration_ticks
        let duration_ticks = u64::from_le_bytes([
            buffer[16], buffer[17], buffer[18], buffer[19], buffer[20], buffer[21], buffer[22],
            buffer[23],
        ]);
        assert_eq!(duration_ticks, 500000);

        Ok(())
    }

    #[test]
    fn test_event_record_write_with_inline_thread() -> Result<()> {
        // Create an event with inline thread
        let event = Event {
            timestamp: 1000000,
            thread: ThreadOrRef::ProcessAndThread(12345, 67890),
            category: StringOrRef::Ref(2),
            name: StringOrRef::Ref(3),
            arguments: Vec::new(),
        };

        let instant_record = EventRecord::Instant(Instant { event });

        // Write it to a buffer
        let mut buffer = Vec::new();
        instant_record.write(&mut buffer)?;

        // Verify the length - should be 32 bytes (8 for header + 8 for timestamp + 16 for process and thread IDs)
        assert_eq!(buffer.len(), 32);

        // Verify the header
        let header_value = u64::from_le_bytes([
            buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6], buffer[7],
        ]);

        // Check thread ref (bits 24-31 = 0 for inline thread)
        let thread_ref = (header_value >> 24) & 0xFF;
        assert_eq!(thread_ref, 0);

        // Verify process ID
        let process_id = u64::from_le_bytes([
            buffer[16], buffer[17], buffer[18], buffer[19], buffer[20], buffer[21], buffer[22],
            buffer[23],
        ]);
        assert_eq!(process_id, 12345);

        // Verify thread ID
        let thread_id = u64::from_le_bytes([
            buffer[24], buffer[25], buffer[26], buffer[27], buffer[28], buffer[29], buffer[30],
            buffer[31],
        ]);
        assert_eq!(thread_id, 67890);

        Ok(())
    }

    #[test]
    fn test_event_record_write_with_inline_category() -> Result<()> {
        // Create an event with inline category
        let event = Event {
            timestamp: 1000000,
            thread: ThreadOrRef::Ref(1),
            category: StringOrRef::String("cat".to_string()),
            name: StringOrRef::Ref(3),
            arguments: Vec::new(),
        };

        let instant_record = EventRecord::Instant(Instant { event });

        // Write it to a buffer
        let mut buffer = Vec::new();
        instant_record.write(&mut buffer)?;

        // Verify the length - should be 24 bytes (8 for header + 8 for timestamp + 8 for padded category string)
        assert_eq!(buffer.len(), 24);

        // Verify the header
        let header_value = u64::from_le_bytes([
            buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6], buffer[7],
        ]);

        // Check category field (bits 32-47 should have high bit set)
        let category_field = (header_value >> 32) & 0xFFFF;
        assert!(category_field & 0x8000 != 0); // High bit should be set for inline

        // Extract the category string
        let category_bytes = &buffer[16..24];
        let mut category_string = String::new();
        for &byte in category_bytes {
            if byte != 0 {
                category_string.push(byte as char);
            }
        }
        assert_eq!(category_string, "cat");

        Ok(())
    }

    #[test]
    fn test_event_record_write_with_inline_name() -> Result<()> {
        // Create an event with inline name
        let event = Event {
            timestamp: 1000000,
            thread: ThreadOrRef::Ref(1),
            category: StringOrRef::Ref(2),
            name: StringOrRef::String("test".to_string()),
            arguments: Vec::new(),
        };

        let instant_record = EventRecord::Instant(Instant { event });

        // Write it to a buffer
        let mut buffer = Vec::new();
        instant_record.write(&mut buffer)?;

        // Verify the length - should be 24 bytes (8 for header + 8 for timestamp + 8 for padded name string)
        assert_eq!(buffer.len(), 24);

        // Verify the header
        let header_value = u64::from_le_bytes([
            buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6], buffer[7],
        ]);

        // Check name field (bits 48-63 should have high bit set)
        let name_field = (header_value >> 48) & 0xFFFF;
        assert!(name_field & 0x8000 != 0); // High bit should be set for inline

        // Extract the name string
        let name_bytes = &buffer[16..24];
        let mut name_string = String::new();
        for &byte in name_bytes {
            if byte != 0 {
                name_string.push(byte as char);
            }
        }
        assert_eq!(name_string, "test");

        Ok(())
    }

    #[test]
    fn test_event_record_write_with_multiple_inline_fields() -> Result<()> {
        // Create an event with inline thread, category, and name
        let event = Event {
            timestamp: 5000000,
            thread: ThreadOrRef::ProcessAndThread(98765, 43210),
            category: StringOrRef::String("debug".to_string()),
            name: StringOrRef::String("operation".to_string()),
            arguments: Vec::new(),
        };

        let instant_record = EventRecord::Instant(Instant { event });

        // Write it to a buffer
        let mut buffer = Vec::new();
        instant_record.write(&mut buffer)?;

        // Verify the length - should be 48 bytes
        // (8 for header + 8 for timestamp + 16 for process and thread IDs + 8 for category + 16 for name)
        assert_eq!(buffer.len(), 56);

        // Verify the header
        let header_value = u64::from_le_bytes([
            buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6], buffer[7],
        ]);

        // Check thread ref (bits 24-31 = 0 for inline thread)
        let thread_ref = (header_value >> 24) & 0xFF;
        assert_eq!(thread_ref, 0);

        // Check category field (bits 32-47 should have high bit set)
        let category_field = (header_value >> 32) & 0xFFFF;
        assert!(category_field & 0x8000 != 0); // High bit should be set for inline

        // Check name field (bits 48-63 should have high bit set)
        let name_field = (header_value >> 48) & 0xFFFF;
        assert!(name_field & 0x8000 != 0); // High bit should be set for inline

        // Extract the strings from buffer
        // First process ID and thread ID
        let process_id = u64::from_le_bytes([
            buffer[16], buffer[17], buffer[18], buffer[19], buffer[20], buffer[21], buffer[22],
            buffer[23],
        ]);
        assert_eq!(process_id, 98765);

        let thread_id = u64::from_le_bytes([
            buffer[24], buffer[25], buffer[26], buffer[27], buffer[28], buffer[29], buffer[30],
            buffer[31],
        ]);
        assert_eq!(thread_id, 43210);

        // Then category string "debug"
        let category_bytes = &buffer[32..40];
        let mut category_string = String::new();
        for &byte in category_bytes {
            if byte != 0 {
                category_string.push(byte as char);
            }
        }
        assert_eq!(category_string, "debug");

        // Then name string "operation"
        let name_bytes = &buffer[40..48];
        let mut name_string = String::new();
        for &byte in name_bytes {
            if byte != 0 {
                name_string.push(byte as char);
            }
        }
        assert_eq!(name_string, "operation");

        Ok(())
    }

    #[test]
    fn test_instant_event_record_roundtrip() -> Result<()> {
        // Create an instant event with references
        let original_event = Event {
            timestamp: 1000000,
            thread: ThreadOrRef::Ref(5),
            category: StringOrRef::Ref(10),
            name: StringOrRef::Ref(15),
            arguments: Vec::new(),
        };

        let original_record = EventRecord::Instant(Instant {
            event: original_event,
        });

        // Write it to a buffer
        let mut buffer = Vec::new();
        original_record.write(&mut buffer)?;

        // Read it back
        let mut cursor = Cursor::new(&buffer);
        let record = Record::from_bytes(&mut cursor)?;

        // Verify it matches the original
        match record {
            Record::Event(EventRecord::Instant(instant)) => {
                assert_eq!(instant.event.timestamp, 1000000);
                assert_eq!(instant.event.thread, ThreadOrRef::Ref(5));
                assert_eq!(instant.event.category, StringOrRef::Ref(10));
                assert_eq!(instant.event.name, StringOrRef::Ref(15));
                assert!(instant.event.arguments.is_empty());
            }
            _ => panic!("Expected Instant event record, got {:?}", record),
        }

        Ok(())
    }

    #[test]
    fn test_counter_event_record_roundtrip() -> Result<()> {
        // Create a counter event
        let original_event = Event {
            timestamp: 1000000,
            thread: ThreadOrRef::Ref(1),
            category: StringOrRef::Ref(2),
            name: StringOrRef::Ref(3),
            arguments: Vec::new(),
        };

        let original_record = EventRecord::Counter(Counter {
            event: original_event,
            counter_id: 42,
        });

        // Write it to a buffer
        let mut buffer = Vec::new();
        original_record.write(&mut buffer)?;

        // Read it back
        let mut cursor = Cursor::new(&buffer);
        let record = Record::from_bytes(&mut cursor)?;

        // Verify it matches the original
        match record {
            Record::Event(EventRecord::Counter(counter)) => {
                assert_eq!(counter.event.timestamp, 1000000);
                assert_eq!(counter.event.thread, ThreadOrRef::Ref(1));
                assert_eq!(counter.event.category, StringOrRef::Ref(2));
                assert_eq!(counter.event.name, StringOrRef::Ref(3));
                assert_eq!(counter.counter_id, 42);
                assert!(counter.event.arguments.is_empty());
            }
            _ => panic!("Expected Counter event record, got {:?}", record),
        }

        Ok(())
    }

    #[test]
    fn test_duration_complete_event_record_roundtrip() -> Result<()> {
        // Create a duration complete event
        let original_event = Event {
            timestamp: 4000000,
            thread: ThreadOrRef::Ref(8),
            category: StringOrRef::Ref(15),
            name: StringOrRef::Ref(22),
            arguments: Vec::new(),
        };

        let original_record = EventRecord::DurationComplete(DurationComplete {
            event: original_event,
            duration_ticks: 500000,
        });

        // Write it to a buffer
        let mut buffer = Vec::new();
        original_record.write(&mut buffer)?;

        // Read it back
        let mut cursor = Cursor::new(&buffer);
        let record = Record::from_bytes(&mut cursor)?;

        // Verify it matches the original
        match record {
            Record::Event(EventRecord::DurationComplete(complete)) => {
                assert_eq!(complete.event.timestamp, 4000000);
                assert_eq!(complete.event.thread, ThreadOrRef::Ref(8));
                assert_eq!(complete.event.category, StringOrRef::Ref(15));
                assert_eq!(complete.event.name, StringOrRef::Ref(22));
                assert_eq!(complete.duration_ticks, 500000);
                assert!(complete.event.arguments.is_empty());
            }
            _ => panic!("Expected DurationComplete event record, got {:?}", record),
        }

        Ok(())
    }

    #[test]
    fn test_inline_fields_roundtrip() -> Result<()> {
        // Create an event with all inline fields
        let original_event = Event {
            timestamp: 5000000,
            thread: ThreadOrRef::ProcessAndThread(98765, 43210),
            category: StringOrRef::String("debug".to_string()),
            name: StringOrRef::String("operation".to_string()),
            arguments: Vec::new(),
        };

        let original_record = EventRecord::Instant(Instant {
            event: original_event,
        });

        // Write it to a buffer
        let mut buffer = Vec::new();
        original_record.write(&mut buffer)?;

        // Read it back
        let mut cursor = Cursor::new(&buffer);
        let record = Record::from_bytes(&mut cursor)?;

        // Verify it matches the original
        match record {
            Record::Event(EventRecord::Instant(instant)) => {
                assert_eq!(instant.event.timestamp, 5000000);
                assert_eq!(
                    instant.event.thread,
                    ThreadOrRef::ProcessAndThread(98765, 43210)
                );
                assert_eq!(
                    instant.event.category,
                    StringOrRef::String("debug".to_string())
                );
                assert_eq!(
                    instant.event.name,
                    StringOrRef::String("operation".to_string())
                );
                assert!(instant.event.arguments.is_empty());
            }
            _ => panic!("Expected Instant event record, got {:?}", record),
        }

        Ok(())
    }
}
