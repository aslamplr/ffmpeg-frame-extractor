extern crate ffmpeg_read;
use ffmpeg_read::{ffmpeg::ffmpeg_extract_frames, file::FileIterator, image::image_buffer_to_file};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<_>>();
    match &args[..] {
        [_, file_path, extract_path] => {
            let file_iterator = FileIterator::new(&file_path)?;
            let count = std::rc::Rc::new(std::cell::RefCell::new(0usize));
            ffmpeg_extract_frames(file_iterator, |x| {
                let mut count = count.borrow_mut();
                *count += 1;
                image_buffer_to_file(extract_path, x, *count)?;
                Ok(())
            })?;
        }
        _ => eprintln!("{} <file_path> <extract_path>", args[0]),
    }

    Ok(())
}
