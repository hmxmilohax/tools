use std::error::Error;
use std::fs::File;

use crate::fio::read_u32;
use crate::traits::Load;
pub mod amp;
pub mod freq;

pub enum ArkTypes {
    FreqArk(freq::FreqArchive),
    AmpArk(amp::AmpArchive),
}

pub fn load_ark_file(f: &mut File) -> Result<ArkTypes, Box<dyn Error>> {
    let vercheck = read_u32(f, true)?;
    let ark: ArkTypes;
    match vercheck {
        0x004B5241 => {
            let mut freq = freq::FreqArchive::new();
            freq.load(f, 0)?;
            ark = ArkTypes::FreqArk(freq);
        }
        0..=2 => {
            let mut amp = amp::AmpArchive::new();
            amp.load(f, 0)?;
            ark = ArkTypes::AmpArk(amp);
        }
        _ => {
            println!("unrecognized ark version. if gh1 or later, use the .hdr and not the .ark");
            return Err::<_, Box<dyn Error>>("unkver".into());
        }
    }
    Ok(ark)
}
