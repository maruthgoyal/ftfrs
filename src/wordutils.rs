use std::io::Read;
use crate::Result;

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
