use std::{
  error::Error,
  io::{BufReader, Error as IoError, ErrorKind as IoErrorKind, Read, Write},
  process::{Command, Stdio},
  sync::mpsc::channel,
  thread,
};

pub fn ffmpeg_extract_frames<F, R>(
  file: R,
  read_buffer_size: usize,
  height: usize,
  width: usize,
  callback: F,
) -> Result<(), Box<dyn Error>>
where
  F: Fn(&[u8]) -> Result<(), Box<dyn Error>>,
  R: Read + Send + 'static,
{
  let ffmpeg = "ffmpeg";
  let args = &[
    "-hide_banner",
    "-nostats",
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
    &format!("{0}x{1}", width, height),
    "pipe:1",
  ];
  let child = Command::new(ffmpeg)
    .args(args)
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .spawn()?;

  let (tx, rx) = channel();

  let read_t_handle = {
    let stdout = child
      .stdout
      .ok_or_else(|| IoError::new(IoErrorKind::Other, "[ffmpeg] stdout not captured!"))?;
    thread::spawn(move || {
      let mut reader = BufReader::new(stdout);
      let mut buf = vec![0u8; height * width * 3];
      while let Ok(()) = reader.read_exact(&mut buf) {
        tx.send(buf.clone()).expect("Send buf over channel failed");
      }
    })
  };
  let write_t_handle = {
    let mut stdin = child
      .stdin
      .ok_or_else(|| IoError::new(IoErrorKind::Other, "[ffmpeg] stdin not captured!"))?;
    thread::spawn(move || {
      let mut reader = BufReader::new(file);
      let mut buf = vec![0u8; read_buffer_size];
      loop {
        match reader.read(&mut buf) {
          Ok(nread) if nread > 0 => {
            stdin.write_all(&buf).expect("Unable to write to stdin");
          }
          _ => break,
        }
      }
    })
  };
  while let Ok(buf) = rx.recv() {
    callback(&buf)?;
  }

  write_t_handle
    .join()
    .expect("Something went wrong while waiting for the input writer thread to join!");

  read_t_handle
    .join()
    .expect("Something went wrong while waiting for the output reader thread to join!");

  Ok(())
}
