use crate::clap::HttpClient;
use std::{fs::OpenOptions, io::Write};
use wtx::{
  http::{Header, HttpClient as _, KnownHeaderName, ReqResBuffer, client_pool::ClientPoolBuilder},
  misc::{from_utf8_basic, str_split_once1, tracing_tree_init},
};

pub(crate) async fn http_client(http_client: HttpClient) {
  let HttpClient { data, header, method, output, uri, user_agent, verbose } = http_client;
  match verbose {
    0 => {}
    1 => tracing_tree_init(Some("info")).unwrap(),
    2 => tracing_tree_init(Some("debug")).unwrap(),
    _ => tracing_tree_init(Some("trace")).unwrap(),
  }
  let mut rrb = ReqResBuffer::empty();
  for pair in header {
    let (name, values) = str_split_once1(&pair, b':').unwrap();
    rrb
      .headers
      .push_from_iter(Header::from_name_and_value(
        name.trim(),
        values.split(',').map(|el| el.trim()),
      ))
      .unwrap();
  }
  if let Some(elem) = user_agent {
    rrb
      .headers
      .push_from_iter(Header::from_name_and_value(
        KnownHeaderName::UserAgent.into(),
        [elem.as_str()],
      ))
      .unwrap();
  }
  if let Some(elem) = data {
    rrb.body.extend_from_copyable_slice(elem.as_bytes()).unwrap();
  }
  let mut client = ClientPoolBuilder::tokio_rustls(1).build();
  let res = client.send_recv_single(method, rrb, &uri.as_str().into()).await.unwrap();
  if let Some(elem) = output {
    OpenOptions::new()
      .create(true)
      .truncate(true)
      .write(true)
      .open(elem)
      .unwrap()
      .write_all(&res.rrd.body)
      .unwrap();
  } else {
    println!("{}", from_utf8_basic(&res.rrd.body).unwrap());
  }
}
