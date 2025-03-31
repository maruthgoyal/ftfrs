use anyhow::Result;
use std::io::Read;
use thiserror::Error;

use crate::{
    extract_bits,
    wordutils::{self, read_aligned_str, read_u64_word},
    Argument, RecordHeader, StringOrRef, ThreadOrRef,
};

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

#[derive(Debug)]
pub struct Event {
    pub timestamp: u64,
    pub thread: ThreadOrRef,
    pub category: StringOrRef,
    pub name: StringOrRef,
    pub arguments: Vec<Argument>,
}

#[derive(Debug)]
pub struct Instant {
    pub event: Event,
}

#[derive(Debug)]
pub struct Counter {
    pub event: Event,
    pub counter_id: u64,
}

#[derive(Debug)]
pub struct DurationBegin {
    pub event: Event,
}

#[derive(Debug)]
pub struct DurationEnd {
    pub event: Event,
}

#[derive(Debug)]
pub struct DurationComplete {
    pub event: Event,
    pub duration_ticks: u64,
}

impl Counter {
    fn parse<U: Read>(reader: &mut U, event: Event) -> Result<Self> {
        let counter_id = read_u64_word(reader)?;
        Ok(Self { event, counter_id })
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
}

#[derive(Debug)]
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
            EventType::AsyncBegin => unimplemented!(),
            EventType::AsyncEnd => unimplemented!(),
            EventType::AsyncInstant => unimplemented!(),
            EventType::FlowBegin => unimplemented!(),
            EventType::FlowStep => unimplemented!(),
            EventType::FlowEnd => unimplemented!(),
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

        log::warn!("Argument parsing not implemented!");

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

        // unimplemented!()
    }
}
