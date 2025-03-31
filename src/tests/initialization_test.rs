use crate::initialization::InitializationRecord;
use crate::header::RecordHeader;
use std::io::Cursor;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialization_record_parsing() {
        // Create header with:
        // - Record type: Initialization (bits 0-3 = 1)
        // - Size: 2 (bits 4-15) - 2 * 8 = 16 bytes (8 for header + 8 for ticks_per_second)
        
        let header_value: u64 = 0
            | (2 << 4)    // Size (2 * 8 = 16 bytes)
            | 1;          // Record type (Initialization)
            
        let header = RecordHeader { value: header_value };
        
        // Create test data with ticks_per_second value
        let ticks_per_second: u64 = 10_000_000; // 10 MHz example
        let data = ticks_per_second.to_le_bytes();
        let mut cursor = Cursor::new(data);
        
        let record = InitializationRecord::parse(&mut cursor, header).unwrap();
        
        assert_eq!(record.ticks_per_second, ticks_per_second);
    }
}