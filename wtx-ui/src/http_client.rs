use crate::clap::HttpClient;
use std::{fs::OpenOptions, io::Write};
use wtx::{
  executor::TokioExecutor,
  http::{
    Header, HttpClient as _, KnownHeaderName, MsgBuffer, ReqBuilder,
    http2_client_pool::Http2ClientPoolBuilder,
  },
  misc::{AsciiGeneric, from_utf8_basic, str_split_once1, tracing_tree_init},
  rng::{ChaCha20, CryptoSeedableRng},
  tls::TlsConfig,
};

pub(crate) async fn http_client(http_client: HttpClient) {
  let HttpClient { data, header, method, output, uri, user_agent, verbose } = http_client;
  match verbose {
    0 => {}
    1 => tracing_tree_init(Some("info")).unwrap(),
    2 => tracing_tree_init(Some("debug")).unwrap(),
    _ => tracing_tree_init(Some("trace")).unwrap(),
  }
  let mut msg_buffer = MsgBuffer::from_uri(uri.into());
  for pair in header {
    let (name, values) = str_split_once1(&pair, AsciiGeneric::COLON).unwrap();
    msg_buffer
      .headers
      .push_from_iter(Header::from_name_and_value(
        name.trim(),
        values.split(',').map(|el| el.trim()),
      ))
      .unwrap();
  }
  if let Some(elem) = user_agent {
    msg_buffer
      .headers
      .push_from_iter(Header::from_name_and_value(
        KnownHeaderName::UserAgent.into(),
        [elem.as_str()],
      ))
      .unwrap();
  }
  if let Some(elem) = data {
    msg_buffer.body.extend_from_copyable_slice(elem.as_bytes()).unwrap();
  }
  let client = Http2ClientPoolBuilder::new(TokioExecutor, 1, TlsConfig::from_ccadb())
    .build(ChaCha20::from_std_random().unwrap());
  let res =
    client.send_req_recv_res(ReqBuilder::new(method, msg_buffer).into_request()).await.unwrap();
  if let Some(elem) = output {
    OpenOptions::new()
      .create(true)
      .truncate(true)
      .write(true)
      .open(elem)
      .unwrap()
      .write_all(&res.msg_data.body)
      .unwrap();
  } else {
    println!("{}", from_utf8_basic(&res.msg_data.body).unwrap());
  }
}
