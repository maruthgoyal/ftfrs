#![warn(missing_docs)]
use crate::{header::RecordHeader, wordutils::read_u64_word, Result};
use std::io::{Read, Write};

/// Initialization record
/// Specifies number of ticks per second in this trace
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InitializationRecord {
    ticks_per_second: u64,
}

impl InitializationRecord {
    pub(super) fn new(ticks_per_second: u64) -> Self {
        Self { ticks_per_second }
    }

    /// Number of ticks in a second for this trace
    pub fn ticks_per_second(&self) -> u64 {
        self.ticks_per_second
    }

    pub(super) fn parse<U: Read>(reader: &mut U, _header: RecordHeader) -> Result<Self> {
        Ok(InitializationRecord {
            ticks_per_second: read_u64_word(reader)?,
        })
    }

    pub(super) fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        let header = RecordHeader::build(crate::header::RecordType::Initialization, 2, &[])?;
        writer.write_all(&header.value.to_le_bytes())?;
        writer.write_all(&self.ticks_per_second.to_le_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::identity_op)]
mod tests {
    use super::*;
    use crate::Record;
    use crate::Result;
    use std::io::Cursor;

    #[test]
    fn test_initialization_record_parsing() {
        // Create header with:
        // - Record type: Initialization (bits 0-3 = 1)
        // - Size: 2 (bits 4-15) - 2 * 8 = 16 bytes (8 for header + 8 for ticks_per_second)

        let header_value: u64 = 0
            | (2 << 4)    // Size (2 * 8 = 16 bytes)
            | 1; // Record type (Initialization)

        let header = RecordHeader {
            value: header_value,
        };

        // Create test data with ticks_per_second value
        let ticks_per_second: u64 = 10_000_000; // 10 MHz example
        let data = ticks_per_second.to_le_bytes();
        let mut cursor = Cursor::new(data);

        let record = InitializationRecord::parse(&mut cursor, header).unwrap();

        assert_eq!(record.ticks_per_second, ticks_per_second);
    }

    #[test]
    fn test_initialization_record_write() -> Result<()> {
        // Create an initialization record
        let record = InitializationRecord {
            ticks_per_second: 10_000_000, // 10 MHz
        };

        // Write it to a buffer
        let mut buffer = Vec::new();
        record.write(&mut buffer)?;

        // Verify the length
        assert_eq!(buffer.len(), 16); // 8 bytes header + 8 bytes for ticks_per_second

        // Verify the header
        let header_value = u64::from_ne_bytes([
            buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6], buffer[7],
        ]);
        let header = RecordHeader {
            value: header_value,
        };

        assert_eq!(
            header.record_type()?,
            crate::header::RecordType::Initialization
        );
        assert_eq!(header.size() * 8, 16); // 2 words * 8 bytes

        // Verify the data
        let ticks_value = u64::from_ne_bytes([
            buffer[8], buffer[9], buffer[10], buffer[11], buffer[12], buffer[13], buffer[14],
            buffer[15],
        ]);
        assert_eq!(ticks_value, 10_000_000);

        Ok(())
    }

    #[test]
    fn test_initialization_record_roundtrip() -> Result<()> {
        // Create an initialization record
        let original_record = InitializationRecord {
            ticks_per_second: 12_345_678,
        };

        // Write it to a buffer
        let mut buffer = Vec::new();
        original_record.write(&mut buffer)?;

        // Read it back
        let mut cursor = Cursor::new(&buffer);
        let record = Record::read(&mut cursor)?;

        // Verify it matches the original
        match record {
            Record::Initialization(parsed_record) => {
                assert_eq!(
                    parsed_record.ticks_per_second,
                    original_record.ticks_per_second
                );
            }
            _ => panic!("Expected Initialization record, got {:?}", record),
        }

        Ok(())
    }
}
