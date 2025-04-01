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
            StringRef::Inline(read_aligned_str(reader, (arg_name & 0x7FFF) as usize)?)
        };

        match arg_type {
            ArgumentType::Null => Ok(Argument::Null(arg_name)),
            ArgumentType::Int32 => Ok(Argument::Int32(arg_name, extract_bits!(header, 32, 63) as i32)),
            ArgumentType::UInt32 => Ok(Argument::UInt32(arg_name, extract_bits!(header, 32, 63) as u32)),
            ArgumentType::Int64 => Ok(Argument::Int64(arg_name, read_u64_word(reader)? as i64)),
            ArgumentType::UInt64 => Ok(Argument::UInt64(arg_name, read_u64_word(reader)?)),
            ArgumentType::Float => Ok(Argument::Float(arg_name, f64::from_bits(read_u64_word(reader)?))),
            ArgumentType::Str => {
                let arg_value = extract_bits!(header, 32, 47) as u16;
                let arg_value = if StringRef::field_is_ref(arg_value) {
                    StringRef::Ref(arg_value)
                } else {
                    StringRef::Inline(read_aligned_str(reader, (arg_value & 0x7FFF) as usize)?)
                };
                Ok(Argument::Str(arg_name, arg_value))
            }
            ArgumentType::Pointer => Ok(Argument::Pointer(arg_name, read_u64_word(reader)?)),
            ArgumentType::KernelObjectId => Ok(Argument::KernelObjectId(arg_name, read_u64_word(reader)?)),
            ArgumentType::Boolean => Ok(Argument::Boolean(arg_name, extract_bits!(header, 32, 32) == 1))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    // Helper function to create a header word for argument records
    fn create_argument_header(
        arg_type: u8,
        arg_size: u16,
        arg_name: u16,
        data: u32,
    ) -> u64 {
        let mut header: u64 = 0;
        header |= (arg_type as u64) & 0xF; // bits 0-3
        header |= ((arg_size as u64) & 0xFFF) << 4; // bits 4-15
        header |= ((arg_name as u64) & 0xFFFF) << 16; // bits 16-31
        header |= ((data as u64) & 0xFFFFFFFF) << 32; // bits 32-63
        header
    }

    #[test]
    fn test_null_argument() -> Result<()> {
        // Null argument with reference name
        let arg_name_ref = 0x0123; // Reference to string at index 0x123
        let header = create_argument_header(0, 1, arg_name_ref, 0);
        
        let mut data = Vec::new();
        data.extend_from_slice(&header.to_le_bytes());
        
        let mut cursor = Cursor::new(data);
        let arg = Argument::read(&mut cursor)?;
        
        match arg {
            Argument::Null(name) => {
                match name {
                    StringRef::Ref(idx) => assert_eq!(idx, 0x0123),
                    _ => panic!("Expected string reference name"),
                }
            },
            _ => panic!("Expected Null argument"),
        }
        
        Ok(())
    }

    #[test]
    fn test_int32_argument() -> Result<()> {
        // Int32 argument with value -42
        let arg_name = 0x0042; // Reference to string at index 0x42
        let value: i32 = -42;
        let header = create_argument_header(1, 1, arg_name, value as u32);
        
        let mut data = Vec::new();
        data.extend_from_slice(&header.to_le_bytes());
        
        let mut cursor = Cursor::new(data);
        let arg = Argument::read(&mut cursor)?;
        
        match arg {
            Argument::Int32(name, val) => {
                assert_eq!(name, StringRef::Ref(0x0042));
                assert_eq!(val, -42);
            },
            _ => panic!("Expected Int32 argument"),
        }
        
        // Max positive value
        let max_val: i32 = i32::MAX;
        let header = create_argument_header(1, 1, arg_name, max_val as u32);
        
        let mut data = Vec::new();
        data.extend_from_slice(&header.to_le_bytes());
        
        let mut cursor = Cursor::new(data);
        let arg = Argument::read(&mut cursor)?;
        
        match arg {
            Argument::Int32(_, val) => assert_eq!(val, i32::MAX),
            _ => panic!("Expected Int32 argument"),
        }
        
        // Min negative value
        let min_val: i32 = i32::MIN;
        let header = create_argument_header(1, 1, arg_name, min_val as u32);
        
        let mut data = Vec::new();
        data.extend_from_slice(&header.to_le_bytes());
        
        let mut cursor = Cursor::new(data);
        let arg = Argument::read(&mut cursor)?;
        
        match arg {
            Argument::Int32(_, val) => assert_eq!(val, i32::MIN),
            _ => panic!("Expected Int32 argument"),
        }
        
        Ok(())
    }

    #[test]
    fn test_uint32_argument() -> Result<()> {
        // UInt32 argument with value 42
        let arg_name = 0x0052; // Reference to string at index 0x52
        let value: u32 = 42;
        let header = create_argument_header(2, 1, arg_name, value);
        
        let mut data = Vec::new();
        data.extend_from_slice(&header.to_le_bytes());
        
        let mut cursor = Cursor::new(data);
        let arg = Argument::read(&mut cursor)?;
        
        match arg {
            Argument::UInt32(name, val) => {
                assert_eq!(name, StringRef::Ref(0x0052));
                assert_eq!(val, 42);
            },
            _ => panic!("Expected UInt32 argument"),
        }
        
        // Max value
        let max_val: u32 = u32::MAX;
        let header = create_argument_header(2, 1, arg_name, max_val);
        
        let mut data = Vec::new();
        data.extend_from_slice(&header.to_le_bytes());
        
        let mut cursor = Cursor::new(data);
        let arg = Argument::read(&mut cursor)?;
        
        match arg {
            Argument::UInt32(_, val) => assert_eq!(val, u32::MAX),
            _ => panic!("Expected UInt32 argument"),
        }
        
        Ok(())
    }

    #[test]
    fn test_int64_argument() -> Result<()> {
        // Int64 argument with value -1234567890123
        let arg_name = 0x0062; // Reference to string at index 0x62
        let value: i64 = -1234567890123;
        let header = create_argument_header(3, 2, arg_name, 0); // Size 2 = 16 bytes (header + value)
        
        let mut data = Vec::new();
        data.extend_from_slice(&header.to_le_bytes());
        data.extend_from_slice(&(value as u64).to_le_bytes());
        
        let mut cursor = Cursor::new(data);
        let arg = Argument::read(&mut cursor)?;
        
        match arg {
            Argument::Int64(name, val) => {
                assert_eq!(name, StringRef::Ref(0x0062));
                assert_eq!(val, -1234567890123);
            },
            _ => panic!("Expected Int64 argument"),
        }
        
        // Max positive value
        let max_val: i64 = i64::MAX;
        let header = create_argument_header(3, 2, arg_name, 0);
        
        let mut data = Vec::new();
        data.extend_from_slice(&header.to_le_bytes());
        data.extend_from_slice(&(max_val as u64).to_le_bytes());
        
        let mut cursor = Cursor::new(data);
        let arg = Argument::read(&mut cursor)?;
        
        match arg {
            Argument::Int64(_, val) => assert_eq!(val, i64::MAX),
            _ => panic!("Expected Int64 argument"),
        }
        
        // Min negative value
        let min_val: i64 = i64::MIN;
        let header = create_argument_header(3, 2, arg_name, 0);
        
        let mut data = Vec::new();
        data.extend_from_slice(&header.to_le_bytes());
        data.extend_from_slice(&(min_val as u64).to_le_bytes());
        
        let mut cursor = Cursor::new(data);
        let arg = Argument::read(&mut cursor)?;
        
        match arg {
            Argument::Int64(_, val) => assert_eq!(val, i64::MIN),
            _ => panic!("Expected Int64 argument"),
        }
        
        Ok(())
    }

    #[test]
    fn test_uint64_argument() -> Result<()> {
        // UInt64 argument with value 12345678901234
        let arg_name = 0x0072; // Reference to string at index 0x72
        let value: u64 = 12345678901234;
        let header = create_argument_header(4, 2, arg_name, 0); // Size 2 = 16 bytes (header + value)
        
        let mut data = Vec::new();
        data.extend_from_slice(&header.to_le_bytes());
        data.extend_from_slice(&value.to_le_bytes());
        
        let mut cursor = Cursor::new(data);
        let arg = Argument::read(&mut cursor)?;
        
        match arg {
            Argument::UInt64(name, val) => {
                assert_eq!(name, StringRef::Ref(0x0072));
                assert_eq!(val, 12345678901234);
            },
            _ => panic!("Expected UInt64 argument"),
        }
        
        // Max value
        let max_val: u64 = u64::MAX;
        let header = create_argument_header(4, 2, arg_name, 0);
        
        let mut data = Vec::new();
        data.extend_from_slice(&header.to_le_bytes());
        data.extend_from_slice(&max_val.to_le_bytes());
        
        let mut cursor = Cursor::new(data);
        let arg = Argument::read(&mut cursor)?;
        
        match arg {
            Argument::UInt64(_, val) => assert_eq!(val, u64::MAX),
            _ => panic!("Expected UInt64 argument"),
        }
        
        Ok(())
    }
    
    #[test]
    fn test_float_argument() -> Result<()> {
        // Float argument with value 3.14159
        let arg_name = 0x0082; // Reference to string at index 0x82
        let value: f64 = 3.14159;
        let header = create_argument_header(5, 2, arg_name, 0); // Size 2 = 16 bytes (header + value)
        
        let mut data = Vec::new();
        data.extend_from_slice(&header.to_le_bytes());
        data.extend_from_slice(&value.to_bits().to_le_bytes());
        
        let mut cursor = Cursor::new(data);
        let arg = Argument::read(&mut cursor)?;
        
        match arg {
            Argument::Float(name, val) => {
                assert_eq!(name, StringRef::Ref(0x0082));
                assert!((val - 3.14159).abs() < f64::EPSILON);
            },
            _ => panic!("Expected Float argument"),
        }
        
        // Special values: NaN
        let nan_val = f64::NAN;
        let header = create_argument_header(5, 2, arg_name, 0);
        
        let mut data = Vec::new();
        data.extend_from_slice(&header.to_le_bytes());
        data.extend_from_slice(&nan_val.to_bits().to_le_bytes());
        
        let mut cursor = Cursor::new(data);
        let arg = Argument::read(&mut cursor)?;
        
        match arg {
            Argument::Float(_, val) => assert!(val.is_nan()),
            _ => panic!("Expected Float argument with NaN value"),
        }
        
        // Special values: Infinity
        let inf_val = f64::INFINITY;
        let header = create_argument_header(5, 2, arg_name, 0);
        
        let mut data = Vec::new();
        data.extend_from_slice(&header.to_le_bytes());
        data.extend_from_slice(&inf_val.to_bits().to_le_bytes());
        
        let mut cursor = Cursor::new(data);
        let arg = Argument::read(&mut cursor)?;
        
        match arg {
            Argument::Float(_, val) => assert!(val.is_infinite() && val.is_sign_positive()),
            _ => panic!("Expected Float argument with Infinity value"),
        }
        
        Ok(())
    }
    
    #[test]
    fn test_str_argument() -> Result<()> {
        // String argument with reference name and reference value
        let arg_name_ref = 0x0123; // Reference to string at index 0x123
        let arg_value_ref = 0x0456; // Reference to string at index 0x456
        let header = create_argument_header(6, 1, arg_name_ref, arg_value_ref as u32);
        
        let mut data = Vec::new();
        data.extend_from_slice(&header.to_le_bytes());
        
        let mut cursor = Cursor::new(data);
        let arg = Argument::read(&mut cursor)?;
        
        match arg {
            Argument::Str(name, value) => {
                match (name, value) {
                    (StringRef::Ref(n), StringRef::Ref(v)) => {
                        assert_eq!(n, 0x0123);
                        assert_eq!(v, 0x0456);
                    },
                    _ => panic!("Expected string references for both name and value"),
                }
            },
            _ => panic!("Expected Str argument"),
        }
        
        Ok(())
    }
    
    #[test]
    fn test_pointer_argument() -> Result<()> {
        // Pointer argument with reference name
        let arg_name = 0x0099; // Reference to string at index 0x99
        let value: u64 = 0xDEADBEEFCAFEBABE;
        let header = create_argument_header(7, 2, arg_name, 0); // Size 2 = 16 bytes (header + value)
        
        let mut data = Vec::new();
        data.extend_from_slice(&header.to_le_bytes());
        data.extend_from_slice(&value.to_le_bytes());
        
        let mut cursor = Cursor::new(data);
        let arg = Argument::read(&mut cursor)?;
        
        match arg {
            Argument::Pointer(name, val) => {
                assert_eq!(name, StringRef::Ref(0x0099));
                assert_eq!(val, 0xDEADBEEFCAFEBABE);
            },
            _ => panic!("Expected Pointer argument"),
        }
        
        // Null pointer
        let value: u64 = 0;
        let header = create_argument_header(7, 2, arg_name, 0);
        
        let mut data = Vec::new();
        data.extend_from_slice(&header.to_le_bytes());
        data.extend_from_slice(&value.to_le_bytes());
        
        let mut cursor = Cursor::new(data);
        let arg = Argument::read(&mut cursor)?;
        
        match arg {
            Argument::Pointer(_, val) => assert_eq!(val, 0),
            _ => panic!("Expected Pointer argument"),
        }
        
        Ok(())
    }
    
    #[test]
    fn test_kernel_object_id_argument() -> Result<()> {
        // KernelObjectId argument
        let arg_name = 0x0099; // Reference to string at index 0x99
        let value: u64 = 0x1234567890ABCDEF;
        let header = create_argument_header(8, 2, arg_name, 0); // Size 2 = 16 bytes (header + value)
        
        let mut data = Vec::new();
        data.extend_from_slice(&header.to_le_bytes());
        data.extend_from_slice(&value.to_le_bytes());
        
        let mut cursor = Cursor::new(data);
        let arg = Argument::read(&mut cursor)?;
        
        match arg {
            Argument::KernelObjectId(name, val) => {
                assert_eq!(name, StringRef::Ref(0x0099));
                assert_eq!(val, 0x1234567890ABCDEF);
            },
            _ => panic!("Expected KernelObjectId argument"),
        }
        
        Ok(())
    }
    
    #[test]
    fn test_boolean_argument() -> Result<()> {
        // Boolean argument: true
        let arg_name = 0x00AA; // Reference to string at index 0xAA
        let value: u32 = 1; // 1 = true
        let header = create_argument_header(9, 1, arg_name, value);
        
        let mut data = Vec::new();
        data.extend_from_slice(&header.to_le_bytes());
        
        let mut cursor = Cursor::new(data);
        let arg = Argument::read(&mut cursor)?;
        
        match arg {
            Argument::Boolean(name, val) => {
                assert_eq!(name, StringRef::Ref(0x00AA));
                assert_eq!(val, true);
            },
            _ => panic!("Expected Boolean argument"),
        }
        
        // Boolean argument: false
        let value: u32 = 0; // 0 = false
        let header = create_argument_header(9, 1, arg_name, value);
        
        let mut data = Vec::new();
        data.extend_from_slice(&header.to_le_bytes());
        
        let mut cursor = Cursor::new(data);
        let arg = Argument::read(&mut cursor)?;
        
        match arg {
            Argument::Boolean(_, val) => assert_eq!(val, false),
            _ => panic!("Expected Boolean argument"),
        }
        
        Ok(())
    }
    
    #[test]
    fn test_null_argument_with_inline_name() -> Result<()> {
        // Create a buffer with header for null argument with inline name
        let name_str = "myname";
        let name_len = name_str.len() as u16;
        
        // We need to create a header where:
        // - bits 0-3: Argument type (0 for Null)
        // - bits 4-15: Size (2 words = 16 bytes: 8 for header, 8 for inline string)
        // - bits 16-31: Name field with 0x8000 bit set 
        let arg_name_field = name_len | 0x8000; 
        let header = create_argument_header(0, 2, arg_name_field, 0);
        
        let mut data = Vec::new();
        data.extend_from_slice(&header.to_le_bytes());
        
        // Add inline string data with padding to 8 bytes
        let mut padded_name = name_str.as_bytes().to_vec();
        padded_name.resize(8, 0); // Pad with zeros to 8 bytes
        data.extend_from_slice(&padded_name);
        
        let mut cursor = Cursor::new(data);
        let arg = Argument::read(&mut cursor)?;
        
        match arg {
            Argument::Null(name) => {
                match name {
                    StringRef::Inline(s) => assert_eq!(s, name_str),
                    _ => panic!("Expected inline string name"),
                }
            },
            _ => panic!("Expected Null argument"),
        }
        
        Ok(())
    }
    
    #[test]
    fn test_str_argument_with_inline_values() -> Result<()> {
        // Test string argument with inline name and inline value
        let name_str = "name";
        let value_str = "value";
        let name_len = name_str.len() as u16;
        let value_len = value_str.len() as u16;
        
        // We need to create a header where:
        // - bits 0-3: Argument type (6 for Str)
        // - bits 4-15: Size (3 words = 24 bytes: 8 for header, 8 for inline name, 8 for inline value)
        // - bits 16-31: Name field with 0x8000 bit set (inline) + length
        // - bits 32-47: Value field with 0x8000 bit set (inline) + length
        let arg_name_field = name_len | 0x8000; 
        let arg_value_field = value_len | 0x8000; 
        
        // Combine name and value fields into data field (bits 32-63)
        let data_field = arg_value_field as u32;
        
        let header = create_argument_header(6, 3, arg_name_field, data_field);
        
        let mut data = Vec::new();
        data.extend_from_slice(&header.to_le_bytes());
        
        // Add inline name string data with padding to 8 bytes
        let mut padded_name = name_str.as_bytes().to_vec();
        padded_name.resize(8, 0); // Pad with zeros to 8 bytes
        data.extend_from_slice(&padded_name);
        
        // Add inline value string data with padding to 8 bytes
        let mut padded_value = value_str.as_bytes().to_vec();
        padded_value.resize(8, 0); // Pad with zeros to 8 bytes
        data.extend_from_slice(&padded_value);
        
        let mut cursor = Cursor::new(data);
        let arg = Argument::read(&mut cursor)?;
        
        match arg {
            Argument::Str(name, value) => {
                match (name, value) {
                    (StringRef::Inline(n), StringRef::Inline(v)) => {
                        assert_eq!(n, name_str);
                        assert_eq!(v, value_str);
                    },
                    _ => panic!("Expected inline strings for both name and value"),
                }
            },
            _ => panic!("Expected Str argument"),
        }
        
        Ok(())
    }
    
    #[test]
    fn test_str_argument_mixed_inline_ref() -> Result<()> {
        // Test string argument with inline name and reference value
        let name_str = "inname";
        let name_len = name_str.len() as u16;
        let value_ref = 0x0456; // Reference to string at index 0x456
        
        // Header setup:
        // - bits 0-3: Argument type (6 for Str)
        // - bits 4-15: Size (2 words = 16 bytes: 8 for header, 8 for inline name)
        // - bits 16-31: Name field with 0x8000 bit set (inline) + length
        // - bits 32-47: Value field (reference) with 0x8000 bit set
        let arg_name_field = name_len | 0x8000; 
        
        // Value field in data portion (bits 32-63)
        let data_field = value_ref as u32; 
        
        let header = create_argument_header(6, 2, arg_name_field, data_field);
        
        let mut data = Vec::new();
        data.extend_from_slice(&header.to_le_bytes());
        
        // Add inline name string data with padding to 8 bytes
        let mut padded_name = name_str.as_bytes().to_vec();
        padded_name.resize(8, 0); // Pad with zeros to 8 bytes
        data.extend_from_slice(&padded_name);
        
        let mut cursor = Cursor::new(data);
        let arg = Argument::read(&mut cursor)?;
        
        match arg {
            Argument::Str(name, value) => {
                match (name, value) {
                    (StringRef::Inline(n), StringRef::Ref(v)) => {
                        assert_eq!(n, name_str);
                        assert_eq!(v, value_ref);
                    },
                    _ => panic!("Expected inline name and reference value"),
                }
            },
            _ => panic!("Expected Str argument"),
        }
        
        Ok(())
    }
    
    #[test]
    fn test_pointer_argument_with_inline_name() -> Result<()> {
        // Create a pointer argument with inline name
        let name_str = "ptr";
        let name_len = name_str.len() as u16;
        let pointer_value: u64 = 0xDEADBEEFCAFEBABE;
        
        // Header setup:
        // - bits 0-3: Argument type (7 for Pointer)
        // - bits 4-15: Size (3 words = 24 bytes: 8 for header, 8 for inline name, 8 for value)
        // - bits 16-31: Name field with 0x8000 bit set (inline) + length
        let arg_name_field = name_len | 0x8000; 
        
        let header = create_argument_header(7, 3, arg_name_field, 0);
        
        let mut data = Vec::new();
        data.extend_from_slice(&header.to_le_bytes());
        
        // Add inline name string data with padding to 8 bytes
        let mut padded_name = name_str.as_bytes().to_vec();
        padded_name.resize(8, 0); // Pad with zeros to 8 bytes
        data.extend_from_slice(&padded_name);
        
        // Add pointer value
        data.extend_from_slice(&pointer_value.to_le_bytes());
        
        let mut cursor = Cursor::new(data);
        let arg = Argument::read(&mut cursor)?;
        
        match arg {
            Argument::Pointer(name, val) => {
                match name {
                    StringRef::Inline(n) => assert_eq!(n, name_str),
                    _ => panic!("Expected inline string name"),
                }
                assert_eq!(val, pointer_value);
            },
            _ => panic!("Expected Pointer argument"),
        }
        
        Ok(())
    }
    
    #[test]
    fn test_invalid_argument_type() {
        // Try to parse an invalid argument type (10 is beyond the valid range)
        let arg_name = 0x00BB; // Reference to string at index 0xBB
        let header = create_argument_header(10, 1, arg_name, 0);
        
        let mut data = Vec::new();
        data.extend_from_slice(&header.to_le_bytes());
        
        let mut cursor = Cursor::new(data);
        let result = Argument::read(&mut cursor);
        
        assert!(result.is_err());
        
        // Verify the error is of the expected type
        match result {
            Err(crate::FtfError::InvalidArgumentType(e)) => {
                assert_eq!(e.0, 10);
            },
            _ => panic!("Expected InvalidArgumentType error"),
        }
    }
}