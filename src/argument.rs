use std::io::Read;
use thiserror::Error;

use crate::{extract_bits, wordutils::{read_aligned_str, read_u64_word}, Result, StringRef};

#[derive(Debug, Clone, PartialEq)]
pub enum Argument {
    Null(StringRef),
    Int32(StringRef, i32),
    UInt32(StringRef, u32),
    Int64(StringRef, i64),
    UInt64(StringRef, u64),
    Float(StringRef, f64),
    Str(StringRef, StringRef),
    Pointer(StringRef, u64),
    KernelObjectId(StringRef, u64),
    Boolean(StringRef, bool),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum ArgumentType {
    Null = 0,
    Int32 = 1,
    UInt32 = 2,
    Int64 = 3,
    UInt64 = 4,
    Float = 5,
    Str = 6,
    Pointer = 7,
    KernelObjectId = 8,
    Boolean = 9,
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
#[error("Invalid argument type {0}")]
pub struct ArgumentTypeParseError(u8);

impl TryFrom<u8> for ArgumentType {
    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Null),
            1 => Ok(Self::Int32),
            2 => Ok(Self::UInt32),
            3 => Ok(Self::Int64),
            4 => Ok(Self::UInt64),
            5 => Ok(Self::Float),
            6 => Ok(Self::Str),
            7 => Ok(Self::Pointer),
            8 => Ok(Self::KernelObjectId),
            9 => Ok(Self::Boolean),
            _ => Err(ArgumentTypeParseError(value))
        }
    }

    type Error = ArgumentTypeParseError;
}

impl Argument {
    pub(super) fn read<R: Read>(reader: &mut R) -> Result<Self> {
        let header = read_u64_word(reader)?;
        let arg_type = extract_bits!(header, 0, 3) as u8;
        let arg_type = ArgumentType::try_from(arg_type)?;

        // size as multiple of 8 bytes including header
        let arg_size = extract_bits!(header, 4, 15) as u16;

        let arg_name = extract_bits!(header, 16, 31) as u16;        
        let arg_name = if StringRef::field_is_ref(arg_name) {
            StringRef::Ref(arg_name)
        } else {
            StringRef::Inline(read_aligned_str(reader, arg_name as usize)?)
        };

        match arg_type {
            ArgumentType::Null => Ok(Argument::Null(arg_name)),
            ArgumentType::Int32 => Ok(Argument::Int32(arg_name, extract_bits!(header, 32, 63) as i32)),
            ArgumentType::UInt32 => Ok(Argument::UInt32(arg_name, extract_bits!(header, 32, 63) as u32)),
            ArgumentType::Int64 => Ok(Argument::Int64(arg_name, read_u64_word(reader)? as i64)),
            ArgumentType::UInt64 => Ok(Argument::UInt64(arg_name, read_u64_word(reader)?)),
            ArgumentType::Float => Ok(Argument::Float(arg_name, read_u64_word(reader)? as f64)),
            ArgumentType::Str => {
                let arg_value = extract_bits!(header, 32, 47) as u16;
                let arg_value = if StringRef::field_is_ref(arg_value) {
                    StringRef::Ref(arg_value)
                } else {
                    StringRef::Inline(read_aligned_str(reader, arg_value as usize)?)
                };
                Ok(Argument::Str(arg_name, arg_value))
            }
            ArgumentType::Pointer => Ok(Argument::Pointer(arg_name, read_u64_word(reader)?)),
            ArgumentType::KernelObjectId => Ok(Argument::KernelObjectId(arg_name, read_u64_word(reader)?)),
            ArgumentType::Boolean => Ok(Argument::Boolean(arg_name, extract_bits!(header, 32, 32) == 1))
        }
    }
}