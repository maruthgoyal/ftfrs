use crate::header::CustomField;
use crate::wordutils::pad_to_multiple_of_8;
use crate::{FtfError, Result};
use std::io::{Read, Write};
use thiserror::Error;

use crate::{
    extract_bits,
    wordutils::{read_aligned_str, read_u64_word},
    Argument, RecordHeader, StringOrRef, ThreadOrRef,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum EventType {
    Instant = 0,
    Counter = 1,
    DurationBegin = 2,
    DurationEnd = 3,
    DurationComplete = 4,
    AsyncBegin = 5,
    AsyncInstant = 6,
    AsyncEnd = 7,
    FlowBegin = 8,
    FlowStep = 9,
    FlowEnd = 10,
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
#[error("Invalid Event type {0}")]
pub struct EventTypeParseError(u8);

impl TryFrom<u8> for EventType {
    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Instant),
            1 => Ok(Self::Counter),
            2 => Ok(Self::DurationBegin),
            3 => Ok(Self::DurationEnd),
            4 => Ok(Self::DurationComplete),
            5 => Ok(Self::AsyncBegin),
            6 => Ok(Self::AsyncInstant),
            7 => Ok(Self::AsyncEnd),
            8 => Ok(Self::FlowBegin),
            9 => Ok(Self::FlowStep),
            10 => Ok(Self::FlowEnd),
            _ => Err(EventTypeParseError(value)),
        }
    }

    type Error = EventTypeParseError;
}

#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    pub timestamp: u64,
    pub thread: ThreadOrRef,
    pub category: StringOrRef,
    pub name: StringOrRef,
    pub arguments: Vec<Argument>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Instant {
    pub event: Event,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Counter {
    pub event: Event,
    pub counter_id: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DurationBegin {
    pub event: Event,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DurationEnd {
    pub event: Event,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DurationComplete {
    pub event: Event,
    pub duration_ticks: u64,
}

impl Instant {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.event
            .write_event(writer, EventType::Instant, Vec::new())
    }
}

impl DurationBegin {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.event
            .write_event(writer, EventType::DurationBegin, Vec::new())
    }
}

impl DurationEnd {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.event
            .write_event(writer, EventType::DurationEnd, Vec::new())
    }
}

impl Counter {
    fn parse<U: Read>(reader: &mut U, event: Event) -> Result<Self> {
        let counter_id = read_u64_word(reader)?;
        Ok(Self { event, counter_id })
    }

    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.event
            .write_event(writer, EventType::Counter, vec![self.counter_id])
    }
}

impl DurationComplete {
    fn parse<U: Read>(reader: &mut U, event: Event) -> Result<Self> {
        let duration_ticks = read_u64_word(reader)?;
        Ok(Self {
            event,
            duration_ticks,
        })
    }

    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.event.write_event(
            writer,
            EventType::DurationComplete,
            vec![self.duration_ticks],
        )
    }
}

