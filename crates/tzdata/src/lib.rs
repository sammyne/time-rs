use std::io::Read;

/// zipped data ofr zoneinfo.zip built from static/zoneinfo folder.
const ZIPDATA: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/zoneinfo.zip"));

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("corrupted")]
    Corrupted,
    #[error("not found")]
    NotFound,
    #[error("unsupported compression in embedded tzdata")]
    UnsupportedCompression,
}

/// returns the contents of the file with the given name in the embedded uncompressed zip file zoneinfo.zip.
pub fn load(name: &str) -> Result<&'static [u8], Error> {
    //const ZECHEADER: usize = 0x06054b50;
    const ZCHEADER: usize = 0x02014b50;
    const ZTAILSIZE: usize = 22;
    //const ZHEADERSIZE: usize = 30;
    const ZHEADER: usize = 0x04034b50;

    let z = ZIPDATA;
    let name = name.as_bytes();

    let idx = z.len() - ZTAILSIZE;
    let n = get2s(&z[idx + 10..]);
    let mut idx = get4s(&z[idx + 16..]);

    for _ in 0..n {
        if get4s(&z[idx..]) != ZCHEADER {
            break;
        }

        let meth = get2s(&z[idx + 10..]);
        let size = get4s(&z[idx + 24..]);
        let namelen = get2s(&z[idx + 28..]);
        let xlen = get2s(&z[idx + 30..]);
        let fclen = get2s(&z[idx + 32..]);
        let off = get4s(&z[idx + 42..]);
        let zname = &z[idx + 46..idx + 46 + namelen];
        idx += 46 + namelen + xlen + fclen;
        if zname != name {
            continue;
        }
        if meth != 0 {
            return Err(Error::UnsupportedCompression);
        }

        idx = off;
        if get4s(&z[idx..]) != ZHEADER
            || get2s(&z[idx..]) != meth
            || get2s(&z[idx..]) != namelen
            || &z[idx + 30..idx + 30 + namelen] != name
        {
            return Err(Error::Corrupted);
        }
        let xlen = get2s(&z[idx + 28..]);
        idx += 30 + namelen + xlen;
        return Ok(&z[idx..idx + size]);
    }

    Err(Error::NotFound)
}

/// returns the little-endian 16-bit value at the start of s.
fn get2s(mut s: &[u8]) -> usize {
    let mut buf = [0u8; 2];
    if s.read_exact(&mut buf).is_err() {
        0
    } else {
        u16::from_le_bytes(buf) as usize
    }
}

/// returns the little-endian 32-bit value at the start of s.
fn get4s(mut s: &[u8]) -> usize {
    let mut buf = [0u8; 4];
    if s.read_exact(&mut buf).is_err() {
        0
    } else {
        u32::from_le_bytes(buf) as usize
    }
}
