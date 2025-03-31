
use crate::extract_bits;
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

pub struct RecordHeader {
    pub value: u64,
}

impl RecordHeader {
    pub fn size(&self) -> u16 {
        extract_bits!(self.value, 4, 15) as u16
    }

    pub fn record_type(&self) -> anyhow::Result<RecordType> {
        Ok(RecordType::try_from(extract_bits!(self.value, 0, 3) as u8)?)
    }
}
