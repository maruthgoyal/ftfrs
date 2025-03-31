mod bitutils;
mod event;
mod header;
mod initialization;
mod metadata;
mod string_rec;
mod thread_rec;
mod wordutils;
#[cfg(test)]
mod tests {
    pub mod bitutils_test;
    pub mod event_test;
    pub mod initialization_test;
    pub mod metadata_test;
    pub mod string_rec_test;
    pub mod thread_rec_test;
}

use crate::metadata::{MetadataRecord, MetadataTypeParseError};
use event::{EventRecord, EventTypeParseError};
use header::{RecordHeader, RecordType, RecordTypeParseError};
use initialization::InitializationRecord;
use string_rec::StringRecord;
use thread_rec::ThreadRecord;
use wordutils::read_u64_word;

use std::io::Read;
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadOrRef {
    ProcessAndThread(u64, u64),
    Ref(u8),
}

trait WriteRec<W: std::io::Write> {
    fn write(writer: W) -> Result<()>;
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
    pub fn from_bytes<U: Read>(mut reader: U) -> Result<Record> {
        let header = RecordHeader {
            value: read_u64_word(&mut reader)?,
        };

        let record_type = header.record_type()?;
        match record_type {
            RecordType::Metadata => Ok(Self::Metadata(MetadataRecord::parse(&mut reader, header)?)),
            RecordType::Initialization => Ok(Self::Initialization(InitializationRecord::parse(
                &mut reader,
                header,
            )?)),
            RecordType::String => Ok(Self::String(StringRecord::parse(&mut reader, header)?)),
            RecordType::Thread => Ok(Self::Thread(ThreadRecord::parse(&mut reader, header)?)),
            RecordType::Event => Ok(Self::Event(EventRecord::parse(&mut reader, header)?)),
            _ => Err(FtfError::UnsupportedRecordType(record_type)),
        }
    }
}
