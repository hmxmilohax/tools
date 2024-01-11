use std::error::Error;
use std::mem::ManuallyDrop;
mod amp;
mod freq;

pub union ArkTypes {
    amp: ManuallyDrop<amp::AmpArchive>,
}

pub fn load_ark_file() -> Result<Option<ArkTypes>, Box<dyn Error>> {
    Ok(None)
}
