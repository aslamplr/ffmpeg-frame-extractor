use async_std::{
  io::{BufReader as BufReaderAsync, Read as ReadAsync},
  prelude::*,
  sync::Arc,
  task,
};
use std::{
  error::Error,
  io::{BufReader, Error as IoError, ErrorKind as IoErrorKind, Read, Result as IoResult, Write},
  process::{Command, Stdio},
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

pub async fn ffmpeg_extract_frames<F, R>(
  file: R,
  read_buffer_size: usize,
  height: usize,
  width: usize,
  callback: F,
) -> Result<(), Box<dyn Error>>
where
  F: Fn(&[u8]) -> Result<(), Box<dyn Error>> + Send + Sync + 'static,
  R: ReadAsync + Unpin + Send + 'static,
{
  let callback = Arc::new(callback);
  let (stdout, mut stdin) = spawn_ffmpeg_process(width, height)
    .expect("Something went wrong while spawning ffmpeg process!");

  let read_task = task::spawn(async move {
    let mut reader = BufReader::new(stdout);
    let mut buf = vec![0u8; height * width * 3];
    while let Ok(()) = reader.read_exact(&mut buf) {
      callback(&buf).expect("An error occured while calling callback!");
    }
  });

  let write_task = task::spawn(async move {
    let mut reader = BufReaderAsync::new(file);
    let mut buf = vec![0u8; read_buffer_size];
    loop {
      let read = reader.read(&mut buf);
      match read.await {
        Ok(nread) if nread > 0 => {
          stdin
            .write_all(&buf)
            .expect("Unable to write to pipe stdin");
        }
        _ => break,
      }
    }
  });

  write_task.await;
  read_task.await;
  Ok(())
}
