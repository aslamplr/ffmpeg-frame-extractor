use std::{
  error::Error,
  io::{Error as IoError, ErrorKind as IoErrorKind},
  thread,
};

pub fn spawn_thread<F, T>(f: F) -> impl FnOnce(&str) -> Result<(), Box<dyn Error>>
where
  F: FnOnce() -> Result<(), T>,
  F: Send + 'static,
  T: Error + Send + 'static,
{
  let handle = thread::spawn(f);
  move |error_msg: &str| -> Result<(), Box<dyn Error>> {
    match handle.join() {
      Ok(resp) => {
        if let Err(err) = resp {
          return Err(Box::new(err));
        }
        Ok(())
      }
      Err(_) => Err(Box::new(IoError::new(IoErrorKind::Other, error_msg))),
    }
  }
}
