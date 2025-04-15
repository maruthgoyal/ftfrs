use crate::{event::EventType, extract_bits, mask_length, Result};
use thiserror::Error;

/// Type of a record
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RecordType {
    /// Metadata record
    Metadata = 0,
    /// Initialization record
    Initialization = 1,
    /// String record
    String = 2,
    /// Thread record
    Thread = 3,
    /// Event record
    Event = 4,
    /// BLOB record
    Blob = 5,
    /// Userspace record
    Userspace = 6,
    /// Kernel record
    Kernel = 7,
    /// Scheduling record
    Scheduling = 8,
    /// Log record
    Log = 9,
    /// Large BLOB record
    LargeBlob = 15,
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
#[error("Invalid record type {0}")]
pub struct RecordTypeParseError(u8);

impl TryFrom<u8> for RecordType {
    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Metadata),
            1 => Ok(Self::Initialization),
            2 => Ok(Self::String),
            3 => Ok(Self::Thread),
            4 => Ok(Self::Event),
            5 => Ok(Self::Blob),
            6 => Ok(Self::Userspace),
            7 => Ok(Self::Kernel),
            8 => Ok(Self::Scheduling),
            9 => Ok(Self::Log),
            15 => Ok(Self::LargeBlob),
            _ => Err(RecordTypeParseError(value)),
        }
    }

    type Error = RecordTypeParseError;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct CustomField {
    pub width: u8,
    pub value: u64,
}

/// Header for a record
pub struct RecordHeader {
    pub(crate) value: u64,
}

impl RecordHeader {
    pub(super) fn build_event_header(
        record_size: u8,
        event_type: EventType,
        nargs: usize,
        tid: u8,
        cid: u16,
        nid: u16,
    ) -> Result<Self> {
        let mut res: u64 = 0;

        res |= RecordType::Event as u64;
        res |= (record_size as u64) << 4;
        res |= mask_length!(event_type as u64, 4) << 16;
        res |= mask_length!(nargs as u64, 4) << 20;
        res |= mask_length!(tid as u64, 8) << 24;
        res |= mask_length!(cid as u64, 16) << 32;
        res |= mask_length!(nid as u64, 16) << 48;

        Ok(Self { value: res })
    }

    pub(super) fn build(
        record_type: RecordType,
        record_size: u8,
        fields: &[CustomField],
    ) -> Result<Self> {
        let record_type = record_type as u8;
        let mut res: u64 = 0;

        res |= record_type as u64;
        res |= (record_size as u64) << 4;

        let mut offset: u8 = 4 + 12;
        for field in fields {
            res |= mask_length!(field.value, field.width) << offset;
            offset += field.width;
        }

        Ok(Self { value: res })
    }

    /// Create a RecordHeader
    /// * value: 8-byte header for a record
    pub fn new(value: u64) -> Self {
        Self { value }
    }

    /// Returns the size of the record described by this header
    /// as a multiple of 8-bytes. i.e., size of 2 means the record is 16-bytes long
    pub fn size(&self) -> u16 {
        extract_bits!(self.value, 4, 15) as u16
    }

    /// Type of the record described by this header
    pub fn record_type(&self) -> Result<RecordType> {
        Ok(RecordType::try_from(extract_bits!(self.value, 0, 3) as u8)?)
    }
}
