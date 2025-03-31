use crate::{header::RecordHeader, wordutils::read_u64_word, Result};
use std::io::Read;

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
}
