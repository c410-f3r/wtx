use crate::{
  client_api_framework::{
    Api,
    misc::{
      _log_res, manage_after_sending_bytes, manage_after_sending_pkg, manage_before_sending_bytes,
      manage_before_sending_pkg,
    },
    network::{
      HttpParams, HttpReqParams, HttpResParams, TransportGroup,
      transport::{RecievingTransport, SendingTransport, Transport, TransportParams},
    },
    pkg::{Package, PkgsAux},
  },
  http::{Header, HttpClient, KnownHeaderName, Method, ReqResBuffer, Response, WTX_USER_AGENT},
  http2::{Http2, Http2Buffer, Http2Data},
  misc::{LeaseMut, Lock, RefCounter, StreamWriter, UriRef},
};
use core::{mem, ops::Range};

impl<HD, SW, TP> RecievingTransport<TP> for Http2<HD, true>
where
  HD: RefCounter,
  HD::Item: Lock<Resource = Http2Data<Http2Buffer, SW, true>>,
  SW: StreamWriter,
{
  #[inline]
  async fn recv<A, DRSR>(
    &mut self,
    pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
  ) -> Result<Range<usize>, A::Error>
  where
    A: Api,
  {
    _log_res(&pkgs_aux.byte_buffer);
    Ok(0..pkgs_aux.byte_buffer.len())
  }
}
impl<HD, SW, TP> SendingTransport<TP> for Http2<HD, true>
where
  HD: RefCounter,
  HD::Item: Lock<Resource = Http2Data<Http2Buffer, SW, true>>,
  SW: StreamWriter,
  TP: LeaseMut<HttpParams>,
{
  #[inline]
  async fn send_bytes<A, DRSR>(
    &mut self,
    bytes: &[u8],
    pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
  ) -> Result<(), A::Error>
  where
    A: Api,
  {
    send_bytes(bytes, self, pkgs_aux).await
  }

  #[inline]
  async fn send_pkg<A, DRSR, P>(
    &mut self,
    pkg: &mut P,
    pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
  ) -> Result<(), A::Error>
  where
    A: Api,
    P: Package<A, DRSR, Self::Inner, TP>,
  {
    send_pkg(self, pkg, pkgs_aux).await
  }
}
impl<HD, SW, TP> Transport<TP> for Http2<HD, true>
where
  HD: RefCounter,
  HD::Item: Lock<Resource = Http2Data<Http2Buffer, SW, true>>,
  SW: StreamWriter,
{
  const GROUP: TransportGroup = TransportGroup::HTTP;
  type Inner = Self;
}

#[inline]
async fn send<A, AUX, DRSR, HD, SW, TP>(
  mut aux: &mut AUX,
  client: &mut Http2<HD, true>,
  pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
  before_sending: impl AsyncFnOnce(
    &mut AUX,
    &mut PkgsAux<A, DRSR, TP>,
    &mut Http2<HD, true>,
  ) -> Result<(), A::Error>,
  send: impl AsyncFnOnce(
    &mut AUX,
    Method,
    ReqResBuffer,
    &mut Http2<HD, true>,
    &UriRef<'_>,
  ) -> crate::Result<Response<ReqResBuffer>>,
  after_sending: impl AsyncFnOnce(
    &mut AUX,
    &mut PkgsAux<A, DRSR, TP>,
    &mut Http2<HD, true>,
  ) -> Result<(), A::Error>,
) -> Result<(), A::Error>
where
  A: Api,
  HD: RefCounter,
  HD::Item: Lock<Resource = Http2Data<Http2Buffer, SW, true>>,
  SW: StreamWriter,
  TP: LeaseMut<HttpParams>,
{
  before_sending(&mut aux, pkgs_aux, client).await?;
  let tp = pkgs_aux.tp.lease_mut();
  let (req_params, res_params) = tp.ext_params_mut();
  let HttpReqParams { headers, method, mime, uri } = req_params;
  let HttpResParams { status_code } = res_params;
  headers.push_from_iter(Header::from_name_and_value(
    KnownHeaderName::UserAgent.into(),
    [WTX_USER_AGENT.as_bytes()],
  ))?;
  if let Some(elem) = mime {
    headers.push_from_iter(Header::from_name_and_value(
      KnownHeaderName::ContentType.into(),
      [elem.as_str().as_bytes()],
    ))?;
  }
  let mut rrb = ReqResBuffer::empty();
  mem::swap(&mut rrb.body, &mut pkgs_aux.byte_buffer);
  mem::swap(&mut rrb.headers, headers);
  // Only servers use the URI buffer so there is no need for URI swaps
  let mut res = send(&mut aux, *method, rrb, client, &uri.to_ref()).await?;
  mem::swap(&mut res.rrd.body, &mut pkgs_aux.byte_buffer);
  mem::swap(&mut res.rrd.headers, headers);
  *status_code = res.status_code;
  after_sending(&mut aux, pkgs_aux, client).await?;
  Ok(())
}

