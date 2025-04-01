mod bitutils;
pub mod event;
mod header;
mod initialization;
mod metadata;
mod string_rec;
mod thread_rec;
mod wordutils;

use event::{EventRecord, EventTypeParseError};
use header::{RecordHeader, RecordType, RecordTypeParseError};
use initialization::InitializationRecord;
use metadata::{
    MetadataRecord, MetadataTypeParseError, ProviderEvent, ProviderInfo, ProviderSection, TraceInfo,
};
use string_rec::StringRecord;
use thread_rec::ThreadRecord;
use wordutils::read_u64_word;

#[cfg(test)]
mod tests {
    pub mod archive_test;
    pub mod bitutils_test;
}

use std::io::{ErrorKind, Read, Write};
use std::string::FromUtf8Error;
use thiserror::Error;

/// Custom error type for the FTF parsing library
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

    #[error("Unsupported record type: {0:?}")]
    UnsupportedRecordType(RecordType),

    #[error("Unimplemented feature: {0}")]
    Unimplemented(String),

    #[error("Parse error: {0}")]
    ParseError(String),
}

/// Type alias for Result with FtfError
pub type Result<T> = std::result::Result<T, FtfError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StringOrRef {
    String(String),
    Ref(u16),
}

impl StringOrRef {
    pub fn to_field(&self) -> u16 {
        match self {
            StringOrRef::Ref(r) => *r & 0x7FFF,
            StringOrRef::String(s) => (s.len() as u16) | 0x8000,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadOrRef {
    ProcessAndThread(u64, u64),
    Ref(u8),
}

impl ThreadOrRef {
    pub fn to_field(&self) -> u8 {
        match self {
            Self::ProcessAndThread(_, _) => 0,
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Argument {
    Null,
    Int32(i32),
    UInt32(u32),
    Int64(i64),
    UInt64(u64),
    Float(f64),
    Str,
    Pointer,
    KernelObjectId,
    Boolean(bool),
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

    // TODO: change to [u8; 5]
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
        thread: ThreadOrRef,
        category: StringOrRef,
        name: StringOrRef,
        arguments: Vec<Argument>,
    ) -> Self {
        Self::Event(EventRecord::create_instant(timestamp, thread, category, name, arguments))
    }

    pub fn create_counter_event(
        timestamp: u64,
        thread: ThreadOrRef,
        category: StringOrRef,
        name: StringOrRef,
        arguments: Vec<Argument>,
        counter_id: u64,
    ) -> Self {
        Self::Event(EventRecord::create_counter(timestamp, thread, category, name, arguments, counter_id))
    }

    pub fn create_duration_begin_event(
        timestamp: u64,
        thread: ThreadOrRef,
        category: StringOrRef,
        name: StringOrRef,
        arguments: Vec<Argument>,
    ) -> Self {
        Self::Event(EventRecord::create_duration_begin(timestamp, thread, category, name, arguments))
    }

    pub fn create_duration_end_event(
        timestamp: u64,
        thread: ThreadOrRef,
        category: StringOrRef,
        name: StringOrRef,
        arguments: Vec<Argument>,
    ) -> Self {
        Self::Event(EventRecord::create_duration_end(timestamp, thread, category, name, arguments))
    }

    pub fn create_duration_complete_event(
        timestamp: u64,
        thread: ThreadOrRef,
        category: StringOrRef,
        name: StringOrRef,
        arguments: Vec<Argument>,
        end_ts: u64,
    ) -> Self {
        Self::Event(EventRecord::create_duration_complete(timestamp, thread, category, name, arguments, end_ts))
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
