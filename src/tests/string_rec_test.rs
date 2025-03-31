use crate::header::RecordHeader;
use crate::string_rec::StringRecord;
use std::io::Cursor;

#[cfg(test)]
#[allow(clippy::identity_op)]
mod tests {
    use super::*;
    use crate::Result;

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
}
