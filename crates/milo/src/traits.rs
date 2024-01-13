use std::error::Error;
use std::fs::File;
pub trait Load {
    fn load(&mut self, f: File) -> Result<(), Box<dyn Error>>;
}

pub trait Save {
    fn save(&mut self, f: File) -> Result<(), Box<dyn Error>>;
}

pub trait Port {
    fn import(&mut self, f: File) -> Result<(), Box<dyn Error>>;
    fn export(&mut self, f: File) -> Result<(), Box<dyn Error>>;
}
