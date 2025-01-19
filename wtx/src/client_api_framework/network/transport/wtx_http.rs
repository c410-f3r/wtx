use crate::{
  client_api_framework::{
    misc::{manage_after_sending_related, manage_before_sending_related},
    network::{
      transport::{RecievingTransport, SendingTransport, Transport, TransportParams},
      HttpParams, HttpReqParams, TransportGroup,
    },
    pkg::{Package, PkgsAux},
    Api,
  },
  http::{
    client_pool::{ClientPool, ClientPoolResource},
    Header, KnownHeaderName, ReqBuilder, ReqResBuffer, WTX_USER_AGENT,
  },
  http2::{Http2, Http2Buffer, Http2Data},
  misc::{Lock, RefCounter, StreamWriter},
  pool::{ResourceManager, SimplePoolResource},
};
use core::{mem, ops::Range};

impl<AUX, HD, RL, RM, SW> RecievingTransport for ClientPool<RL, RM>
where
  HD: RefCounter + 'static,
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
    pkgs_aux: &mut PkgsAux<A, DRSR, Self::Params>,
  ) -> Result<Range<usize>, A::Error>
  where
    A: Api,
  {
    Ok(0..pkgs_aux.byte_buffer.len())
  }
}

impl<AUX, HD, RL, RM, SW> SendingTransport for ClientPool<RL, RM>
where
  HD: RefCounter + 'static,
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
  async fn send<A, DRSR, P>(
    &mut self,
    pkg: &mut P,
    pkgs_aux: &mut PkgsAux<A, DRSR, HttpParams>,
  ) -> Result<(), A::Error>
  where
    A: Api,
    P: Package<A, DRSR, HttpParams>,
  {
    response(
      &mut self.lock(&pkgs_aux.tp.ext_req_params_mut().uri.to_ref()).await?.client,
      pkg,
      pkgs_aux,
    )
    .await?;
    Ok(())
  }
}

impl<RL, RM> Transport for ClientPool<RL, RM> {
  const GROUP: TransportGroup = TransportGroup::HTTP;
  type Params = HttpParams;
}

impl<AUX, HD, RL, RM, SW> RecievingTransport for &ClientPool<RL, RM>
where
  HD: RefCounter + 'static,
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
    pkgs_aux: &mut PkgsAux<A, DRSR, Self::Params>,
  ) -> Result<Range<usize>, A::Error>
  where
    A: Api,
  {
    Ok(0..pkgs_aux.byte_buffer.len())
  }
}

impl<AUX, HD, RL, RM, SW> SendingTransport for &ClientPool<RL, RM>
where
  HD: RefCounter + 'static,
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
  async fn send<A, DRSR, P>(
    &mut self,
    pkg: &mut P,
    pkgs_aux: &mut PkgsAux<A, DRSR, HttpParams>,
  ) -> Result<(), A::Error>
  where
    A: Api,
    P: Package<A, DRSR, HttpParams>,
  {
    response(
      &mut self.lock(&pkgs_aux.tp.ext_req_params_mut().uri.to_ref()).await?.client,
      pkg,
      pkgs_aux,
    )
    .await?;
    Ok(())
  }
}

impl<AUX, HD, RL, RM, SW> Transport for &ClientPool<RL, RM>
where
  HD: RefCounter + 'static,
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
  type Params = HttpParams;
}

impl<HD, SW> RecievingTransport for Http2<HD, true>
where
  HD: RefCounter,
  HD::Item: Lock<Resource = Http2Data<Http2Buffer, SW, true>>,
  SW: StreamWriter,
{
  #[inline]
  async fn recv<A, DRSR>(
    &mut self,
    pkgs_aux: &mut PkgsAux<A, DRSR, Self::Params>,
  ) -> Result<Range<usize>, A::Error>
  where
    A: Api,
  {
    Ok(0..pkgs_aux.byte_buffer.len())
  }
}

impl<HD, SW> SendingTransport for Http2<HD, true>
where
  HD: RefCounter,
  HD::Item: Lock<Resource = Http2Data<Http2Buffer, SW, true>>,
  SW: StreamWriter,
{
  #[inline]
  async fn send<A, DRSR, P>(
    &mut self,
    pkg: &mut P,
    pkgs_aux: &mut PkgsAux<A, DRSR, HttpParams>,
  ) -> Result<(), A::Error>
  where
    A: Api,
    P: Package<A, DRSR, HttpParams>,
  {
    response(self, pkg, pkgs_aux).await?;
    Ok(())
  }
}

impl<HD, SW> Transport for Http2<HD, true>
where
  HD: RefCounter,
  HD::Item: Lock<Resource = Http2Data<Http2Buffer, SW, true>>,
  SW: StreamWriter,
{
  const GROUP: TransportGroup = TransportGroup::HTTP;
  type Params = HttpParams;
}

#[inline]
async fn response<A, DRSR, HD, P, SW>(
  mut client: &mut Http2<HD, true>,
  pkg: &mut P,
  pkgs_aux: &mut PkgsAux<A, DRSR, HttpParams>,
) -> Result<(), A::Error>
where
  A: Api,
  P: Package<A, DRSR, HttpParams>,
  HD: RefCounter,
  HD::Item: Lock<Resource = Http2Data<Http2Buffer, SW, true>>,
  SW: StreamWriter,
{
  pkgs_aux.byte_buffer.clear();
  pkgs_aux.tp.ext_req_params_mut().headers.clear();
  manage_before_sending_related(pkg, pkgs_aux, &mut client).await?;
  let HttpReqParams { headers, method, mime, uri } = pkgs_aux.tp.ext_req_params_mut();
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
  let mut res = ReqBuilder::get(rrb).method(*method).send(client, &uri.to_ref()).await?;
  mem::swap(&mut pkgs_aux.byte_buffer, &mut res.rrd.body);
  mem::swap(headers, &mut res.rrd.headers);
  pkgs_aux.tp.ext_res_params_mut().status_code = res.status_code;
  manage_after_sending_related(pkg, pkgs_aux).await?;
  pkgs_aux.tp.reset();
  Ok(())
}
