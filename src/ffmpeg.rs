use crate::{file::FileIterator, IMAGE_LEN};
use std::error::Error;
use std::io::{BufReader, Error as IoError, ErrorKind as IoErrorKind, Read, Write};
use std::process::{Command, Stdio};
use std::sync::mpsc::channel;
use std::thread;

pub fn ffmpeg_extract_frames<F>(
  file_reader: FileIterator,
  callback: F,
) -> Result<(), Box<dyn Error>>
where
  F: Fn([u8; IMAGE_LEN * IMAGE_LEN * 3]) -> Result<(), Box<dyn Error>>,
{
  let ffmpeg = "ffmpeg";
  let args = &[
    "-hide_banner",
    "-nostats",
    "-f",
    "webm",
    "-i",
    "pipe:0",
    "-filter_complex",
    "[0]fps=60[s0]",
    "-map",
    "[s0]",
    "-f",
    "rawvideo",
    "-pix_fmt",
    "rgb24",
    "-s",
    &format!("{0}x{0}", IMAGE_LEN),
    "pipe:1",
  ];
  let child = Command::new(ffmpeg)
    .args(args)
    .stdin(Stdio::piped())
    .stderr(Stdio::piped())
    .stdout(Stdio::piped())
    .spawn()?;

  let (tx, rx) = channel();

  let read_t_handle = {
    let stdout = child
      .stdout
      .ok_or_else(|| IoError::new(IoErrorKind::Other, "[ffmpeg] stdout not captured!"))?;
    thread::spawn(move || {
      let mut reader = BufReader::new(stdout);
      let mut buf = [0u8; IMAGE_LEN * IMAGE_LEN * 3];
      while let Ok(()) = reader.read_exact(&mut buf) {
        tx.send(buf).expect("Send buf over channel failed");
      }
    })
  };
  let write_t_handle = {
    let mut stdin = child
      .stdin
      .ok_or_else(|| IoError::new(IoErrorKind::Other, "[ffmpeg] stdin not captured!"))?;
    thread::spawn(move || {
      for buf in file_reader {
        stdin.write_all(&buf).expect("Unable to write to stdin");
      }
    })
  };
  while let Ok(buf) = rx.recv() {
    callback(buf)?;
  }

  write_t_handle
    .join()
    .expect("Something went wrong while waiting for the input writer thread to join!");

  read_t_handle
    .join()
    .expect("Something went wrong while waiting for the output reader thread to join!");

  Ok(())
}
