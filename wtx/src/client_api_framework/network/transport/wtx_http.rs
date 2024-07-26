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
  http::{Client, Header, Headers, KnownHeaderName, ReqResBuffer},
  http2::{Http2, Http2Buffer, Http2Data},
  misc::{Lock, RefCounter, Stream, Vector},
  pool::SimplePoolResource,
};
use core::{mem, ops::Range};

impl<DRSR, HD, RL, S, SF> Transport<DRSR> for Client<HD, RL, SF>
where
  HD: RefCounter + 'static,
  HD::Item: Lock<Resource = Http2Data<Http2Buffer<ReqResBuffer>, ReqResBuffer, S, true>>,
  RL: Lock<Resource = SimplePoolResource<Http2<HD, true>>> + 'static,
  S: Stream,
  SF: Future<Output = crate::Result<S>> + 'static,
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

async fn response<A, DRSR, HD, P, RL, S, SF>(
  client: &mut Client<HD, RL, SF>,
  pkg: &mut P,
  pkgs_aux: &mut PkgsAux<A, DRSR, HttpParams>,
) -> Result<(), A::Error>
where
  A: Api,
  P: Package<A, DRSR, HttpParams>,
  HD: RefCounter + 'static,
  HD::Item: Lock<Resource = Http2Data<Http2Buffer<ReqResBuffer>, ReqResBuffer, S, true>>,
  RL: Lock<Resource = SimplePoolResource<Http2<HD, true>>> + 'static,
  S: Stream,
  SF: Future<Output = crate::Result<S>> + 'static,
{
  manage_before_sending_related(pkg, pkgs_aux, &mut *client).await?;
  if let Some(mime) = pkgs_aux.tp.ext_req_params().mime {
    pkgs_aux.tp.ext_req_params_mut().headers.push_front(
      Header {
        is_sensitive: false,
        is_trailer: false,
        name: KnownHeaderName::ContentType.into(),
        value: mime._as_str().as_bytes(),
      },
      &[],
    )?;
  }
  if let Some(elem) = pkgs_aux.tp.ext_req_params().user_agent {
    pkgs_aux.tp.ext_req_params_mut().headers.push_front(
      Header {
        is_sensitive: false,
        is_trailer: false,
        name: KnownHeaderName::UserAgent.into(),
        value: elem._as_str().as_bytes(),
      },
      &[],
    )?;
  }
  let (mut rrb_data, mut rrb_headers) = (Vector::new(), Headers::new(0));
  {
    mem::swap(&mut rrb_data, &mut pkgs_aux.byte_buffer);
    mem::swap(&mut rrb_headers, &mut pkgs_aux.tp.ext_req_params_mut().headers);
  }
  let rrb = ReqResBuffer::new(rrb_data, rrb_headers);
  let res = (*client).send(pkgs_aux.tp.ext_req_params().method, rrb).await?;
  pkgs_aux.tp.ext_res_params_mut().status_code = res.status_code;
  {
    let (mut rrb_data, mut rrb_headers) = res.rrd.into_parts();
    mem::swap(&mut pkgs_aux.byte_buffer, &mut rrb_data);
    mem::swap(&mut pkgs_aux.tp.ext_req_params_mut().headers, &mut rrb_headers);
  }
  manage_after_sending_related(pkg, pkgs_aux).await?;
  pkgs_aux.tp.reset();
  Ok(())
}
