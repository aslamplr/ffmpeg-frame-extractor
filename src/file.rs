use std::{
  fs::File,
  io::{BufReader, Error, Read},
};

fn new_slice_with_capacity<T: Copy>(capacity: usize) -> Box<[T]> {
  let mut vector = Vec::with_capacity(capacity);
  unsafe { vector.set_len(capacity) };
  vector.into_boxed_slice()
}

pub struct FileIterator {
  buf_reader: BufReader<File>,
  buffer: Box<[u8]>,
}

impl FileIterator {
  pub fn new(file_name: &str, buffer_size: usize) -> Result<Self, Error> {
    let file = File::open(file_name)?;
    let buf_reader = BufReader::new(file);
    let buffer = new_slice_with_capacity(buffer_size);
    Ok(FileIterator { buf_reader, buffer })
  }
}

impl Iterator for FileIterator {
  type Item = Box<[u8]>;
  fn next(&mut self) -> Option<Self::Item> {
    match self.buf_reader.read(&mut self.buffer) {
      Ok(nread) if nread > 0 => Some(self.buffer.clone()),
      _ => None,
    }
  }
}