impl Event {
    fn write_event<W: Write>(
        &self,
        writer: &mut W,
        event_type: EventType,
        event_extra_words: Vec<u64>,
    ) -> Result<()> {
        // header + timestamp always
        let mut num_words = 1 + 1;
        if let ThreadOrRef::ProcessAndThread(_, _) = &self.thread {
            num_words += 2;
        }

        if let StringOrRef::String(s) = &self.category {
            num_words += (s.len() + 7) / 8;
        }

        if let StringOrRef::String(s) = &self.name {
            num_words += (s.len() + 7) / 8;
        }

        if !self.arguments.is_empty() {
            todo!("Implement arguments support");
        }

        let header = RecordHeader::build(
            crate::header::RecordType::Event,
            num_words as u8 + event_extra_words.len() as u8,
            vec![
                CustomField {
                    width: 4,
                    value: event_type as u64,
                },
                CustomField {
                    width: 4,
                    value: self.arguments.len() as u64,
                },
                CustomField {
                    width: 8,
                    value: self.thread.to_field() as u64,
                },
                CustomField {
                    width: 16,
                    value: self.category.to_field() as u64,
                },
                CustomField {
                    width: 16,
                    value: self.name.to_field() as u64,
                },
            ],
        )?;

        writer.write_all(&header.value.to_le_bytes())?;
        writer.write_all(&self.timestamp.to_le_bytes())?;

        if let ThreadOrRef::ProcessAndThread(p, t) = self.thread {
            writer.write_all(&p.to_le_bytes())?;
            writer.write_all(&t.to_le_bytes())?;
        }

        if let StringOrRef::String(s) = &self.category {
            let padded = pad_to_multiple_of_8(s.as_bytes());
            writer.write_all(&padded)?;
        }

        if let StringOrRef::String(s) = &self.name {
            let padded = pad_to_multiple_of_8(s.as_bytes());
            writer.write_all(&padded)?;
        }

        // arguments should go here

        for extra in event_extra_words {
            writer.write_all(&extra.to_le_bytes())?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum EventRecord {
    Instant(Instant),
    Counter(Counter),
    DurationBegin(DurationBegin),
    DurationEnd(DurationEnd),
    DurationComplete(DurationComplete),
    AsyncBegin,
    AsyncEnd,
    AsyncInstant,
    FlowBegin,
    FlowEnd,
    FlowStep,
}

impl EventRecord {
    pub fn parse<U: Read>(reader: &mut U, header: RecordHeader) -> Result<Self> {
        let (event_type, event) = EventRecord::parse_event(reader, &header)?;
        match event_type {
            EventType::Instant => Ok(Self::Instant(Instant { event })),
            EventType::Counter => Ok(Self::Counter(Counter::parse(reader, event)?)),
            EventType::DurationBegin => Ok(Self::DurationBegin(DurationBegin { event })),
            EventType::DurationEnd => Ok(Self::DurationEnd(DurationEnd { event })),
            EventType::DurationComplete => Ok(Self::DurationComplete(DurationComplete::parse(
                reader, event,
            )?)),
            EventType::AsyncBegin => Err(FtfError::Unimplemented(
                "AsyncBegin event type not implemented".to_string(),
            )),
            EventType::AsyncEnd => Err(FtfError::Unimplemented(
                "AsyncEnd event type not implemented".to_string(),
            )),
            EventType::AsyncInstant => Err(FtfError::Unimplemented(
                "AsyncInstant event type not implemented".to_string(),
            )),
            EventType::FlowBegin => Err(FtfError::Unimplemented(
                "FlowBegin event type not implemented".to_string(),
            )),
            EventType::FlowStep => Err(FtfError::Unimplemented(
                "FlowStep event type not implemented".to_string(),
            )),
            EventType::FlowEnd => Err(FtfError::Unimplemented(
                "FlowEnd event type not implemented".to_string(),
            )),
        }
    }

    fn parse_event<U: Read>(reader: &mut U, header: &RecordHeader) -> Result<(EventType, Event)> {
        let event_type = extract_bits!(header.value, 16, 19) as u8;
        let n_args = extract_bits!(header.value, 20, 23) as u8;
        let thread = extract_bits!(header.value, 24, 31) as u8;
        let category = extract_bits!(header.value, 32, 47) as u16;
        let name = extract_bits!(header.value, 48, 63) as u16;

        let event_type = EventType::try_from(event_type)?;

        let timestamp = read_u64_word(reader)?;

        let thread = if thread == 0 {
            let process_id = read_u64_word(reader)?;
            let thread_id = read_u64_word(reader)?;
            ThreadOrRef::ProcessAndThread(process_id, thread_id)
        } else {
            ThreadOrRef::Ref(thread)
        };

        let category = if (category >> 15) == 0 {
            StringOrRef::Ref(category)
        } else {
            let cat = read_aligned_str(reader, (category & 0x7FFF) as usize)?;
            StringOrRef::String(cat)
        };

        let name = if (name >> 15) == 0 {
            StringOrRef::Ref(name)
        } else {
            let n = read_aligned_str(reader, (name & 0x7FFF) as usize)?;
            StringOrRef::String(n)
        };

        if n_args > 0 {
            return Err(FtfError::Unimplemented(
                "Argument parsing not implemented yet".to_string(),
            ));
        }

        Ok((
            event_type,
            Event {
                timestamp,
                thread,
                category,
                name,
                arguments: Vec::new(),
            },
        ))
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        match self {
            EventRecord::Counter(e) => e.write(writer),
            EventRecord::Instant(e) => e.write(writer),
            EventRecord::DurationBegin(e) => e.write(writer),
            EventRecord::DurationEnd(e) => e.write(writer),
            EventRecord::DurationComplete(e) => e.write(writer),
            _ => Err(FtfError::Unimplemented(
                "Write not implemented for this type yet".to_string(),
            )),
        }
    }
}
