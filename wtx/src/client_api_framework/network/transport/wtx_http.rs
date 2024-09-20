use crate::{
  client_api_framework::{
    misc::{manage_after_sending_related, manage_before_sending_related},
    network::{
      transport::{Transport, TransportParams},
      HttpParams, HttpReqParams, TransportGroup,
    },
    pkg::{Package, PkgsAux},
    Api,
  },
  http::{client_framework::ClientFramework, Header, KnownHeaderName, ReqResBuffer},
  http2::{Http2, Http2Buffer, Http2Data},
  misc::{Lock, RefCounter, StreamWriter},
  pool::{ResourceManager, SimplePoolResource},
};
use core::{mem, ops::Range};

impl<DRSR, HD, RL, RM, SW> Transport<DRSR> for ClientFramework<RL, RM>
where
  HD: RefCounter + 'static,
  HD::Item: Lock<Resource = Http2Data<Http2Buffer<ReqResBuffer>, ReqResBuffer, SW, true>>,
  RL: Lock<Resource = SimplePoolResource<RM::Resource>>,
  RM: ResourceManager<
    CreateAux = str,
    Error = crate::Error,
    RecycleAux = str,
    Resource = Http2<HD, true>,
  >,
  SW: StreamWriter,
  for<'any> RL: 'any,
  for<'any> RM: 'any,
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
    P: Package<A, DRSR, HttpParams>,
  {
    response(self, pkg, pkgs_aux).await?;
    Ok(())
  }

  #[inline]
  async fn send_recv<A, P>(
    &mut self,
    pkg: &mut P,
    pkgs_aux: &mut PkgsAux<A, DRSR, HttpParams>,
  ) -> Result<Range<usize>, A::Error>
  where
    A: Api,
    P: Package<A, DRSR, HttpParams>,
  {
    response(self, pkg, pkgs_aux).await?;
    Ok(0..pkgs_aux.byte_buffer.len())
  }
}

impl<DRSR, HD, RL, RM, SW> Transport<DRSR> for &ClientFramework<RL, RM>
where
  HD: RefCounter + 'static,
  HD::Item: Lock<Resource = Http2Data<Http2Buffer<ReqResBuffer>, ReqResBuffer, SW, true>>,
  RL: Lock<Resource = SimplePoolResource<RM::Resource>>,
  RM: ResourceManager<
    CreateAux = str,
    Error = crate::Error,
    RecycleAux = str,
    Resource = Http2<HD, true>,
  >,
  SW: StreamWriter,
  for<'any> RL: 'any,
  for<'any> RM: 'any,
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
    P: Package<A, DRSR, HttpParams>,
  {
    response(self, pkg, pkgs_aux).await?;
    Ok(())
  }

  #[inline]
  async fn send_recv<A, P>(
    &mut self,
    pkg: &mut P,
    pkgs_aux: &mut PkgsAux<A, DRSR, HttpParams>,
  ) -> Result<Range<usize>, A::Error>
  where
    A: Api,
    P: Package<A, DRSR, HttpParams>,
  {
    response(self, pkg, pkgs_aux).await?;
    Ok(0..pkgs_aux.byte_buffer.len())
  }
}

async fn response<A, DRSR, HD, P, RL, RM, SW>(
  client: &ClientFramework<RL, RM>,
  pkg: &mut P,
  pkgs_aux: &mut PkgsAux<A, DRSR, HttpParams>,
) -> Result<(), A::Error>
where
  A: Api,
  P: Package<A, DRSR, HttpParams>,
  HD: RefCounter + 'static,
  HD::Item: Lock<Resource = Http2Data<Http2Buffer<ReqResBuffer>, ReqResBuffer, SW, true>>,
  RL: Lock<Resource = SimplePoolResource<RM::Resource>>,
  RM: ResourceManager<
    CreateAux = str,
    Error = crate::Error,
    RecycleAux = str,
    Resource = Http2<HD, true>,
  >,
  SW: StreamWriter,
  for<'any> RL: 'any,
  for<'any> RM: 'any,
{
  pkgs_aux.byte_buffer.clear();
  pkgs_aux.tp.ext_req_params_mut().headers.clear();
  manage_before_sending_related(pkg, pkgs_aux, client).await?;
  let HttpReqParams { headers, method, mime, uri, user_agent } = pkgs_aux.tp.ext_req_params_mut();
  if let Some(elem) = mime {
    headers.push_from_iter(Header::from_name_and_value(
      KnownHeaderName::ContentType.into(),
      [elem.as_str().as_bytes()],
    ))?;
  }
  if let Some(elem) = user_agent {
    headers.push_from_iter(Header::from_name_and_value(
      KnownHeaderName::UserAgent.into(),
      [elem._as_str().as_bytes()],
    ))?;
  }
  let mut rrb = ReqResBuffer::empty();
  mem::swap(&mut rrb.data, &mut pkgs_aux.byte_buffer);
  mem::swap(&mut rrb.headers, headers);
  let mut res = (*client).send(*method, rrb, &uri.to_ref()).await?;
  mem::swap(&mut pkgs_aux.byte_buffer, &mut res.rrd.data);
  mem::swap(headers, &mut res.rrd.headers);
  pkgs_aux.tp.ext_res_params_mut().status_code = res.status_code;
  manage_after_sending_related(pkg, pkgs_aux).await?;
  pkgs_aux.tp.reset();
  Ok(())
}
