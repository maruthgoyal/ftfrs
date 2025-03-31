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
    pub trace_info_type: u8,
    // only 40 bits, but no point in encoding as [u8; 5]
    pub data: u64,
}

impl TraceInfo {
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
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
    pub provider_id: u32,
    pub provider_name: String,
}

impl ProviderInfo {
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        let header = RecordHeader::build(
            crate::header::RecordType::Metadata,
            1,
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
    pub provider_id: u32,
}
impl ProviderSection {
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
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
    pub provider_id: u32,
    pub event_id: u8,
}

impl ProviderEvent {
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
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

    pub fn parse<U: Read>(reader: &mut U, header: RecordHeader) -> Result<Self> {
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
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
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
