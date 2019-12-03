extern crate ffmpeg_read;

use ffmpeg_read::{ffmpeg::ffmpeg_extract_frames, file::FileIterator};
use std::{cell::RefCell, env::args, error::Error, rc::Rc};

const READ_BUFFER_SIZE: usize = 2048;
const IMAGE_HEIGHT: u32 = 120;
const IMAGE_WIDTH: u32 = 160;

fn main() -> Result<(), Box<dyn Error>> {
    let args = args().collect::<Vec<_>>();
    match &args[..] {
        [_, file_path] => {
            let file_iterator = FileIterator::new(file_path, READ_BUFFER_SIZE)?;
            let count = Rc::new(RefCell::new(0usize));
            ffmpeg_extract_frames(
                file_iterator,
                IMAGE_HEIGHT as usize,
                IMAGE_WIDTH as usize,
                |_frame| {
                    let mut count = count.borrow_mut();
                    *count += 1;
                    Ok(())
                },
            )?;
            println!("total_frames={}", count.borrow());
        }
        _ => eprintln!(
            "{} <file_path>",
            args[0]
        ),
    }

    Ok(())
}
