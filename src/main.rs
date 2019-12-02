extern crate image;

use std::fs::File;
use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::sync::mpsc::channel;
use std::thread;

const IMAGE_LEN: usize = 112;
const BUFFER_SIZE: usize = 2048;

struct FileIterator {
    buf_reader: std::io::BufReader<File>,
    buffer: Box<[u8; BUFFER_SIZE]>,
}

impl FileIterator {
    fn new(file_name: &str) -> Result<Self, std::io::Error> {
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

fn ffmpeg_extract_frames<F>(
    file_reader: FileIterator,
    callback: F,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: Fn([u8; IMAGE_LEN * IMAGE_LEN * 3]) -> Result<(), Box<dyn std::error::Error>>,
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
        let stdout = child.stdout.ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::Other, "[ffmpeg] stdout not captured!")
        })?;
        thread::spawn(move || {
            let mut reader = std::io::BufReader::new(stdout);
            let mut buf = [0u8; IMAGE_LEN * IMAGE_LEN * 3];
            while let Ok(()) = reader.read_exact(&mut buf) {
                tx.send(buf).expect("Send buf over channel failed");
            }
        })
    };
    let write_t_handle = {
        let mut stdin = child.stdin.ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::Other, "[ffmpeg] stdin not captured!")
        })?;
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

fn frame_to_file(
    extract_path: &str,
    frame: [u8; IMAGE_LEN * IMAGE_LEN * 3],
    number: usize,
) -> Result<(), std::io::Error> {
    image::save_buffer(
        format!("{}/frame_{:0>4}.png", extract_path, number),
        &frame,
        IMAGE_LEN as u32,
        IMAGE_LEN as u32,
        image::RGB(8),
    )?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<_>>();
    match args.as_slice() {
        [_, file_path, extract_path] => {
            let file_iterator = FileIterator::new(&file_path)?;
            let count = std::rc::Rc::new(std::cell::RefCell::new(0usize));
            ffmpeg_extract_frames(file_iterator, |x| {
                let mut count = count.borrow_mut();
                *count += 1;
                frame_to_file(extract_path, x, *count)?;
                Ok(())
            })?;
        }
        _ => eprintln!("{} <file_path> <extract_path>", args[0]),
    }

    Ok(())
}
