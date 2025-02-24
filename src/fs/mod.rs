use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read};
use std::path::Path;

pub fn read_file_to_string<P: AsRef<Path>>(path: P) -> io::Result<String> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut contents = String::new();
    reader.read_to_string(&mut contents)?;
    Ok(contents)
}

pub fn create_write_buffer<P: AsRef<Path>>(path: P) -> io::Result<BufWriter<File>> {
    let file = File::create(path)?;
    Ok(BufWriter::new(file))
}