#[inline]
async fn send_bytes<A, DRSR, HD, SW, TP>(
  mut bytes: &[u8],
  client: &mut Http2<HD, true>,
  pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
) -> Result<(), A::Error>
where
  A: Api,
  HD: RefCounter,
  HD::Item: Lock<Resource = Http2Data<Http2Buffer, SW, true>>,
  SW: StreamWriter,
  TP: LeaseMut<HttpParams>,
{
  send(
    &mut bytes,
    client,
    pkgs_aux,
    async move |aux, pa, tr| manage_before_sending_bytes(aux, pa, tr).await,
    async move |aux, method, mut rrb, tr, uri| {
      let stream = tr.send_req(method, (aux, &mut rrb.headers), uri).await?;
      tr.recv_res(rrb, stream).await
    },
    async move |_, pa, _| manage_after_sending_bytes(pa).await,
  )
  .await
}

#[inline]
async fn send_pkg<A, DRSR, HD, P, SW, TP>(
  client: &mut Http2<HD, true>,
  pkg: &mut P,
  pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
) -> Result<(), A::Error>
where
  A: Api,
  HD: RefCounter,
  HD::Item: Lock<Resource = Http2Data<Http2Buffer, SW, true>>,
  P: Package<A, DRSR, Http2<HD, true>, TP>,
  SW: StreamWriter,
  TP: LeaseMut<HttpParams>,
{
  send(
    pkg,
    client,
    pkgs_aux,
    async move |aux, pa, tr| manage_before_sending_pkg(aux, pa, tr).await,
    async move |_, method, rrb, tr, uri| tr.send_recv_single(method, rrb, uri).await,
    async move |aux, pa, tr| manage_after_sending_pkg(aux, pa, tr).await,
  )
  .await
}

#[cfg(feature = "http-client-pool")]
mod http_client_pool {
  use crate::{
    client_api_framework::{
      Api,
      misc::_log_res,
      network::{
        HttpParams, TransportGroup,
        transport::{
          RecievingTransport, SendingTransport, Transport, TransportParams,
          wtx_http::{send_bytes, send_pkg},
        },
      },
      pkg::{Package, PkgsAux},
    },
    http::client_pool::{ClientPool, ClientPoolResource},
    http2::{Http2, Http2Buffer, Http2Data},
    misc::{LeaseMut, Lock, RefCounter, StreamWriter},
    pool::{ResourceManager, SimplePoolResource},
  };
  use core::ops::Range;

  impl<AUX, HD, RL, RM, SW, TP> RecievingTransport<TP> for ClientPool<RL, RM>
  where
    HD: RefCounter,
    HD::Item: Lock<Resource = Http2Data<Http2Buffer, SW, true>>,
    RL: Lock<Resource = SimplePoolResource<RM::Resource>>,
    RM: ResourceManager<
        CreateAux = str,
        Error = crate::Error,
        RecycleAux = str,
        Resource = ClientPoolResource<AUX, Http2<HD, true>>,
      >,
    SW: StreamWriter,
    for<'any> RL: 'any,
    for<'any> RM: 'any,
  {
    #[inline]
    async fn recv<A, DRSR>(
      &mut self,
      pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
    ) -> Result<Range<usize>, A::Error>
    where
      A: Api,
    {
      _log_res(&pkgs_aux.byte_buffer);
      Ok(0..pkgs_aux.byte_buffer.len())
    }
  }
  impl<AUX, HD, RL, RM, SW, TP> SendingTransport<TP> for ClientPool<RL, RM>
  where
    HD: RefCounter,
    HD::Item: Lock<Resource = Http2Data<Http2Buffer, SW, true>>,
    RL: Lock<Resource = SimplePoolResource<RM::Resource>>,
    RM: ResourceManager<
        CreateAux = str,
        Error = crate::Error,
        RecycleAux = str,
        Resource = ClientPoolResource<AUX, Http2<HD, true>>,
      >,
    SW: StreamWriter,
    TP: LeaseMut<HttpParams>,
    for<'any> RL: 'any,
    for<'any> RM: 'any,
  {
    #[inline]
    async fn send_bytes<A, DRSR>(
      &mut self,
      bytes: &[u8],
      pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
    ) -> Result<(), A::Error>
    where
      A: Api,
    {
      send_bytes(
        bytes,
        &mut self.lock(&pkgs_aux.tp.lease_mut().ext_req_params_mut().uri.to_ref()).await?.client,
        pkgs_aux,
      )
      .await
    }

    #[inline]
    async fn send_pkg<A, DRSR, P>(
      &mut self,
      pkg: &mut P,
      pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
    ) -> Result<(), A::Error>
    where
      A: Api,
      P: Package<A, DRSR, Self::Inner, TP>,
    {
      send_pkg(
        &mut self.lock(&pkgs_aux.tp.lease_mut().ext_req_params_mut().uri.to_ref()).await?.client,
        pkg,
        pkgs_aux,
      )
      .await
    }
  }
  impl<AUX, HD, RL, RM, SW, TP> Transport<TP> for ClientPool<RL, RM>
  where
    HD: RefCounter,
    HD::Item: Lock<Resource = Http2Data<Http2Buffer, SW, true>>,
    RL: Lock<Resource = SimplePoolResource<RM::Resource>>,
    RM: ResourceManager<
        CreateAux = str,
        Error = crate::Error,
        RecycleAux = str,
        Resource = ClientPoolResource<AUX, Http2<HD, true>>,
      >,
    SW: StreamWriter,
  {
    const GROUP: TransportGroup = TransportGroup::HTTP;
    type Inner = Http2<HD, true>;
  }

