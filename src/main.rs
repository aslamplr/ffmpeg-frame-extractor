extern crate image;

use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::mpsc::channel;
use std::thread;

const IMAGE_LEN: usize = 112;

fn read_from_file(file_name: &str) -> std::io::Result<Vec<u8>> {
    let mut file = File::open(file_name)?;

    let mut data = Vec::new();
    file.read_to_end(&mut data)?;

    Ok(data)
}

fn ffmpeg_extract_frames<F>(
    file_content: Vec<u8>,
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
            stdin
                .write_all(&file_content)
                .expect("Unable to write to stdin");
            drop(stdin);
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
    frame: [u8; IMAGE_LEN * IMAGE_LEN * 3],
    number: usize,
) -> Result<(), std::io::Error> {
    image::save_buffer(
        format!(
            "/Users/aslam/Downloads/ffmpeg_samples/extracted_frames/frame_{:0>4}.png",
            number
        ),
        &frame,
        IMAGE_LEN as u32,
        IMAGE_LEN as u32,
        image::RGB(8),
    )?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_content =
        read_from_file("/Users/aslam/Downloads/ffmpeg_samples/Schlossbergbahn.webm.480p.vp9.webm")?;
    let count = std::rc::Rc::new(std::cell::RefCell::new(0usize));
    ffmpeg_extract_frames(file_content, move |x| {
        let mut count = count.borrow_mut();
        *count += 1;
        frame_to_file(x, *count)?;
        Ok(())
    })?;

    Ok(())
}
