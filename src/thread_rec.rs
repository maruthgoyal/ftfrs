use crate::{extract_bits, header::CustomField, wordutils::read_u64_word, RecordHeader, Result};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThreadRecord {
    pub index: u8,
    pub process_koid: u64,
    pub thread_koid: u64,
}

impl ThreadRecord {
    pub fn parse<U: Read>(reader: &mut U, header: RecordHeader) -> Result<Self> {
        let index = extract_bits!(header.value, 16, 23) as u8;

        let process_koid = read_u64_word(reader)?;
        let thread_koid = read_u64_word(reader)?;

        Ok(ThreadRecord {
            index,
            process_koid,
            thread_koid,
        })
    }
    
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        let header = RecordHeader::build(crate::header::RecordType::Thread, 3, vec![
            CustomField {
                width: 8,
                value: self.index as u64
            }
        ])?;

        writer.write_all(&header.value.to_le_bytes())?;
        writer.write_all(&self.process_koid.to_le_bytes())?;
        writer.write_all(&self.thread_koid.to_le_bytes())?;
        
        Ok(())
    }
}