  impl<AUX, HD, RL, RM, SW, TP> RecievingTransport<TP> for &ClientPool<RL, RM>
  where
    HD: RefCounter,
    HD::Item: Lock<Resource = Http2Data<Http2Buffer, SW, true>>,
    RL: Lock<Resource = SimplePoolResource<RM::Resource>>,
    RM: ResourceManager<
        CreateAux = str,
        Error = crate::Error,
        RecycleAux = str,
        Resource = ClientPoolResource<AUX, Http2<HD, true>>,
      >,
    SW: StreamWriter,
    for<'any> RL: 'any,
    for<'any> RM: 'any,
  {
    #[inline]
    async fn recv<A, DRSR>(
      &mut self,
      pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
    ) -> Result<Range<usize>, A::Error>
    where
      A: Api,
    {
      _log_res(&pkgs_aux.byte_buffer);
      Ok(0..pkgs_aux.byte_buffer.len())
    }
  }
  impl<AUX, HD, RL, RM, SW, TP> SendingTransport<TP> for &ClientPool<RL, RM>
  where
    HD: RefCounter,
    HD::Item: Lock<Resource = Http2Data<Http2Buffer, SW, true>>,
    RL: Lock<Resource = SimplePoolResource<RM::Resource>>,
    RM: ResourceManager<
        CreateAux = str,
        Error = crate::Error,
        RecycleAux = str,
        Resource = ClientPoolResource<AUX, Http2<HD, true>>,
      >,
    SW: StreamWriter,
    TP: LeaseMut<HttpParams>,
    for<'any> RL: 'any,
    for<'any> RM: 'any,
  {
    #[inline]
    async fn send_bytes<A, DRSR>(
      &mut self,
      bytes: &[u8],
      pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
    ) -> Result<(), A::Error>
    where
      A: Api,
    {
      send_bytes(
        bytes,
        &mut self.lock(&pkgs_aux.tp.lease_mut().ext_req_params_mut().uri.to_ref()).await?.client,
        pkgs_aux,
      )
      .await
    }

    #[inline]
    async fn send_pkg<A, DRSR, P>(
      &mut self,
      pkg: &mut P,
      pkgs_aux: &mut PkgsAux<A, DRSR, TP>,
    ) -> Result<(), A::Error>
    where
      A: Api,
      P: Package<A, DRSR, Self::Inner, TP>,
    {
      send_pkg(
        &mut self.lock(&pkgs_aux.tp.lease_mut().ext_req_params_mut().uri.to_ref()).await?.client,
        pkg,
        pkgs_aux,
      )
      .await
    }
  }
  impl<AUX, HD, RL, RM, SW, TP> Transport<TP> for &ClientPool<RL, RM>
  where
    HD: RefCounter,
    HD::Item: Lock<Resource = Http2Data<Http2Buffer, SW, true>>,
    RL: Lock<Resource = SimplePoolResource<RM::Resource>>,
    RM: ResourceManager<
        CreateAux = str,
        Error = crate::Error,
        RecycleAux = str,
        Resource = ClientPoolResource<AUX, Http2<HD, true>>,
      >,
    SW: StreamWriter,
    for<'any> RL: 'any,
    for<'any> RM: 'any,
  {
    const GROUP: TransportGroup = TransportGroup::HTTP;
    type Inner = Http2<HD, true>;
  }
}
