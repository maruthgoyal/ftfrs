use std::io::{Read, Write};
use thiserror::Error;

use crate::{
    extract_bits,
    header::CustomField,
    wordutils::{self, pad_to_multiple_of_8},
    RecordHeader, Result,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TraceInfo {
    trace_info_type: u8,
    // only 40 bits, but no point in encoding as [u8; 5]
    data: u64,
}

impl TraceInfo {
    pub fn new(trace_info_type: u8, data: &[u8; 5]) -> Self {
        let mut tmp = [0_u8; 8];
        tmp[3] = data[0];
        tmp[4] = data[1];
        tmp[5] = data[2];
        tmp[6] = data[3];
        tmp[7] = data[4];

        Self {
            trace_info_type,
            data: u64::from_be_bytes(tmp),
        }
    }
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        let header = RecordHeader::build(
            crate::header::RecordType::Metadata,
            1,
            vec![
                CustomField {
                    width: 4,
                    value: MetadataType::TraceInfo as u64,
                },
                CustomField {
                    width: 4,
                    value: self.trace_info_type as u64,
                },
                CustomField {
                    width: 40,
                    value: self.data,
                },
            ],
        )?;

        writer.write_all(&header.value.to_le_bytes())?;

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderInfo {
    provider_id: u32,
    provider_name: String,
}

impl ProviderInfo {
    pub fn new(provider_id: u32, provider_name: String) -> Self {
        Self {
            provider_id,
            provider_name,
        }
    }

    pub fn provider_id(&self) -> u32 {
        self.provider_id
    }

    pub fn provider_name(&self) -> &String {
        &self.provider_name
    }

    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        let str_bytes = self.provider_name.as_bytes();
        let size = 1 + (str_bytes.len() + 7) / 8;

        let header = RecordHeader::build(
            crate::header::RecordType::Metadata,
            size as u8,
            vec![
                CustomField {
                    width: 4,
                    value: MetadataType::ProviderInfo as u64,
                },
                CustomField {
                    width: 32,
                    value: self.provider_id as u64,
                },
                CustomField {
                    width: 8,
                    value: self.provider_name.len() as u64,
                },
            ],
        )?;

        writer.write_all(&header.value.to_le_bytes())?;

        let padded = pad_to_multiple_of_8(self.provider_name.as_bytes());
        writer.write_all(&padded)?;

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProviderSection {
    provider_id: u32,
}
impl ProviderSection {
    pub fn new(provider_id: u32) -> Self {
        Self { provider_id }
    }

    pub fn provider_id(&self) -> u32 {
        self.provider_id
    }

    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        let header = RecordHeader::build(
            crate::header::RecordType::Metadata,
            1,
            vec![
                CustomField {
                    width: 4,
                    value: MetadataType::ProviderSection as u64,
                },
                CustomField {
                    width: 32,
                    value: self.provider_id as u64,
                },
            ],
        )?;

        writer.write_all(&header.value.to_le_bytes())?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProviderEvent {
    provider_id: u32,
    event_id: u8,
}

impl ProviderEvent {
    pub fn new(provider_id: u32, event_id: u8) -> Self {
        Self {
            provider_id,
            event_id,
        }
    }

    pub fn provider_id(&self) -> u32 {
        self.provider_id
    }

    pub fn event_id(&self) -> u8 {
        self.event_id
    }

    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        let header = RecordHeader::build(
            crate::header::RecordType::Metadata,
            1,
            vec![
                CustomField {
                    width: 4,
                    value: MetadataType::ProviderEvent as u64,
                },
                CustomField {
                    width: 32,
                    value: self.provider_id as u64,
                },
                CustomField {
                    width: 4,
                    value: self.event_id as u64,
                },
            ],
        )?;

        writer.write_all(&header.value.to_le_bytes())?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MetadataType {
    ProviderInfo = 1,
    ProviderSection = 2,
    ProviderEvent = 3,
    TraceInfo = 4,
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
#[error("Invalid metadata type {0}")]
pub struct MetadataTypeParseError(u8);

impl TryFrom<u8> for MetadataType {
    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::ProviderInfo),
            2 => Ok(Self::ProviderSection),
            3 => Ok(Self::ProviderEvent),
            4 => Ok(Self::TraceInfo),
            _ => Err(MetadataTypeParseError(value)),
        }
    }

    type Error = MetadataTypeParseError;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetadataRecord {
    ProviderInfo(ProviderInfo),
    ProviderSection(ProviderSection),
    ProviderEvent(ProviderEvent),
    TraceInfo(TraceInfo),
    MagicNumber,
}

impl MetadataRecord {
    pub const MAGIC_NUMBER_RECORD: u64 = 0x0016547846040010;

    fn metadata_type(header: &RecordHeader) -> Result<MetadataType> {
        let ty = extract_bits!(header.value, 16, 19) as u8;
        Ok(MetadataType::try_from(ty)?)
    }

    #[inline]
    fn provider_id(header: &RecordHeader) -> u32 {
        extract_bits!(header.value, 20, 51) as u32
    }

    pub(super) fn parse<U: Read>(reader: &mut U, header: RecordHeader) -> Result<Self> {
        if header.value == Self::MAGIC_NUMBER_RECORD {
            return Ok(Self::MagicNumber);
        }

        match MetadataRecord::metadata_type(&header)? {
            MetadataType::ProviderInfo => {
                let provider_id = Self::provider_id(&header);
                let namelen = extract_bits!(header.value, 52, 59) as usize;

                let provider_name = wordutils::read_aligned_str(reader, namelen)?;

                Ok(Self::ProviderInfo(ProviderInfo {
                    provider_id,
                    provider_name,
                }))
            }
            MetadataType::ProviderSection => {
                let provider_id = Self::provider_id(&header);
                Ok(Self::ProviderSection(ProviderSection { provider_id }))
            }
            MetadataType::ProviderEvent => {
                let provider_id = Self::provider_id(&header);
                let event_id = extract_bits!(header.value, 52, 55) as u8;
                Ok(Self::ProviderEvent(ProviderEvent {
                    provider_id,
                    event_id,
                }))
            }
            MetadataType::TraceInfo => {
                let trace_info_type = extract_bits!(header.value, 20, 23) as u8;
                let data = extract_bits!(header.value, 24, 63);
                Ok(Self::TraceInfo(TraceInfo {
                    trace_info_type,
                    data,
                }))
            }
        }
    }
    pub(super) fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        match self {
            MetadataRecord::MagicNumber => {
                writer.write_all(&Self::MAGIC_NUMBER_RECORD.to_le_bytes())?;
            }
            MetadataRecord::ProviderEvent(e) => e.write(writer)?,
            MetadataRecord::ProviderInfo(e) => e.write(writer)?,
            MetadataRecord::ProviderSection(e) => e.write(writer)?,
            MetadataRecord::TraceInfo(e) => e.write(writer)?,
        }
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

    #[test]
    fn test_magic_number_write() -> Result<()> {
        // Create a magic number record
        let record = MetadataRecord::MagicNumber;

        // Write it to a buffer
        let mut buffer = Vec::new();
        record.write(&mut buffer)?;

        // Verify the length - should be only 8 bytes for the magic number constant
        assert_eq!(buffer.len(), 8);

        // Verify the value
        let value = u64::from_ne_bytes([
            buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6], buffer[7],
        ]);
        assert_eq!(value, MetadataRecord::MAGIC_NUMBER_RECORD);

        Ok(())
    }

    #[test]
    fn test_provider_info_write() -> Result<()> {
        // Create a provider info record
        let provider_info = ProviderInfo {
            provider_id: 42,
            provider_name: "test_lib".to_string(),
        };

        let record = MetadataRecord::ProviderInfo(provider_info.clone());

        // Write it to a buffer
        let mut buffer = Vec::new();
        record.write(&mut buffer)?;

        // Verify the length: 8 bytes header + 8 bytes for "test_lib" (aligned to 8)
        assert_eq!(buffer.len(), 16);

        // Verify the header
        let header_value = u64::from_ne_bytes([
            buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6], buffer[7],
        ]);
        let header = RecordHeader {
            value: header_value,
        };

        assert_eq!(header.record_type()?, crate::header::RecordType::Metadata);

        // Extract the string
        let name_bytes = &buffer[8..16];
        let name = String::from_utf8(name_bytes[0..8].to_vec())?;
        assert_eq!(name, "test_lib");

        Ok(())
    }

    #[test]
    fn test_provider_section_write() -> Result<()> {
        // Create a provider section record
        let provider_section = ProviderSection { provider_id: 123 };

        let record = MetadataRecord::ProviderSection(provider_section);

        // Write it to a buffer
        let mut buffer = Vec::new();
        record.write(&mut buffer)?;

        // Verify the length: 8 bytes header only (no additional data)
        assert_eq!(buffer.len(), 8);

        // Verify the header
        let header_value = u64::from_ne_bytes([
            buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6], buffer[7],
        ]);

        // Check metadata type (bits 16-19 = 2 for ProviderSection)
        let metadata_type = (header_value >> 16) & 0xF;
        assert_eq!(metadata_type, 2);

        // Check provider ID (bits 20-51)
        let provider_id = (header_value >> 20) & 0xFFFFFFFF;
        assert_eq!(provider_id, 123);

        Ok(())
    }

    #[test]
    fn test_provider_event_write() -> Result<()> {
        // Create a provider event record
        let provider_event = ProviderEvent {
            provider_id: 456,
            event_id: 7,
        };

        let record = MetadataRecord::ProviderEvent(provider_event);

        // Write it to a buffer
        let mut buffer = Vec::new();
        record.write(&mut buffer)?;

        // Verify the length: 8 bytes header only (no additional data)
        assert_eq!(buffer.len(), 8);

        // Verify the header
        let header_value = u64::from_ne_bytes([
            buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6], buffer[7],
        ]);

        // Check metadata type (bits 16-19 = 3 for ProviderEvent)
        let metadata_type = (header_value >> 16) & 0xF;
        assert_eq!(metadata_type, 3);

        // Check provider ID (bits 20-51)
        let provider_id = (header_value >> 20) & 0xFFFFFFFF;
        assert_eq!(provider_id, 456);

        // Check event ID (bits 52-55)
        let event_id = (header_value >> 52) & 0xF;
        assert_eq!(event_id, 7);

        Ok(())
    }

    #[test]
    fn test_trace_info_write() -> Result<()> {
        // Create a trace info record
        let trace_info = TraceInfo {
            trace_info_type: 3,
            data: 0xABCDEF,
        };

        let record = MetadataRecord::TraceInfo(trace_info);

        // Write it to a buffer
        let mut buffer = Vec::new();
        record.write(&mut buffer)?;

        // Verify the length: 8 bytes header only (no additional data)
        assert_eq!(buffer.len(), 8);

        // Verify the header
        let header_value = u64::from_ne_bytes([
            buffer[0], buffer[1], buffer[2], buffer[3], buffer[4], buffer[5], buffer[6], buffer[7],
        ]);

        // Check metadata type (bits 16-19 = 4 for TraceInfo)
        let metadata_type = (header_value >> 16) & 0xF;
        assert_eq!(metadata_type, 4);

        // Check trace info type (bits 20-23)
        let info_type = (header_value >> 20) & 0xF;
        assert_eq!(info_type, 3);

        // Check data (bits 24-63)
        let data = (header_value >> 24) & 0xFFFFFFFFFF;
        assert_eq!(data, 0xABCDEF);

        Ok(())
    }

    #[test]
    fn test_metadata_record_roundtrip() -> Result<()> {
        // Test all types of metadata records for roundtrip

        // 1. Magic Number
        let mut buffer = Vec::new();
        MetadataRecord::MagicNumber.write(&mut buffer)?;

        let mut cursor = Cursor::new(&buffer);
        let record = Record::from_bytes(&mut cursor)?;
        assert!(matches!(
            record,
            Record::Metadata(MetadataRecord::MagicNumber)
        ));

        // 2. Provider Info
        let provider_info = ProviderInfo {
            provider_id: 42,
            provider_name: "test_lib".to_string(),
        };

        buffer.clear();
        MetadataRecord::ProviderInfo(provider_info.clone()).write(&mut buffer)?;

        let mut cursor = Cursor::new(&buffer);
        let record = Record::from_bytes(&mut cursor)?;

        if let Record::Metadata(MetadataRecord::ProviderInfo(parsed_info)) = record {
            assert_eq!(parsed_info.provider_id, provider_info.provider_id);
            assert_eq!(parsed_info.provider_name, provider_info.provider_name);
        } else {
            panic!("Expected ProviderInfo record, got {:?}", record);
        }

        // 3. Provider Section
        let provider_section = ProviderSection { provider_id: 123 };

        buffer.clear();
        MetadataRecord::ProviderSection(provider_section).write(&mut buffer)?;

        let mut cursor = Cursor::new(&buffer);
        let record = Record::from_bytes(&mut cursor)?;

        if let Record::Metadata(MetadataRecord::ProviderSection(parsed_section)) = record {
            assert_eq!(parsed_section.provider_id, provider_section.provider_id);
        } else {
            panic!("Expected ProviderSection record, got {:?}", record);
        }

        // 4. Provider Event
        let provider_event = ProviderEvent {
            provider_id: 456,
            event_id: 7,
        };

        buffer.clear();
        MetadataRecord::ProviderEvent(provider_event).write(&mut buffer)?;

        let mut cursor = Cursor::new(&buffer);
        let record = Record::from_bytes(&mut cursor)?;

        if let Record::Metadata(MetadataRecord::ProviderEvent(parsed_event)) = record {
            assert_eq!(parsed_event.provider_id, provider_event.provider_id);
            assert_eq!(parsed_event.event_id, provider_event.event_id);
        } else {
            panic!("Expected ProviderEvent record, got {:?}", record);
        }

        // 5. Trace Info
        let trace_info = TraceInfo {
            trace_info_type: 3,
            data: 0xABCDEF,
        };

        buffer.clear();
        MetadataRecord::TraceInfo(trace_info).write(&mut buffer)?;

        let mut cursor = Cursor::new(&buffer);
        let record = Record::from_bytes(&mut cursor)?;

        if let Record::Metadata(MetadataRecord::TraceInfo(parsed_info)) = record {
            assert_eq!(parsed_info.trace_info_type, trace_info.trace_info_type);
            assert_eq!(parsed_info.data, trace_info.data);
        } else {
            panic!("Expected TraceInfo record, got {:?}", record);
        }

        Ok(())
    }
}
