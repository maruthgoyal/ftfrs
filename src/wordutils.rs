use crate::Result;
use std::io::Read;

pub fn read_u64_word<U: Read>(reader: &mut U) -> Result<u64> {
    let mut buf = [0; 8];
    reader.read_exact(&mut buf)?;
    Ok(u64::from_le_bytes(buf))
}
pub fn read_aligned_str<U: Read>(reader: &mut U, len: usize) -> Result<String> {
    let bytes_to_read = ((len + 7) / 8) * 8;
    let mut buf = vec![0; bytes_to_read];
    reader.read_exact(&mut buf)?;

    if len % 8 == 0 {
        Ok(String::from_utf8(buf)?)
    } else {
        // get rid of 0-padding
        let res = buf[0..len].to_vec();
        Ok(String::from_utf8(res)?)
    }
}

pub fn pad_to_multiple_of_8(input: &[u8]) -> Vec<u8> {
    let remainder = input.len() % 8;

    // If the length is already a multiple of 8, no padding needed
    if remainder == 0 {
        return input.to_vec();
    }

    // Calculate how many padding bytes are needed
    let padding_needed = 8 - remainder;

    // Create a new vector with the original data plus padding
    let mut padded = Vec::with_capacity(input.len() + padding_needed);
    padded.extend_from_slice(input);
    padded.extend(vec![0; padding_needed]);

    padded
}
