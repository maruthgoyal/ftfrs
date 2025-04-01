use crate::{extract_bits, mask_length, Result};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RecordType {
    Metadata = 0,
    Initialization = 1,
    String = 2,
    Thread = 3,
    Event = 4,
    Blob = 5,
    Userspace = 6,
    Kernel = 7,
    Scheduling = 8,
    Log = 9,
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
pub(super) struct RecordHeader {
    pub value: u64,
}

impl RecordHeader {
    pub(super) fn build(
        record_type: RecordType,
        record_size: u8,
        fields: Vec<CustomField>,
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

    #[allow(dead_code)]
    pub fn size(&self) -> u16 {
        extract_bits!(self.value, 4, 15) as u16
    }

    pub fn record_type(&self) -> Result<RecordType> {
        Ok(RecordType::try_from(extract_bits!(self.value, 0, 3) as u8)?)
    }
}
