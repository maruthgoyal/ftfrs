use crate::header::CustomField;
use crate::wordutils::{self, pad_to_multiple_of_8};
use crate::{extract_bits, RecordHeader, Result};
use std::io::{Read, Write};

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

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        let str_bytes = self.value.as_bytes();
        // header + num words for string
        let num_words = 1 + ((str_bytes.len() + 7)/ 8);
        let header = RecordHeader::build(crate::header::RecordType::String, num_words as u8, vec![
            CustomField {
                width: 15,
                value: self.index as u64
            },
            CustomField {
                width: 1,
                value: 0
            },
            CustomField {
                width: 15,
                value: self.length as u64
            },
        ])?;
        
        writer.write_all(&header.value.to_le_bytes())?;

        let padded = pad_to_multiple_of_8(str_bytes);
        writer.write_all(&padded)?;
       
        Ok(())
    }
}
