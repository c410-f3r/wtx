//! Long lived secret

extern crate wtx;

use crate::wtx::rng::CryptoSeedableRng;
use std::{env, sync::OnceLock};
use wtx::{
  collection::Vector,
  misc::{Secret, SecretContext},
  rng::ChaCha20,
};

static SECRET: OnceLock<Secret> = OnceLock::new();

fn main() -> wtx::Result<()> {
  let data = env::args().nth(1).ok_or_else(|| wtx::Error::GenericStatic("No data".into()))?;
  let mut rng = ChaCha20::from_getrandom()?;
  let secret_context = SecretContext::new(&mut rng)?;
  let secret = Secret::new(data.into_bytes().as_mut(), &mut rng, secret_context)?;
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
