use crate::{header::RecordHeader, wordutils::read_u64_word, Result};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InitializationRecord {
    pub ticks_per_second: u64,
}

impl InitializationRecord {
    pub fn parse<U: Read>(reader: &mut U, _header: RecordHeader) -> Result<Self> {
        Ok(InitializationRecord {
            ticks_per_second: read_u64_word(reader)?,
        })
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        let header = RecordHeader::build(crate::header::RecordType::Initialization, 2, Vec::new())?;
        writer.write_all(&header.value.to_le_bytes())?;
        writer.write_all(&self.ticks_per_second.to_le_bytes())?;
        Ok(())
    }
}
