use super::error::Error;
use super::error::Result;
use super::util::{read_u16_be, read_u8};
use std::io::Read;

pub enum Marker {
    StartOfImage,
    ApplicationSegment(u8, u16),
    Comment(u16),
    DefineQuantizationTable(u16),
    StartOfFrame(u8, u16),
    DefineHuffmanTable(u16),
    StartOfScan(u16),
    EndOfImage,
}

impl Marker {
    pub fn from_reader<R: Read>(reader: &mut R) -> Result<Self> {
        loop {
            // This should be an error as the JPEG spec doesn't allow extraneous data between marker segments.
            // libjpeg allows this though and there are images in the wild utilising it, so we are
            // forced to support this behavior.
            // Sony Ericsson P990i is an example of a device which produce this sort of JPEGs.
            while read_u8(reader)? != 0xff {}

            // Section B.1.1.2
            // All markers are assigned two-byte codes: an X’FF’ byte followed by a
            // byte which is not equal to 0 or X’FF’ (see Table B.1). Any marker may
            // optionally be preceded by any number of fill bytes, which are bytes
            // assigned code X’FF’.
            let mut byte = read_u8(reader)?;

            // Section B.1.1.2
            // "Any marker may optionally be preceded by any number of fill bytes, which are bytes assigned code X’FF’."
            while byte == 0xff {
                byte = read_u8(reader)?;
            }

            // Section B.1.1.4
            if byte != 0x00 && byte != 0xff {
                return match byte {
                    0xd8 => Ok(Self::StartOfImage),
                    0xe0 => Ok(Self::ApplicationSegment(0, read_u16_be(reader)?)),
                    0xe1 => Ok(Self::ApplicationSegment(1, read_u16_be(reader)?)),
                    0xe2 => Ok(Self::ApplicationSegment(2, read_u16_be(reader)?)),
                    0xe3 => Ok(Self::ApplicationSegment(3, read_u16_be(reader)?)),
                    0xe4 => Ok(Self::ApplicationSegment(4, read_u16_be(reader)?)),
                    0xe5 => Ok(Self::ApplicationSegment(5, read_u16_be(reader)?)),
                    0xe6 => Ok(Self::ApplicationSegment(6, read_u16_be(reader)?)),
                    0xe7 => Ok(Self::ApplicationSegment(7, read_u16_be(reader)?)),
                    0xe8 => Ok(Self::ApplicationSegment(8, read_u16_be(reader)?)),
                    0xe9 => Ok(Self::ApplicationSegment(9, read_u16_be(reader)?)),
                    0xfe => Ok(Self::Comment(read_u16_be(reader)?)),
                    0xdb => Ok(Self::DefineQuantizationTable(read_u16_be(reader)?)),
                    0xc0 => Ok(Self::StartOfFrame(0, read_u16_be(reader)?)),
                    0xc4 => Ok(Self::DefineHuffmanTable(read_u16_be(reader)?)),
                    0xda => Ok(Self::StartOfScan(read_u16_be(reader)?)),
                    0xd9 => Ok(Self::EndOfImage),
                    _ => Err(Error::Unsupported("Unsupported marker")),
                };
            }
        }
    }
}
