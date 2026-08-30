//! Browsers usually send `form-data` content when dealing with files.

use wtx::{
  executor::TokioExecutor,
  http::http2_server_framework::{
    CorsMiddleware, FormData, FormDataIter, Http2ServerFramework, HttpRouter, State, post,
  },
  rng::{ChaCha20, CryptoSeedableRng as _},
  tls::TlsConfig,
};
use wtx_examples::{PUBLIC_KEY, SECRET_KEY, host_from_args};

fn main() -> wtx::Result<()> {
  let mut rng = ChaCha20::from_std_random()?;
  let tls_config = TlsConfig::from_keys_pem(PUBLIC_KEY.try_into()?, &mut rng, SECRET_KEY)?;
  let router =
    HttpRouter::new(wtx::paths!(("/form_data", post(form_data))), CorsMiddleware::permissive())?;
  Http2ServerFramework::new(TokioExecutor::default(), rng, tls_config)?
    .set_error_cb(|err| eprintln!("Error: {err}"))
    .run_in_threads(&host_from_args(), router)
}

async fn form_data(
  State { req, .. }: State<'_, ()>,
  FormData(delimiter): FormData,
) -> wtx::Result<()> {
  for block_rslt in FormDataIter::new(&req.msg_data.body, &delimiter)? {
    println!("{:?}", block_rslt?);
  }
  Ok(())
}
