use crate::{header::RecordHeader, wordutils::read_u64_word};
use std::io::Read;
use anyhow::Result;

#[derive(Debug)]
pub struct InitializationRecord {
    pub ticks_per_second: u64
}

impl InitializationRecord {
    pub fn parse<U: Read>(reader: &mut U, header: RecordHeader) -> Result<Self> {
        Ok(InitializationRecord { ticks_per_second: read_u64_word(reader)? })
    } 
}