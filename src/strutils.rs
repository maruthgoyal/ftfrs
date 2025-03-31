use crate::RecordHeader;
use anyhow::Result;
use std::io::Read;

pub fn read_aligned_str<U: Read>(reader: &mut U, len: usize, header: &RecordHeader) -> Result<String> {
    let bytes_to_read = (header.size() - 1) * 8;
    let mut buf = vec![0; bytes_to_read as usize];
    reader.read_exact(&mut buf)?;

    if len % 8 == 0 {
        Ok(String::from_utf8(buf)?)
    } else {
        // get rid of 0-padding
        let res = buf[0..len].to_vec();
        Ok(String::from_utf8(res)?)
    }
}
