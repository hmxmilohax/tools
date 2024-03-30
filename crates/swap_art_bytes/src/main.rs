use std::error::Error;
use std::path::Path;

use clap::Parser;

use swap_art_bytes::swap_art_bytes;

#[derive(clap::Parser)]
struct Args {
    input_file: Box<Path>,
    output_file: Box<Path>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let mut buf = std::fs::read(args.input_file)?;
    swap_art_bytes(&mut buf);
    std::fs::write(args.output_file, buf)?;
    Ok(())
}
