use image_lib::save_buffer;

use crate::IMAGE_LEN;

pub fn image_buffer_to_file(
  extract_path: &str,
  buffer: [u8; IMAGE_LEN * IMAGE_LEN * 3],
  number: usize,
) -> Result<(), std::io::Error> {
  save_buffer(
    format!("{}/image_{:0>4}.png", extract_path, number),
    &buffer,
    IMAGE_LEN as u32,
    IMAGE_LEN as u32,
    image::RGB(8),
  )?;

  Ok(())
}
