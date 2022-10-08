use super::error::Result;
use std::io::Read;

pub fn read_u8<R: Read>(reader: &mut R) -> Result<u8> {
    let mut length = [0];
    reader.read_exact(&mut length)?;
    Ok(length[0])
}

pub fn read_u16_be<R: Read>(reader: &mut R) -> Result<u16> {
    let mut data = [0; 2];
    reader.read_exact(&mut data)?;
    Ok(u8s_be_to_u16(&data))
}

fn u8s_be_to_u16(bytes: &[u8]) -> u16 {
    let msb = bytes[0] as u16;
    let lsb = bytes[1] as u16;
    (msb << 8) + lsb
}
