use crate::header::CustomField;
use crate::wordutils::{self, pad_to_multiple_of_8};
use crate::{extract_bits, RecordHeader, Result};
use std::io::{Read, Write};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StringRecord {
    index: u16,
    length: u32,
    value: String,
}

impl StringRecord {
    pub fn new(index: u16, length: u32, value: String) -> Self {
        Self {
            index,
            length,
            value,
        }
    }

    pub fn index(&self) -> u16 {
        self.index
    }

    pub fn length(&self) -> u32 {
        self.length
    }

    pub fn value(&self) -> &String {
        &self.value
    }

    pub(super) fn parse<U: Read>(reader: &mut U, header: RecordHeader) -> Result<Self> {
        let index = extract_bits!(header.value, 16, 30) as u16;
        let length = extract_bits!(header.value, 32, 46) as u32;

        let value = wordutils::read_aligned_str(reader, length as usize)?;
        Ok(StringRecord {
            index,
            length,
            value,
        })
    }

    pub(super) fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        let str_bytes = self.value.as_bytes();
        // header + num words for string
        let num_words = 1 + ((str_bytes.len() + 7) / 8);
        let header = RecordHeader::build(
            crate::header::RecordType::String,
            num_words as u8,
            vec![
                CustomField {
                    width: 15,
                    value: self.index as u64,
                },
                CustomField { width: 1, value: 0 },
                CustomField {
                    width: 15,
                    value: self.length as u64,
                },
            ],
        )?;

        writer.write_all(&header.value.to_le_bytes())?;

        let padded = pad_to_multiple_of_8(str_bytes);
        writer.write_all(&padded)?;

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
    fn test_string_record_parsing() -> Result<()> {
        // Create header with:
        // - Record type: String (bits 0-3 = 2)
        // - Size: 3 (bits 4-15) - 3 * 8 = 24 bytes (8 for header + 16 for aligned string)
        // - String index: 42 (bits 16-30)
        // - Length: 11 (bits 32-46)

        let header_value: u64 = 0
            | (11 << 32)  // Length (11 bytes)
            | (42 << 16)  // String index
            | (3 << 4)    // Size (3 * 8 = 24 bytes)
            | 2; // Record type (String)

        let header = RecordHeader {
            value: header_value,
        };

        // Create test data with "Hello World" string
        let data = b"Hello World\0\0\0\0\0"; // Padded to 16 bytes (multiple of 8)
        let mut cursor = Cursor::new(data);

        let record = StringRecord::parse(&mut cursor, header)?;

        assert_eq!(record.index, 42);
        assert_eq!(record.length, 11);
        assert_eq!(record.value, "Hello World");

        Ok(())
    }

    #[test]
    fn test_string_record_write() -> Result<()> {
        // Create a string record
        let record = StringRecord {
            index: 42,
            length: 11,
            value: "Hello World".to_string(),
        };

        // Write it to a buffer
        let mut buffer = Vec::new();
        record.write(&mut buffer)?;

        // Verify the length: 8 bytes header + 16 bytes for padded string
        assert_eq!(buffer.len(), 24);

        // Verify the header
        let header_value = u64::from_le_bytes([
            buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6], buffer[7],
        ]);
        let header = RecordHeader {
            value: header_value,
        };

        assert_eq!(header.record_type()?, crate::header::RecordType::String);
        assert_eq!(header.size() * 8, 24); // 3 words * 8 bytes

        // Verify the string data
        let mut string_data = Vec::new();
        for i in 8..24 {
            string_data.push(buffer[i]);
        }

        // Extract the string (removing padding)
        let string = String::from_utf8(string_data[0..11].to_vec())?;
        assert_eq!(string, "Hello World");

        // Verify padding
        for i in 11..16 {
            assert_eq!(string_data[i], 0, "Expected padding byte to be 0");
        }

        Ok(())
    }

    #[test]
    fn test_string_record_write_exact_multiple_of_8() -> Result<()> {
        // Test with a string whose length is exactly a multiple of 8
        let record = StringRecord {
            index: 100,
            length: 8,
            value: "ABCDEFGH".to_string(),
        };

        // Write it to a buffer
        let mut buffer = Vec::new();
        record.write(&mut buffer)?;

        // Verify length: 8 bytes header + 8 bytes for string (exact multiple of 8)
        assert_eq!(buffer.len(), 16);

        // Extract the string
        let string_data = &buffer[8..16];
        let string = String::from_utf8(string_data.to_vec())?;
        assert_eq!(string, "ABCDEFGH");

        Ok(())
    }

    #[test]
    fn test_string_record_roundtrip() -> Result<()> {
        // Create a string record
        let original_record = StringRecord {
            index: 123,
            length: 13,
            value: "Test String!!".to_string(),
        };

        // Write it to a buffer
        let mut buffer = Vec::new();
        original_record.write(&mut buffer)?;

        // Read it back
        let mut cursor = Cursor::new(&buffer);
        let record = Record::from_bytes(&mut cursor)?;

        // Verify it matches the original
        match record {
            Record::String(parsed_record) => {
                assert_eq!(parsed_record.index, original_record.index);
                assert_eq!(parsed_record.length, original_record.length);
                assert_eq!(parsed_record.value, original_record.value);
            }
            _ => panic!("Expected String record, got {:?}", record),
        }

        Ok(())
    }
}
