use crate::utils::spawn_thread;
use std::{
  error::Error,
  io::{BufReader, Error as IoError, ErrorKind as IoErrorKind, Read, Result as IoResult, Write},
  process::{Command, Stdio},
  sync::mpsc::{channel, SendError},
};

pub fn spawn_ffmpeg_process(width: usize, height: usize) -> IoResult<(impl Read, impl Write)> {
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

  let stdout = child
    .stdout
    .ok_or_else(|| IoError::new(IoErrorKind::Other, "[ffmpeg] stdout not captured!"))?;

  let stdin = child
    .stdin
    .ok_or_else(|| IoError::new(IoErrorKind::Other, "[ffmpeg] stdin not captured!"))?;

  Ok((stdout, stdin))
}

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
  let (stdout, mut stdin) = spawn_ffmpeg_process(width, height)?;

  let (tx, rx) = channel();

  let read_t_join = spawn_thread(move || -> Result<(), SendError<Vec<u8>>> {
    let mut reader = BufReader::new(stdout);
    let mut buf = vec![0u8; height * width * 3];
    while let Ok(()) = reader.read_exact(&mut buf) {
      tx.send(buf.clone())?;
    }
    Ok(())
  });

  let write_t_join = spawn_thread(move || -> IoResult<()> {
    let mut reader = BufReader::new(file);
    let mut buf = vec![0u8; read_buffer_size];
    loop {
      match reader.read(&mut buf) {
        Ok(nread) if nread > 0 => {
          stdin.write_all(&buf)?;
        }
        _ => break,
      }
    }
    Ok(())
  });
  while let Ok(buf) = rx.recv() {
    callback(&buf)?;
  }

  write_t_join(
    "[writer] something went wrong while waiting for the output reader thread to join!",
  )?;
  read_t_join("[reader] something went wrong while waiting for the output reader thread to join!")?;
  Ok(())
}
