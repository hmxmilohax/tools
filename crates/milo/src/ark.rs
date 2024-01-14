use std::error::Error;
use std::fs::File;
use std::mem::ManuallyDrop;

use crate::fio::read_u32;
use crate::traits::Load;
pub mod amp;
pub mod freq;

pub union ArkTypes {
    empty: Option<u32>,
    freq: freq::FreqArchive,
    amp: ManuallyDrop<amp::AmpArchive>,
}

pub fn load_ark_file(f: &mut File) -> Result<Option<ArkTypes>, Box<dyn Error>> {
    let vercheck = read_u32(f, true)?;
    let mut ark = ArkTypes {empty: Some(0)};
    match vercheck {
        0x004B5241 => {
            let mut freq = freq::FreqArchive::new();
            freq.load(f)?;
            ark.freq = freq;
        }
        0..=2 => {
            let mut amp = amp::AmpArchive::new();
            amp.load(f)?;
            ark.amp = ManuallyDrop::new(amp);
        }
        _ => println!("unrecognized ark version. if gh1 or later, use the header and not the part.")
    }
    Ok(Some(ark))
}
