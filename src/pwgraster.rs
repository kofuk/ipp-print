use std::error::Error;
use std::io::prelude::*;

pub fn read_raster<R>(reader: &mut R) -> Result<(), Box<dyn Error>> where R: Read {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    println!("{}", String::from_utf8(buf.to_vec()).unwrap());

    Ok(())
}
