use crate::header::RecordHeader;
use std::io::Read;
use anyhow::Result;

#[derive(Debug)]
pub struct InitializationRecord {
    pub ticks_per_second: u64
}

impl InitializationRecord {
    pub fn parse<U: Read>(reader: &mut U, header: RecordHeader) -> Result<Self> {
        let mut data = [0; 8];
        reader.read_exact(&mut data)?;
        Ok(InitializationRecord { ticks_per_second: u64::from_le_bytes(data) })
    } 
}