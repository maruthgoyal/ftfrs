use crate::threadrec::ThreadRecord;
use crate::header::RecordHeader;
use std::io::Cursor;

#[cfg(test)]
#[allow(clippy::identity_op)]
mod tests {
    use super::*;

    #[test]
    fn test_thread_record_parsing() {
        // Create header with:
        // - Record type: Thread (bits 0-3 = 3)
        // - Size: 3 (bits 4-15) - 3 * 8 = 24 bytes (8 for header + 16 for two 8-byte KOIDs)
        // - Thread index: 5 (bits 16-23)
        
        let header_value: u64 = 0
            | (5 << 16)   // Thread index
            | (3 << 4)    // Size (3 * 8 = 24 bytes)
            | 3;          // Record type (Thread)
            
        let header = RecordHeader { value: header_value };
        
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
}