use crate::header::RecordHeader;
use crate::metadata::MetadataRecord;
use std::io::Cursor;

#[cfg(test)]
#[allow(clippy::identity_op)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_magic_number() {
        // Create a cursor with the magic number
        let header = RecordHeader {
            value: MetadataRecord::MAGIC_NUMBER_RECORD,
        };

        // Use explicit type for Vec
        let record = MetadataRecord::parse(&mut Cursor::new(Vec::<u8>::new()), header).unwrap();

        match record {
            MetadataRecord::MagicNumber => {} // Success
            _ => panic!("Expected MagicNumber record, got {:?}", record),
        }
    }

    #[test]
    fn test_provider_info_parsing() {
        // Create header with:
        // - Record type: Metadata (bits 0-3 = 0)
        // - Size: 16 (bits 4-15)
        // - Metadata type: ProviderInfo (bits 16-19 = 1)
        // - Provider ID: 42 (bits 20-51)
        // - Name length: 8 (bits 52-59)

        let header_value: u64 = 0
            | (8 << 52)   // Name length (8 bytes)
            | (42 << 20)  // Provider ID
            | (1 << 16)   // Metadata type (ProviderInfo)
            | (2 << 4)   // Size (16 bytes)
            | 0; // Record type (Metadata)

        let header = RecordHeader {
            value: header_value,
        };

        // Create test data with "test_lib" as provider name (8 bytes)
        let data = b"test_lib";
        let mut cursor = Cursor::new(data);

        let record = MetadataRecord::parse(&mut cursor, header).unwrap();

        match record {
            MetadataRecord::ProviderInfo(info) => {
                assert_eq!(info.provider_id, 42);
                assert_eq!(info.provider_name, "test_lib");
            }
            _ => panic!("Expected ProviderInfo record, got {:?}", record),
        }
    }

    #[test]
    fn test_provider_section_parsing() {
        // Create header with:
        // - Record type: Metadata (bits 0-3 = 0)
        // - Size: 8 (bits 4-15)
        // - Metadata type: ProviderSection (bits 16-19 = 2)
        // - Provider ID: 123 (bits 20-51)

        let header_value: u64 = 0
            | (123 << 20) // Provider ID
            | (2 << 16)   // Metadata type (ProviderSection)
            | (8 << 4)    // Size (8 bytes)
            | 0; // Record type (Metadata)

        let header = RecordHeader {
            value: header_value,
        };
        let mut cursor = Cursor::new(Vec::new()); // No additional data needed

        let record = MetadataRecord::parse(&mut cursor, header).unwrap();

        match record {
            MetadataRecord::ProviderSection(section) => {
                assert_eq!(section.provider_id, 123);
            }
            _ => panic!("Expected ProviderSection record, got {:?}", record),
        }
    }

    #[test]
    fn test_provider_event_parsing() {
        // Create header with:
        // - Record type: Metadata (bits 0-3 = 0)
        // - Size: 8 (bits 4-15)
        // - Metadata type: ProviderEvent (bits 16-19 = 3)
        // - Provider ID: 456 (bits 20-51)
        // - Event ID: 7 (bits 52-55)

        let header_value: u64 = 0
            | (7 << 52)    // Event ID
            | (456 << 20)  // Provider ID
            | (3 << 16)    // Metadata type (ProviderEvent)
            | (8 << 4)     // Size (8 bytes)
            | 0; // Record type (Metadata)

        let header = RecordHeader {
            value: header_value,
        };
        let mut cursor = Cursor::new(Vec::new()); // No additional data needed

        let record = MetadataRecord::parse(&mut cursor, header).unwrap();

        match record {
            MetadataRecord::ProviderEvent(event) => {
                assert_eq!(event.provider_id, 456);
                assert_eq!(event.event_id, 7);
            }
            _ => panic!("Expected ProviderEvent record, got {:?}", record),
        }
    }

    #[test]
    fn test_trace_info_parsing() {
        // Create header with:
        // - Record type: Metadata (bits 0-3 = 0)
        // - Size: 8 (bits 4-15)
        // - Metadata type: TraceInfo (bits 16-19 = 4)
        // - Trace info type: 3 (bits 20-23)
        // - Data: 0xABCDEF (bits 24-63, arbitrary test value)

        let header_value: u64 = 0
            | (0xABCDEF << 24) // Data
            | (3 << 20)        // Trace info type
            | (4 << 16)        // Metadata type (TraceInfo)
            | (8 << 4)         // Size (8 bytes)
            | 0; // Record type (Metadata)

        let header = RecordHeader {
            value: header_value,
        };
        let mut cursor = Cursor::new(Vec::new()); // No additional data needed

        let record = MetadataRecord::parse(&mut cursor, header).unwrap();

        match record {
            MetadataRecord::TraceInfo(info) => {
                assert_eq!(info.trace_info_type, 3);
                assert_eq!(info.data, 0xABCDEF);
            }
            _ => panic!("Expected TraceInfo record, got {:?}", record),
        }
    }
}
