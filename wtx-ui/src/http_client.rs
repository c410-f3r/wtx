use crate::clap::HttpClient;
use std::{fs::OpenOptions, io::Write as _};
use wtx::{
  collections::Vector,
  http::{
    Header, HttpClient as _, KnownHeaderName, MsgBuffer, ReqBuilder,
    http2_client_pool::Http2ClientPoolBuilder,
  },
  misc::{AsciiGeneric, from_utf8_basic, into_rslt, str_split_once1, tracing_tree_init},
  tls::{TlsConfig, TlsModeVerified},
};

pub(crate) async fn http_client(http_client: HttpClient) -> wtx::Result<()> {
  let HttpClient { data, header, method, output, uri, user_agent, verbose } = http_client;
  match verbose {
    0 => {}
    1 => tracing_tree_init(Some("info"))?,
    2 => tracing_tree_init(Some("debug"))?,
    _ => tracing_tree_init(Some("trace"))?,
  }
  let mut msg_buffer = MsgBuffer::from_uri(uri.into());
  for pair in header {
    let (name, values) = into_rslt(str_split_once1(&pair, AsciiGeneric::COLON))?;
    msg_buffer
      .headers
      .push_from_iter(Header::from_name_and_value(name.trim(), values.split(',').map(str::trim)))?;
  }
  if let Some(elem) = user_agent {
    msg_buffer.headers.push_from_iter(Header::from_name_and_value(
      KnownHeaderName::UserAgent.into(),
      [elem.as_str()],
    ))?;
  }
  if let Some(elem) = data {
    msg_buffer.body.extend_from_copyable_slice(elem.as_bytes())?;
  }
  let tls_config = TlsConfig::from_ccadb(TlsModeVerified::default())?;
  let client = Http2ClientPoolBuilder::tokio(1, tls_config)?.build();
  let res = client
    .send_req_recv_res(&mut Vector::new(), ReqBuilder::new(method, msg_buffer).into_request())
    .await?;
  if let Some(elem) = output {
    OpenOptions::new()
      .create(true)
      .truncate(true)
      .write(true)
      .open(elem)?
      .write_all(&res.msg_data.body)?;
  } else {
    println!("{}", from_utf8_basic(&res.msg_data.body)?);
  }
  Ok(())
}
