use crate::wordutils;
use crate::{extract_bits, RecordHeader, Result};
use std::io::Read;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StringRecord {
    pub index: u16,
    pub length: u32,
    pub value: String,
}

impl StringRecord {
    pub fn parse<U: Read>(reader: &mut U, header: RecordHeader) -> Result<Self> {
        let index = extract_bits!(header.value, 16, 30) as u16;
        let length = extract_bits!(header.value, 32, 46) as u32;

        let value = wordutils::read_aligned_str(reader, length as usize)?;
        Ok(StringRecord {
            index,
            length,
            value,
        })
    }
}
