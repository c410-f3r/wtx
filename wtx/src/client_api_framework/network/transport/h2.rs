use crate::{
  client_api_framework::{
    misc::{manage_after_sending_related, manage_before_sending_related},
    network::{
      transport::{Transport, TransportParams},
      HttpParams, TransportGroup,
    },
    pkg::{Package, PkgsAux},
    Api,
  },
  misc::{from_utf8_basic_rslt, AsyncBounds},
};
use bytes::Bytes;
use core::ops::Range;
use h2::client::SendRequest;
use http::{
  header::{CONTENT_TYPE, USER_AGENT},
  request::Builder,
  HeaderValue, Request,
};
use tokio::io::{AsyncRead, AsyncWrite};

/// Hyper
#[derive(Debug)]
pub struct H2 {
  sender: SendRequest<Bytes>,
}

impl H2 {
  /// Performas a handshake and then returns an instance.
  pub async fn new<T>(io: T) -> crate::Result<Self>
  where
    T: AsyncRead + AsyncWrite + Send + Unpin + 'static,
  {
    let (sender, conn) = h2::client::Builder::new().handshake(io).await.unwrap();
    let _jh = tokio::task::spawn(async move {
      if let Err(err) = conn.await {
        eprintln!("{err}");
      }
    });
    Ok(Self { sender })
  }
}

impl<DRSR> Transport<DRSR> for H2
where
  DRSR: AsyncBounds,
{
  const GROUP: TransportGroup = TransportGroup::HTTP;
  type Params = HttpParams;

  #[inline]
  async fn send<A, P>(
    &mut self,
    pkg: &mut P,
    pkgs_aux: &mut PkgsAux<A, DRSR, HttpParams>,
  ) -> Result<(), A::Error>
  where
    A: Api,
    P: AsyncBounds + Package<A, DRSR, HttpParams>,
  {
    response(self, pkg, pkgs_aux).await?;
    Ok(())
  }

  #[inline]
  async fn send_and_retrieve<A, P>(
    &mut self,
    pkg: &mut P,
    pkgs_aux: &mut PkgsAux<A, DRSR, HttpParams>,
  ) -> Result<Range<usize>, A::Error>
  where
    A: Api,
    P: AsyncBounds + Package<A, DRSR, HttpParams>,
  {
    response(self, pkg, pkgs_aux).await?;
    Ok(0..pkgs_aux.byte_buffer.len())
  }
}

async fn response<A, DRSR, P>(
  h2: &mut H2,
  pkg: &mut P,
  pkgs_aux: &mut PkgsAux<A, DRSR, HttpParams>,
) -> Result<(), A::Error>
where
  A: Api,
  DRSR: AsyncBounds,
  P: Package<A, DRSR, HttpParams>,
{
  fn manage_data<A, DRSR>(mut rb: Builder, pkgs_aux: &mut PkgsAux<A, DRSR, HttpParams>) -> Builder {
    if let Some(data_format) = &pkgs_aux.tp.ext_req_params().mime {
      rb = rb.header(CONTENT_TYPE, HeaderValue::from_static(data_format._as_str()));
    }
    rb
  }

  let mut rb = Request::builder().uri(pkgs_aux.tp.ext_req_params().uri.uri());
  pkgs_aux.byte_buffer.clear();
  manage_before_sending_related(pkg, pkgs_aux, &mut *h2).await?;
  rb = match pkgs_aux.tp.ext_req_params().method {
    crate::http::Method::Connect => rb.method(http::Method::CONNECT),
    crate::http::Method::Delete => rb.method(http::Method::DELETE),
    crate::http::Method::Get => rb.method(http::Method::GET),
    crate::http::Method::Head => rb.method(http::Method::HEAD),
    crate::http::Method::Options => rb.method(http::Method::OPTIONS),
    crate::http::Method::Patch => manage_data(rb.method(http::Method::PATCH), pkgs_aux),
    crate::http::Method::Post => manage_data(rb.method(http::Method::POST), pkgs_aux),
    crate::http::Method::Put => manage_data(rb.method(http::Method::PUT), pkgs_aux),
    crate::http::Method::Trace => rb.method(http::Method::TRACE),
  };
  for (key, value) in pkgs_aux.tp.ext_req_params().headers.iter() {
    rb = rb.header(key, value);
  }
  if let Some(elem) = pkgs_aux.tp.ext_req_params().user_agent {
    rb = rb.header(USER_AGENT, HeaderValue::from_static(elem._as_str()));
  }
  let (res_fut, mut stream) = h2.sender.send_request(rb.body(()).unwrap(), true).unwrap();
  stream.send_data(pkgs_aux.byte_buffer.clone().into(), true).unwrap();
  let mut res = res_fut.await.unwrap();
  pkgs_aux.byte_buffer.clear();
  let body = res.body_mut();
  while let Some(chunk) = body.data().await {
    pkgs_aux.byte_buffer.extend(chunk.unwrap());
  }
  for (key, value) in res.headers() {
    pkgs_aux
      .tp
      .ext_res_params_mut()
      .headers
      .push_str(key.as_str(), from_utf8_basic_rslt(value.as_bytes()).map_err(Into::into)?)?;
  }
  pkgs_aux.tp.ext_res_params_mut().status_code = <_>::try_from(Into::<u16>::into(res.status()))?;
  manage_after_sending_related(pkg, pkgs_aux).await?;
  pkgs_aux.tp.reset();
  Ok(())
}
