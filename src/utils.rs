use byteorder::ReadBytesExt;
use std::io::Read;

pub fn vari32(r: &mut impl Read) -> anyhow::Result<i32> {
    let b = r.read_u8()? as i32;
    if b & 0x80 == 0 {
        Ok(b & 0x7F)
    } else if b & 0x40 == 0 {
        let v = r.read_u8()? as i32 | ((b & 31) << 8);
        Ok(if b & 0x20 == 0 { v } else { -v })
    } else {
        let c = r.read_u8()? as i32;
        let d = r.read_u8()? as i32;
        let e = r.read_u8()? as i32;
        let v = ((b & 31) << 24) | (c << 16) | (d << 8) | e;
        Ok(if b & 0x20 == 0 { v } else { -v })
    }
}

pub fn varu32(r: &mut impl Read) -> anyhow::Result<u32> {
    let i = vari32(r)?;
    if i < 0 {
        anyhow::bail!("Negative index")
    } else {
        Ok(i as u32)
    }
}
