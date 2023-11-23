use std::fs::File;
use std::io::{self, Read, Write};

pub fn create_dir(path: &str) -> io::Result<()> {
    std::fs::create_dir_all(path)?;
    Ok(())
}

pub fn read_file(filename: &str) -> io::Result<Vec<u8>> {
    let mut file = File::open(filename)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

pub fn write_file(filename: &str, content: &[u8]) -> io::Result<()> {
    let mut file = File::create(filename)?;
    file.write_all(content)?;
    Ok(())
}
