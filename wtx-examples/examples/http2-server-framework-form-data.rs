//! Browsers usually send `form-data` content when dealing with files.

use wtx::{
  http::http2_server_framework::{
    FormData, FormDataIter, Http2ServerFramework, HttpRouter, State, post,
  },
  tls::TlsConfig,
};
use wtx_examples::{PUBLIC_KEY, SECRET_KEY, host_from_args};

fn main() -> wtx::Result<()> {
  Http2ServerFramework::tokio(TlsConfig::from_keys_pem(PUBLIC_KEY, SECRET_KEY)?)?
    .set_error_cb(|err| eprintln!("Error: {err}"))
    .set_http_router(HttpRouter::from_paths(wtx::paths!(("/form_data", post(form_data))))?)
    .run_in_threads(&host_from_args())
}

async fn form_data(
  State { req, data: _ }: State<'_, ()>,
  FormData(delimiter): FormData,
) -> wtx::Result<()> {
  for block_rslt in FormDataIter::new(&req.msg_data.body, &delimiter)? {
    println!("{:?}", block_rslt?);
  }
  Ok(())
}
