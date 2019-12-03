use std::fs::File;
use std::io::Read;

pub struct FileIterator {
  buf_reader: std::io::BufReader<File>,
  buffer: Vec<u8>,
}

impl FileIterator {
  pub fn new(file_name: &str, buffer_size: usize) -> Result<Self, std::io::Error> {
    let file = File::open(file_name)?;
    let buf_reader = std::io::BufReader::new(file);
    let buffer = vec![0u8; buffer_size];
    Ok(FileIterator { buf_reader, buffer })
  }
}

impl Iterator for FileIterator {
  type Item = Vec<u8>;
  fn next(&mut self) -> Option<Self::Item> {
    match self.buf_reader.read(&mut self.buffer) {
      Ok(nread) if nread > 0 => Some(self.buffer.clone()),
      _ => None,
    }
  }
}
