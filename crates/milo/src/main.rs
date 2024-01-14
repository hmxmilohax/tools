use std::fs::File;
use std::error::Error;
use std::path::PathBuf;
use clap::Parser;
use milo::ark;
use milo::traits::Load;

#[derive(clap::Parser)]
struct Args {
    input: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("Hello, world!");
    let args = Args::parse();
    let mut infile = File::open(args.input)?;
    let mut freqindasheets = ark::freq::FreqArchive::new();
    freqindasheets.load(&mut infile, 0)?;
    println!("{}", freqindasheets);
    Ok(())
}
