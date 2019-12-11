extern crate ffmpeg_read;
extern crate async_std;

use ffmpeg_read::{ffmpeg::ffmpeg_extract_frames, image::image_buffer_to_file};
use std::{env::args, error::Error, sync::{Arc, Mutex}};
use async_std::{fs::File as FileAsync, task};

const READ_BUFFER_SIZE: usize = 2048;
const IMAGE_HEIGHT: u32 = 120;
const IMAGE_WIDTH: u32 = 160;

fn main() -> Result<(), Box<dyn Error>> {
    let args = args().collect::<Vec<_>>();
    match &args[..] {
        [_, file_path, extract_path, image_format] => {
            let extract_task = async {
                let file = FileAsync::open(file_path);
                let count = Arc::new(Mutex::new(0usize));
                let extract_path = extract_path.clone();
                let image_format = image_format.clone();
                ffmpeg_extract_frames(
                    file.await.expect("Unable to open file!"),
                    READ_BUFFER_SIZE,
                    IMAGE_HEIGHT as usize,
                    IMAGE_WIDTH as usize,
                    move |frame| {
                        let mut count = count.lock().unwrap();
                        *count += 1;
                        let image_path =
                            format!("{}/image_{:0>4}.{}", extract_path, *count, image_format);
                        image_buffer_to_file(&image_path, frame, IMAGE_HEIGHT, IMAGE_WIDTH)?;
                        Ok(())
                    },
                ).await.expect("Unable to run ffmpeg_extract_frames due to error!");
            };
            task::block_on(extract_task);
        }
        _ => eprintln!(
            "{} <file_path> <extract_path> <image_file_format> {{png|jpg}}",
            args[0]
        ),
    }

    Ok(())
}
