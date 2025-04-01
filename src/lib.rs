mod argument;
mod bitutils;
mod event;
mod header;
mod initialization;
mod metadata;
mod string_rec;
mod thread_rec;
mod wordutils;

pub use crate::argument::Argument;

use argument::ArgumentTypeParseError;
use event::{EventRecord, EventTypeParseError};
use header::{RecordHeader, RecordType, RecordTypeParseError};
use initialization::InitializationRecord;
use metadata::{
    MetadataRecord, MetadataTypeParseError, ProviderEvent, ProviderInfo, ProviderSection, TraceInfo,
};
use string_rec::StringRecord;
use thread_rec::ThreadRecord;
use wordutils::read_u64_word;

use std::io::{ErrorKind, Read, Write};
use std::string::FromUtf8Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FtfError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("UTF-8 conversion error: {0}")]
    Utf8(#[from] FromUtf8Error),

    #[error("Invalid record type: {0}")]
    InvalidRecordType(#[from] RecordTypeParseError),

    #[error("Invalid event type: {0}")]
    InvalidEventType(#[from] EventTypeParseError),

    #[error("Invalid metadata type: {0}")]
    InvalidMetadataType(#[from] MetadataTypeParseError),

    #[error("Invalid argument type: {0}")]
    InvalidArgumentType(#[from] ArgumentTypeParseError),

    #[error("Unsupported record type: {0:?}")]
    UnsupportedRecordType(RecordType),

    #[error("Unimplemented feature: {0}")]
    Unimplemented(String),

    #[error("Parse error: {0}")]
    ParseError(String),
}

pub type Result<T> = std::result::Result<T, FtfError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StringRef {
    Inline(String),
    Ref(u16),
}

impl StringRef {
    pub fn to_field(&self) -> u16 {
        match self {
            StringRef::Ref(r) => *r & 0x7FFF,
            StringRef::Inline(s) => (s.len() as u16) | 0x8000,
        }
    }

    pub fn field_is_ref(field: u16) -> bool {
        // test high bit is not set
        (field & 0x8000) == 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadRef {
    Inline { process_koid: u64, thread_koid: u64 },
    Ref(u8),
}

impl ThreadRef {
    pub fn to_field(&self) -> u8 {
        match self {
            Self::Inline { .. } => 0,
            Self::Ref(r) => *r,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Record {
    Metadata(MetadataRecord),
    Initialization(InitializationRecord),
    String(StringRecord),
    Thread(ThreadRecord),
    Event(EventRecord),
    Blob,
    Userspace,
    Kernel,
    Scheduling,
    Log,
    LargeBlob,
}

pub struct Archive {
    pub records: Vec<Record>,
}

impl Archive {
    pub fn read<R: Read>(mut reader: R) -> Result<Self> {
        let mut res = Vec::new();
        loop {
            match Record::from_bytes(&mut reader) {
                Ok(r) => res.push(r),
                Err(FtfError::Io(e)) => match e.kind() {
                    ErrorKind::UnexpectedEof => break,
                    _ => return Err(FtfError::Io(e)),
                },
                Err(e) => return Err(e),
            }
        }

        Ok(Archive { records: res })
    }

    pub fn write<W: Write>(&self, mut writer: W) -> Result<()> {
        for record in &self.records {
            record.write(&mut writer)?;
        }
        Ok(())
    }
}

impl Record {
    pub fn create_initialization(ticks_per_second: u64) -> Self {
        Self::Initialization(InitializationRecord::new(ticks_per_second))
    }

    pub fn create_string(index: u16, length: u32, value: String) -> Self {
        Self::String(StringRecord::new(index, length, value))
    }

    pub fn create_thread(index: u8, process_koid: u64, thread_koid: u64) -> Self {
        Self::Thread(ThreadRecord::new(index, process_koid, thread_koid))
    }

    pub fn create_provider_info(provider_id: u32, provider_name: String) -> Self {
        Self::Metadata(MetadataRecord::ProviderInfo(ProviderInfo::new(
            provider_id,
            provider_name,
        )))
    }

    pub fn create_provider_event(provider_id: u32, event_id: u8) -> Self {
        Self::Metadata(MetadataRecord::ProviderEvent(ProviderEvent::new(
            provider_id,
            event_id,
        )))
    }

    pub fn create_provider_section(provider_id: u32) -> Self {
        Self::Metadata(MetadataRecord::ProviderSection(ProviderSection::new(
            provider_id,
        )))
    }

    pub fn create_trace_info(trace_info_type: u8, data: [u8; 5]) -> Self {
        Self::Metadata(MetadataRecord::TraceInfo(TraceInfo::new(
            trace_info_type,
            &data,
        )))
    }

    pub fn create_magic_number() -> Self {
        Self::Metadata(MetadataRecord::MagicNumber)
    }

    pub fn create_instant_event(
        timestamp: u64,
        thread: ThreadRef,
        category: StringRef,
        name: StringRef,
        arguments: Vec<Argument>,
    ) -> Self {
        Self::Event(EventRecord::create_instant(
            timestamp, thread, category, name, arguments,
        ))
    }

    pub fn create_counter_event(
        timestamp: u64,
        thread: ThreadRef,
        category: StringRef,
        name: StringRef,
        arguments: Vec<Argument>,
        counter_id: u64,
    ) -> Self {
        Self::Event(EventRecord::create_counter(
            timestamp, thread, category, name, arguments, counter_id,
        ))
    }

    pub fn create_duration_begin_event(
        timestamp: u64,
        thread: ThreadRef,
        category: StringRef,
        name: StringRef,
        arguments: Vec<Argument>,
    ) -> Self {
        Self::Event(EventRecord::create_duration_begin(
            timestamp, thread, category, name, arguments,
        ))
    }

    pub fn create_duration_end_event(
        timestamp: u64,
        thread: ThreadRef,
        category: StringRef,
        name: StringRef,
        arguments: Vec<Argument>,
    ) -> Self {
        Self::Event(EventRecord::create_duration_end(
            timestamp, thread, category, name, arguments,
        ))
    }

    pub fn create_duration_complete_event(
        timestamp: u64,
        thread: ThreadRef,
        category: StringRef,
        name: StringRef,
        arguments: Vec<Argument>,
        end_ts: u64,
    ) -> Self {
        Self::Event(EventRecord::create_duration_complete(
            timestamp, thread, category, name, arguments, end_ts,
        ))
    }

    pub fn from_bytes<U: Read>(reader: &mut U) -> Result<Record> {
        let header = RecordHeader {
            value: read_u64_word(reader)?,
        };

        let record_type = header.record_type()?;
        match record_type {
            RecordType::Metadata => Ok(Self::Metadata(MetadataRecord::parse(reader, header)?)),
            RecordType::Initialization => Ok(Self::Initialization(InitializationRecord::parse(
                reader, header,
            )?)),
            RecordType::String => Ok(Self::String(StringRecord::parse(reader, header)?)),
            RecordType::Thread => Ok(Self::Thread(ThreadRecord::parse(reader, header)?)),
            RecordType::Event => Ok(Self::Event(EventRecord::parse(reader, header)?)),
            _ => Err(FtfError::UnsupportedRecordType(record_type)),
        }
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        match self {
            Self::Metadata(r) => Ok(r.write(writer)?),
            Self::Initialization(r) => Ok(r.write(writer)?),
            Self::String(r) => Ok(r.write(writer)?),
            Self::Thread(r) => Ok(r.write(writer)?),
            Self::Event(r) => Ok(r.write(writer)?),
            _ => Err(FtfError::Unimplemented("Write".to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn create_provider_info() -> Record {
        Record::create_provider_info(42, "test_provider".to_string())
    }

    fn create_trace_info() -> Record {
        Record::create_trace_info(1, [0x12, 0x34, 0x56, 0x78, 0x90])
    }

    fn create_string_record() -> Record {
        Record::create_string(1, 11, "test_string".to_string())
    }

    fn create_thread_record() -> Record {
        Record::create_thread(1, 0x1234, 0x5678)
    }

    fn create_initialization_record() -> Record {
        Record::create_initialization(1000000)
    }

    fn create_instant_event() -> Record {
        Record::create_instant_event(
            100,
            ThreadRef::Ref(1),
            StringRef::Inline("category".to_string()),
            StringRef::Inline("event_name".to_string()),
            Vec::new(),
        )
    }

    /// Creates a sample archive with various record types
    fn create_sample_archive() -> Archive {
        let records = vec![
            Record::create_magic_number(),
            create_provider_info(),
            create_trace_info(),
            create_string_record(),
            create_thread_record(),
            create_initialization_record(),
            create_instant_event(),
        ];

        Archive { records }
    }

    #[test]
    fn test_empty_archive() -> Result<()> {
        // Create an empty archive
        let archive = Archive {
            records: Vec::new(),
        };

        // Serialize to buffer
        let mut buffer = Vec::new();
        archive.write(&mut buffer)?;

        // Verify empty buffer
        assert!(buffer.is_empty());

        // Deserialize back
        let mut cursor = Cursor::new(&buffer);
        let deserialized = Archive::read(&mut cursor)?;

        // Verify empty archive
        assert!(deserialized.records.is_empty());

        Ok(())
    }

    #[test]
    fn test_single_record_archive() -> Result<()> {
        // Create archive with single record (Magic Number)
        let archive = Archive {
            records: vec![Record::create_magic_number()],
        };

        // Serialize to buffer
        let mut buffer = Vec::new();
        archive.write(&mut buffer)?;

        // Verify non-empty buffer
        assert!(!buffer.is_empty());

        // Deserialize back
        let mut cursor = Cursor::new(&buffer);
        let deserialized = Archive::read(&mut cursor)?;

        // Verify single record
        assert_eq!(deserialized.records.len(), 1);

        // Verify record type
        match &deserialized.records[0] {
            Record::Metadata(MetadataRecord::MagicNumber) => (),
            _ => panic!(
                "Expected MagicNumber record, got {:?}",
                deserialized.records[0]
            ),
        }

        Ok(())
    }

    #[test]
    fn test_multi_record_archive() -> Result<()> {
        // Create a sample archive with multiple records
        let archive = create_sample_archive();
        let original_len = archive.records.len();

        // Serialize to buffer
        let mut buffer = Vec::new();
        archive.write(&mut buffer)?;

        // Verify buffer has content
        assert!(!buffer.is_empty());

        // Deserialize back
        let mut cursor = Cursor::new(&buffer);
        let deserialized = Archive::read(&mut cursor)?;

        // Verify record count
        assert_eq!(deserialized.records.len(), original_len);

        // Verify records match
        for (i, (original, deserialized)) in archive
            .records
            .iter()
            .zip(deserialized.records.iter())
            .enumerate()
        {
            assert_eq!(
                original, deserialized,
                "Record at index {} doesn't match",
                i
            );
        }

        Ok(())
    }

    #[test]
    fn test_archive_with_unsupported_record_types() -> Result<()> {
        // Create an archive with a mixture of supported and unsupported record types
        let mut archive = create_sample_archive();

        // Add unsupported record types
        archive.records.push(Record::Blob);
        archive.records.push(Record::Userspace);

        // Serialize to buffer (should skip unsupported records)
        let mut buffer = Vec::new();

        // This should return an error for unsupported record types
        let write_result = archive.write(&mut buffer);
        assert!(write_result.is_err());

        // But the buffer should still contain some supported records that were written before the error
        assert!(!buffer.is_empty());

        // Deserializing should give us just the records that were successfully written
        let mut cursor = Cursor::new(&buffer);
        let deserialized = Archive::read(&mut cursor)?;

        // Verify all records are supported types
        for record in &deserialized.records {
            match record {
                Record::Metadata(_)
                | Record::Initialization(_)
                | Record::String(_)
                | Record::Thread(_)
                | Record::Event(_) => (),
                _ => panic!("Unexpected unsupported record type: {:?}", record),
            }
        }

        Ok(())
    }

    #[test]
    fn test_archive_handles_incomplete_read() -> Result<()> {
        // Create a sample archive
        let archive = create_sample_archive();

        // Serialize to buffer
        let mut buffer = Vec::new();
        archive.write(&mut buffer)?;

        // Take just part of the buffer (first 16 bytes)
        let partial_buffer = buffer[0..16].to_vec();

        // Deserialize the partial buffer - should handle EOF gracefully
        let mut cursor = Cursor::new(&partial_buffer);
        let deserialized = Archive::read(&mut cursor)?;

        // Should have parsed records up to the truncation
        assert!(!deserialized.records.is_empty());
        assert!(deserialized.records.len() <= archive.records.len());

        Ok(())
    }

    #[test]
    fn test_archive_read_write_read_cycle() -> Result<()> {
        // Create a sample archive
        let original_archive = create_sample_archive();

        // Write to buffer
        let mut buffer1 = Vec::new();
        original_archive.write(&mut buffer1)?;

        // Read back
        let mut cursor1 = Cursor::new(&buffer1);
        let intermediate_archive = Archive::read(&mut cursor1)?;

        // Write again
        let mut buffer2 = Vec::new();
        intermediate_archive.write(&mut buffer2)?;

        // Verify the buffers are identical (round-trip serialization is consistent)
        assert_eq!(buffer1.len(), buffer2.len());
        assert_eq!(buffer1, buffer2);

        // Read again
        let mut cursor2 = Cursor::new(&buffer2);
        let final_archive = Archive::read(&mut cursor2)?;

        // Verify the original and final archives are the same
        assert_eq!(original_archive.records.len(), final_archive.records.len());

        for (i, (original, final_rec)) in original_archive
            .records
            .iter()
            .zip(final_archive.records.iter())
            .enumerate()
        {
            assert_eq!(
                original, final_rec,
                "Record at index {} doesn't match after cycle",
                i
            );
        }

        Ok(())
    }

    #[test]
    fn test_archive_appending() -> Result<()> {
        // Create two separate archives
        let archive1 = Archive {
            records: vec![Record::create_magic_number(), create_string_record()],
        };

        let archive2 = Archive {
            records: vec![create_thread_record()],
        };

        // Serialize both to the same buffer (appending)
        let mut buffer = Vec::new();
        archive1.write(&mut buffer)?;
        archive2.write(&mut buffer)?;

        // Deserialize combined buffer
        let mut cursor = Cursor::new(&buffer);
        let combined = Archive::read(&mut cursor)?;

        // Should have all 3 records
        assert_eq!(combined.records.len(), 3);

        // Verify order and content
        match &combined.records[0] {
            Record::Metadata(MetadataRecord::MagicNumber) => (),
            _ => panic!("Expected MagicNumber, got {:?}", combined.records[0]),
        }

        // Verify the string record
        let mut string_found = false;
        for record in &combined.records {
            if let Record::String(sr) = record {
                assert_eq!(sr.index(), 1);
                assert_eq!(sr.value(), &"test_string".to_string());
                string_found = true;
                break;
            }
        }
        assert!(string_found, "String record not found");

        // Verify the thread record
        let mut thread_found = false;
        for record in &combined.records {
            if let Record::Thread(tr) = record {
                assert_eq!(tr.index(), 1);
                assert_eq!(tr.process_koid(), 0x1234);
                assert_eq!(tr.thread_koid(), 0x5678);
                thread_found = true;
                break;
            }
        }
        assert!(thread_found, "Thread record not found");

        Ok(())
    }
}
