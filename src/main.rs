extern crate ffmpeg_read;

use ffmpeg_read::{ffmpeg::ffmpeg_extract_frames, file::FileIterator, image::image_buffer_to_file};
use std::{cell::RefCell, env::args, error::Error, rc::Rc};

const READ_BUFFER_SIZE: usize = 2048;
const IMAGE_HEIGHT: u32 = 120;
const IMAGE_WIDTH: u32 = 160;

fn main() -> Result<(), Box<dyn Error>> {
    let args = args().collect::<Vec<_>>();
    match &args[..] {
        [_, file_path, extract_path, image_format] => {
            let file_iterator = FileIterator::new(file_path, READ_BUFFER_SIZE)?;
            let count = Rc::new(RefCell::new(0usize));
            ffmpeg_extract_frames(
                file_iterator,
                IMAGE_HEIGHT as usize,
                IMAGE_WIDTH as usize,
                |frame| {
                    let mut count = count.borrow_mut();
                    *count += 1;
                    let image_path =
                        format!("{}/image_{:0>4}.{}", extract_path, count, image_format);
                    image_buffer_to_file(&image_path, frame, IMAGE_HEIGHT, IMAGE_WIDTH)?;
                    Ok(())
                },
            )?;
        }
        _ => eprintln!(
            "{} <file_path> <extract_path> <image_file_format> {{png|jpg}}",
            args[0]
        ),
    }

    Ok(())
}
