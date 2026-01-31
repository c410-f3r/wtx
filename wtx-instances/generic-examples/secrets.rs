//! Long lived secret

extern crate wtx;

use crate::wtx::rng::SeedableRng;
use std::{env, sync::OnceLock};
use wtx::{
  collection::Vector,
  misc::{Secret, SensitiveBytes},
  rng::ChaCha20,
};

static SECRET: OnceLock<Secret> = OnceLock::new();

fn main() -> wtx::Result<()> {
  let data = env::args().nth(1).ok_or(wtx::Error::Generic(Box::new("No data".into())))?;
  let mut rng = ChaCha20::from_std_random()?;
  let secret = Secret::new(SensitiveBytes::new_locked(data.into_bytes().as_mut())?, &mut rng)?;
  let _rslt = SECRET.set(secret);
  std::thread::spawn(|| {
    let mut buffer = Vector::new();
    SECRET.wait().peek(&mut buffer, |_data| {
      // Sign documents, pass API keys, etc...
    })?;
    wtx::Result::Ok(())
  })
  .join()??;
  Ok(())
}
