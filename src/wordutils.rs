use crate::Result;
use std::io::{Read, Write};

pub fn read_u64_word<U: Read>(reader: &mut U) -> Result<u64> {
    let mut buf = [0; 8];
    reader.read_exact(&mut buf)?;
    Ok(u64::from_ne_bytes(buf))
}
pub fn read_aligned_str<U: Read>(reader: &mut U, len: usize) -> Result<String> {
    let bytes_to_read = len.div_ceil(8) * 8;
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

fn get_zero_vec(n: u8) -> &'static [u8] {
    match n {
        1 => &[0_u8],
        2 => &[0_u8; 2],
        3 => &[0_u8; 3],
        4 => &[0_u8; 4],
        5 => &[0_u8; 5],
        6 => &[0_u8; 6],
        7 => &[0_u8; 7],
        _ => unreachable!("Should never request zeros outsidse 1-7 range"),
    }
}

pub fn pad_and_write_string<W: Write>(writer: &mut W, input: &str) -> Result<()> {
    let bytes = input.as_bytes();
    writer.write_all(bytes)?;

    let remainder = bytes.len() % 8;
    if remainder != 0 {
        let num_zeros = 8 - remainder;
        let zeros = get_zero_vec(num_zeros as u8);
        writer.write_all(zeros)?;
    }
    Ok(())
}
