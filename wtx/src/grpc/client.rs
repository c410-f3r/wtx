use crate::http::{Header, KnownHeaderName, ReqResDataMut, Request};

const WTX_USER_AGENT: &str = concat!("wtx/", env!("CARGO_PKG_VERSION"));

pub struct Client {}

impl Client {
  pub fn send_req<RRD>(data: &[u8], mut req: Request<RRD>)
  where
    RRD: ReqResDataMut,
  {
    req.rrd.headers_mut();
  }

  fn append_grpc_headers<RRD>(req: &mut Request<RRD>) -> crate::Result<()>
  where
    RRD: ReqResDataMut,
  {
    let headers = [
      Header {
        is_sensitive: false,
        is_trailer: false,
        name: KnownHeaderName::ContentType.into(),
        value: b"application/grpc",
      },
      Header {
        is_sensitive: false,
        is_trailer: false,
        name: KnownHeaderName::Te.into(),
        value: b"trailers",
      },
      Header {
        is_sensitive: false,
        is_trailer: false,
        name: KnownHeaderName::UserAgent.into(),
        value: WTX_USER_AGENT.as_bytes(),
      },
    ];
    for header in headers {
      req.rrd.headers_mut().push_front(header, &[])?;
    }
    Ok(())
  }
}
