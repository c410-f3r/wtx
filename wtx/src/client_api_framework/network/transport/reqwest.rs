use crate::{
  client_api_framework::{
    misc::{manage_after_sending_related, manage_before_sending_related},
    network::{
      http::HttpMethod,
      transport::{Transport, TransportParams},
      HttpParams, TransportGroup,
    },
    pkg::{Package, PkgsAux},
    Api,
  },
  misc::{from_utf8_basic_rslt, AsyncBounds},
};
use core::ops::Range;
use reqwest::{
  header::{HeaderValue, CONTENT_TYPE, USER_AGENT},
  Client, RequestBuilder,
};

/// ```rust,no_run
/// # async fn fun() -> wtx::Result<()> {
/// use wtx::client_api_framework::{
///   network::{transport::Transport, HttpParams},
///   pkg::PkgsAux,
/// };
/// let _ = reqwest::Client::new()
///   .send_retrieve_and_decode_contained(
///     &mut (),
///     &mut PkgsAux::from_minimum((), (), HttpParams::from_uri("URI")?.into()),
///   )
///   .await?;
/// # Ok(()) }
/// ```
impl<DRSR> Transport<DRSR> for Client
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
    let _res = response(self, pkg, pkgs_aux).await?;
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
    let res = response(self, pkg, pkgs_aux).await?;
    let received_bytes = res.bytes().await.map_err(Into::into)?;
    pkgs_aux.byte_buffer.extend(received_bytes.into_iter());
    Ok(0..pkgs_aux.byte_buffer.len())
  }
}

async fn response<A, DRSR, P>(
  client: &mut Client,
  pkg: &mut P,
  pkgs_aux: &mut PkgsAux<A, DRSR, HttpParams>,
) -> Result<reqwest::Response, A::Error>
where
  A: Api,
  DRSR: AsyncBounds,
  P: Package<A, DRSR, HttpParams>,
{
  fn manage_data<A, DRSR>(
    mut rb: RequestBuilder,
    pkgs_aux: &mut PkgsAux<A, DRSR, HttpParams>,
  ) -> RequestBuilder {
    if let Some(data_format) = &pkgs_aux.tp.ext_req_params().mime_type {
      rb = rb.header(CONTENT_TYPE, HeaderValue::from_static(data_format._as_str()));
    }
    rb = rb.body(pkgs_aux.byte_buffer.clone());
    rb
  }
  pkgs_aux.byte_buffer.clear();
  manage_before_sending_related(pkg, pkgs_aux, &mut *client).await?;
  let mut rb = match pkgs_aux.tp.ext_req_params().method {
    HttpMethod::Delete => client.delete(pkgs_aux.tp.ext_req_params().uri.uri()),
    HttpMethod::Get => client.get(pkgs_aux.tp.ext_req_params().uri.uri()),
    HttpMethod::Patch => {
      manage_data(client.patch(pkgs_aux.tp.ext_req_params().uri.uri()), pkgs_aux)
    }
    HttpMethod::Post => manage_data(client.post(pkgs_aux.tp.ext_req_params().uri.uri()), pkgs_aux),
    HttpMethod::Put => manage_data(client.put(pkgs_aux.tp.ext_req_params().uri.uri()), pkgs_aux),
  };
  pkgs_aux.byte_buffer.clear();
  for (key, value) in pkgs_aux.tp.ext_req_params().headers.iter() {
    rb = rb.header(key, value);
  }
  if let Some(elem) = pkgs_aux.tp.ext_req_params().user_agent {
    rb = rb.header(USER_AGENT, HeaderValue::from_static(elem._as_str()));
  }
  let res = rb.send().await.map_err(Into::into)?;
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
  Ok(res)
}
