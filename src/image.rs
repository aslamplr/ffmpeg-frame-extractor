use image_lib::save_buffer;

pub fn image_buffer_to_file(
  file_path: &str,
  buffer: &[u8],
  height: u32,
  width: u32,
) -> Result<(), std::io::Error> {
  save_buffer(file_path, &buffer, width, height, image::RGB(8))?;

  Ok(())
}
