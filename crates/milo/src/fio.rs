use std::fs::File;
use std::error::Error;
use std::io::Read;
use byteorder::{LittleEndian, BigEndian, ReadBytesExt};

fn readlen(f: &mut File, len: usize) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut buf = vec![0u8; len];
    f.read_exact(&mut buf)?;
    Ok(buf)
}

pub fn readstr(src: &mut File) -> Result<String, Box<dyn Error>> {
    let mut ret: String = String::with_capacity(256);
    for _ in 0..256 { // 256 is a good length limit, right
        let asciidiot = readlen(src, 1)?[0];
        if asciidiot == 0 {
            ret.shrink_to_fit();
            break
        }
        let test = char::from_u32(asciidiot as u32);
        ret.push(test.expect("found eof"));
    }
    Ok(ret)
}

pub fn read_u32(f: &mut File, little_endian: bool) -> Result<u32, Box<dyn Error>> {
    if little_endian { Ok(f.read_u32::<LittleEndian>()?) }
    else { Ok(f.read_u32::<BigEndian>()?) }
}

pub fn read_i32(f: &mut File, little_endian: bool) -> Result<i32, Box<dyn Error>> {
    if little_endian { Ok(f.read_i32::<LittleEndian>()?) }
    else { Ok(f.read_i32::<BigEndian>()?) }
}

pub fn read_u16(f: &mut File, little_endian: bool) -> Result<u16, Box<dyn Error>> {
    if little_endian { Ok(f.read_u16::<LittleEndian>()?) }
    else { Ok(f.read_u16::<BigEndian>()?) }
}

pub fn read_i16(f: &mut File, little_endian: bool) -> Result<i16, Box<dyn Error>> {
    if little_endian { Ok(f.read_i16::<LittleEndian>()?) }
    else { Ok(f.read_i16::<BigEndian>()?) }
}

pub fn read_u8(f: &mut File) -> Result<u8, Box<dyn Error>> { Ok(f.read_u8()?) }
pub fn read_i8(f: &mut File) -> Result<i8, Box<dyn Error>> { Ok(f.read_i8()?) }

pub fn read_f32(f: &mut File, little_endian: bool) -> Result<f32, Box<dyn Error>> {
    if little_endian { Ok(f.read_f32::<LittleEndian>()?) }
    else { Ok(f.read_f32::<BigEndian>()?) }
}
