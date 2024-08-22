use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};

pub fn preadn(f: &mut File, buf: &mut [u8], offset: i64) -> io::Result<()> {
    let whence = if offset >= 0 {
        SeekFrom::Start(offset as u64)
    } else {
        SeekFrom::End(offset)
    };

    let _ = f.seek(whence)?;
    f.read_exact(buf)
}
