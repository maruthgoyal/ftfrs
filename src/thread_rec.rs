use crate::{extract_bits, wordutils::read_u64_word, RecordHeader, Result};
use std::io::Read;

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
}
