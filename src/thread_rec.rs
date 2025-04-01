use crate::{extract_bits, header::CustomField, wordutils::read_u64_word, RecordHeader, Result};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThreadRecord {
    index: u8,
    process_koid: u64,
    thread_koid: u64,
}

impl ThreadRecord {
    pub fn new(index: u8, process_koid: u64, thread_koid: u64) -> Self {
        ThreadRecord {
            index,
            process_koid,
            thread_koid,
        }
    }

    pub fn index(&self) -> u8 {
        self.index
    }

    pub fn process_koid(&self) -> u64 {
        self.process_koid
    }

    pub fn thread_koid(&self) -> u64 {
        self.thread_koid
    }

    pub(super) fn parse<U: Read>(reader: &mut U, header: RecordHeader) -> Result<Self> {
        let index = extract_bits!(header.value, 16, 23) as u8;

        let process_koid = read_u64_word(reader)?;
        let thread_koid = read_u64_word(reader)?;

        Ok(ThreadRecord {
            index,
            process_koid,
            thread_koid,
        })
    }

    pub(super) fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        let header = RecordHeader::build(
            crate::header::RecordType::Thread,
            3,
            vec![CustomField {
                width: 8,
                value: self.index as u64,
            }],
        )?;

        writer.write_all(&header.value.to_le_bytes())?;
        writer.write_all(&self.process_koid.to_le_bytes())?;
        writer.write_all(&self.thread_koid.to_le_bytes())?;

        Ok(())
    }
}
#[cfg(test)]
#[allow(clippy::identity_op)]
mod tests {
    use super::*;
    use crate::{Record, Result};
    use std::io::Cursor;

    #[test]
    fn test_thread_record_parsing() {
        // Create header with:
        // - Record type: Thread (bits 0-3 = 3)
        // - Size: 3 (bits 4-15) - 3 * 8 = 24 bytes (8 for header + 16 for two 8-byte KOIDs)
        // - Thread index: 5 (bits 16-23)

        let header_value: u64 = 0
            | (5 << 16)   // Thread index
            | (3 << 4)    // Size (3 * 8 = 24 bytes)
            | 3; // Record type (Thread)

        let header = RecordHeader {
            value: header_value,
        };

        // Create test data with process_koid and thread_koid
        let process_koid: u64 = 12345;
        let thread_koid: u64 = 67890;

        let mut data = Vec::new();
        data.extend_from_slice(&process_koid.to_le_bytes());
        data.extend_from_slice(&thread_koid.to_le_bytes());

        let mut cursor = Cursor::new(data);

        let record = ThreadRecord::parse(&mut cursor, header).unwrap();

        assert_eq!(record.index, 5);
        assert_eq!(record.process_koid, 12345);
        assert_eq!(record.thread_koid, 67890);
    }

    #[test]
    fn test_thread_record_write() -> Result<()> {
        // Create a thread record
        let record = ThreadRecord {
            index: 5,
            process_koid: 12345,
            thread_koid: 67890,
        };

        // Write it to a buffer
        let mut buffer = Vec::new();
        record.write(&mut buffer)?;

        // Verify the length: 8 bytes header + 16 bytes for two KOIDs
        assert_eq!(buffer.len(), 24);

        // Verify the header
        let header_value = u64::from_le_bytes([
            buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6], buffer[7],
        ]);
        let header = RecordHeader {
            value: header_value,
        };

        assert_eq!(header.record_type()?, crate::header::RecordType::Thread);
        assert_eq!(header.size() * 8, 24); // 3 words * 8 bytes

        // Verify the thread index in the header
        let thread_index = (header_value >> 16) & 0xFF;
        assert_eq!(thread_index, 5);

        // Verify the process KOID
        let process_koid = u64::from_le_bytes([
            buffer[8], buffer[9], buffer[10], buffer[11], buffer[12], buffer[13], buffer[14],
            buffer[15],
        ]);
        assert_eq!(process_koid, 12345);

        // Verify the thread KOID
        let thread_koid = u64::from_le_bytes([
            buffer[16], buffer[17], buffer[18], buffer[19], buffer[20], buffer[21], buffer[22],
            buffer[23],
        ]);
        assert_eq!(thread_koid, 67890);

        Ok(())
    }

    #[test]
    fn test_thread_record_write_large_values() -> Result<()> {
        // Test with very large KOID values
        let record = ThreadRecord {
            index: 255, // Max u8 value
            process_koid: u64::MAX - 10,
            thread_koid: u64::MAX,
        };

        // Write it to a buffer
        let mut buffer = Vec::new();
        record.write(&mut buffer)?;

        // Verify the process KOID
        let process_koid = u64::from_le_bytes([
            buffer[8], buffer[9], buffer[10], buffer[11], buffer[12], buffer[13], buffer[14],
            buffer[15],
        ]);
        assert_eq!(process_koid, u64::MAX - 10);

        // Verify the thread KOID
        let thread_koid = u64::from_le_bytes([
            buffer[16], buffer[17], buffer[18], buffer[19], buffer[20], buffer[21], buffer[22],
            buffer[23],
        ]);
        assert_eq!(thread_koid, u64::MAX);

        Ok(())
    }

    #[test]
    fn test_thread_record_roundtrip() -> Result<()> {
        // Create a thread record
        let original_record = ThreadRecord {
            index: 42,
            process_koid: 987654321,
            thread_koid: 123456789,
        };

        // Write it to a buffer
        let mut buffer = Vec::new();
        original_record.write(&mut buffer)?;

        // Read it back
        let mut cursor = Cursor::new(&buffer);
        let record = Record::from_bytes(&mut cursor)?;

        // Verify it matches the original
        match record {
            Record::Thread(parsed_record) => {
                assert_eq!(parsed_record.index, original_record.index);
                assert_eq!(parsed_record.process_koid, original_record.process_koid);
                assert_eq!(parsed_record.thread_koid, original_record.thread_koid);
            }
            _ => panic!("Expected Thread record, got {:?}", record),
        }

        Ok(())
    }
}
