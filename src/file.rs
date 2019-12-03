use std::fs::File;
use std::io::Read;

pub const BUFFER_SIZE: usize = 2048;

pub struct FileIterator {
  buf_reader: std::io::BufReader<File>,
  buffer: Box<[u8; BUFFER_SIZE]>,
}

impl FileIterator {
  pub fn new(file_name: &str) -> Result<Self, std::io::Error> {
    let file = File::open(file_name)?;
    let buf_reader = std::io::BufReader::new(file);
    let buffer = Box::from([0u8; BUFFER_SIZE]);
    Ok(FileIterator { buf_reader, buffer })
  }
}

impl Iterator for FileIterator {
  type Item = Box<[u8]>;
  fn next(&mut self) -> Option<Self::Item> {
    match self.buf_reader.read(&mut *self.buffer) {
      Ok(nread) if nread > 0 => Some(self.buffer.clone()),
      _ => None,
    }
  }
}
