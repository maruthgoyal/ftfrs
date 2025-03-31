use crate::{extract_bits, RecordHeader};
use anyhow::Result;
use std::io::Read;

#[derive(Debug)]
pub struct ThreadRecord {
    pub index: u8,
    pub process_koid: u64,
    pub thread_koid: u64,
}

impl ThreadRecord {
    pub fn parse<U: Read>(reader: &mut U, header: RecordHeader) -> Result<Self> {
        let index = extract_bits!(header.value, 16, 23) as u8;

        let mut buf = [0; 8];
        reader.read_exact(&mut buf)?;
        let process_koid = u64::from_le_bytes(buf);

        let mut buf = [0; 8];
        reader.read_exact(&mut buf)?;
        let thread_koid = u64::from_le_bytes(buf);

        Ok(ThreadRecord {
            index,
            process_koid,
            thread_koid,
        })
    }
}
