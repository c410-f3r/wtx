//! Long lived secret

extern crate wtx;

use std::{env, sync::OnceLock};
use wtx::secret::{Secret, SecretStr};

static SECRET: OnceLock<SecretStr> = OnceLock::new();

fn main() -> wtx::Result<()> {
  let mut data = env::args()
    .nth(1)
    .ok_or_else(|| wtx::Error::GenericStatic("No data".try_into().unwrap_or_default()))?;
  let secret = Secret::new(data.as_mut_str())?;
  let _rslt = SECRET.set(secret);
  std::thread::spawn(|| {
    let _bytes = SECRET.wait().peek()?;
    // Make API requests, decrypt AES, sign documents, do a flip, etc...
    wtx::Result::Ok(())
  })
  .join()??;
  Ok(())
}
