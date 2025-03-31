use anyhow::Result;
use std::io::Read;
use thiserror::Error;

use crate::{extract_bits, strutils, RecordHeader};

#[derive(Debug)]
pub struct TraceInfo {
    pub trace_info_type: u8,
    // only 40 bits, but no point in encoding as [u8; 5]
    pub data: u64,
}

#[derive(Debug)]
pub struct ProviderInfo {
    pub provider_id: u32,
    pub provider_name: String,
}

#[derive(Debug)]
pub struct ProviderSection {
    pub provider_id: u32,
}

#[derive(Debug)]
pub struct ProviderEvent {
    pub provider_id: u32,
    pub event_id: u8,
}

#[derive(Debug)]
pub struct MagicNumber;

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

#[derive(Debug)]
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

    pub fn parse<U: Read>(reader: &mut U, header: RecordHeader) -> Result<Self> {
        if header.value == Self::MAGIC_NUMBER_RECORD {
            return Ok(Self::MagicNumber);
        }

        match MetadataRecord::metadata_type(&header)? {
            MetadataType::ProviderInfo => {
                let provider_id = Self::provider_id(&header);
                let namelen = extract_bits!(header.value, 52, 59) as usize;

                let provider_name = strutils::read_aligned_str(reader, namelen, &header)?;

                Ok(Self::ProviderInfo(ProviderInfo { provider_id, provider_name })) 

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
                Ok(Self::TraceInfo(TraceInfo { trace_info_type, data }))
            }
        }
    }
}
