mod bitutils;
mod header;
mod metadata;
mod initialization;
mod stringrec;
mod wordutils;
mod threadrec;
mod event;
#[cfg(test)]
mod tests {
    pub mod bitutils_test;
    pub mod metadata_test;
    pub mod initialization_test;
    pub mod stringrec_test;
    pub mod threadrec_test;
}

use crate::metadata::MetadataRecord;
use event::{Event, EventRecord};
use header::{RecordHeader, RecordType};
use initialization::InitializationRecord;
use stringrec::StringRecord;
use threadrec::ThreadRecord;
use wordutils::read_u64_word;

use std::io::Read;

enum StringOrRef {
    String(String),
    Ref(u16),
}
enum ThreadOrRef {
    ProcessAndThread(u64, u64),
    Ref(u8),
}

enum Record {
    Metadata(MetadataRecord),
    Initialization(InitializationRecord),
    String(StringRecord),
    Thread(ThreadRecord),
    Event(EventRecord),
    Blob,
    Userspace,
    Kernel,
    Scheduling,
    Log,
    LargeBlob,
}

struct Archive {
    records: Vec<Record>,
}

enum Argument {
    Null,
    Int32(i32),
    UInt32(u32),
    Int64(i64),
    UInt64(u64),
    Float(f64),
    Str,
    Pointer,
    KernelObjectId,
    Boolean(bool),
}

impl Record {
    fn from_bytes<U: Read>(mut reader: U) -> anyhow::Result<Record> {
        let header = RecordHeader { value: read_u64_word(&mut reader)? };

        let record_type = header.record_type()?;
        match record_type {
            RecordType::Metadata => Ok(Self::Metadata(MetadataRecord::parse(&mut reader, header)?)),
            RecordType::Initialization => Ok(Self::Initialization(InitializationRecord::parse(&mut reader, header)?)),
            RecordType::String => Ok(Self::String(StringRecord::parse(&mut reader, header)?)),
            RecordType::Thread => Ok(Self::Thread(ThreadRecord::parse(&mut reader, header)?)),
            RecordType::Event => Ok(Self::Event(EventRecord::parse(&mut reader, header)?)),
            _ => Err(anyhow::anyhow!("Unsupported record type {:?}", record_type)),
        }
    }
}
