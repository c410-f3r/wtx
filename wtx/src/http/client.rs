use crate::{
  http::{Method, ReqResBuffer, ReqResData, ReqUri, Request, Response},
  http2::{Http2, Http2Buffer, Http2Data, Http2Params},
  misc::{LeaseMut, Lock, RefCounter, Stream, UriString},
  pool::{Pool, ResourceManager, SimplePool, SimplePoolResource},
};
use core::{future::Future, marker::PhantomData};

/// A [Client] composed by tokio parts.
#[cfg(feature = "tokio")]
pub type ClientTokio<S, SF> = Client<
  crate::http2::Http2DataTokio<Http2Buffer<ReqResBuffer>, ReqResBuffer, S, true>,
  tokio::sync::Mutex<
    SimplePoolResource<crate::http2::Http2Tokio<Http2Buffer<ReqResBuffer>, ReqResBuffer, S, true>>,
  >,
  SF,
>;

/// A pool of different HTTP/2 connections lazily constructed from different URIs.
///
/// Currently supports only one domain with multiple connections.
#[derive(Clone, Debug)]
pub struct Client<HD, RL, SF> {
  pool: SimplePool<RL, Http2RM<HD, SF>>,
}

#[cfg(feature = "tokio")]
impl<S, SF> ClientTokio<S, SF>
where
  S: Stream + 'static,
  SF: Future<Output = crate::Result<S>> + 'static,
{
  /// Creates a new instance with the given number of `len` connections.
  #[inline]
  pub fn tokio(len: usize, stream: fn(UriString) -> SF) -> Self {
    Self { pool: SimplePool::new(len, Http2RM { phantom: PhantomData, stream }) }
  }
}

impl<HB, HD, RL, RRB, S, SF> Client<HD, RL, SF>
where
  HB: Default + LeaseMut<Http2Buffer<RRB>>,
  HD: RefCounter + 'static,
  HD::Item: Lock<Resource = Http2Data<HB, RRB, S, true>>,
  RL: Lock<Resource = SimplePoolResource<Http2<HD, true>>> + 'static,
  RRB: LeaseMut<ReqResBuffer> + ReqResData,
  S: Stream,
  SF: Future<Output = crate::Result<S>> + 'static,
{
  /// Sends an arbitrary request to the specified `url`.
  ///
  /// If the pool is full, then this method will block until a connection is available.
  #[inline]
  pub async fn send(&self, method: Method, mut rrb: RRB) -> crate::Result<Response<RRB>> {
    let mut guard = self.pool.get(rrb.uri().as_str(), rrb.uri().as_str()).await?;
    let mut stream = guard.stream().await?;
    stream.send_req(Request::http2(method, rrb.lease_mut()), ReqUri::Data).await?;
    let (res_rrb, status_code) = stream.recv_res(rrb).await?;
    Ok(Response::http2(res_rrb, status_code))
  }
}

#[derive(Clone, Debug)]
pub(crate) struct Http2RM<HD, SF> {
  phantom: PhantomData<HD>,
  stream: fn(UriString) -> SF,
}

impl<HB, HD, RRB, S, SF> ResourceManager for Http2RM<HD, SF>
where
  HB: Default + LeaseMut<Http2Buffer<RRB>>,
  HD: RefCounter,
  HD::Item: Lock<Resource = Http2Data<HB, RRB, S, true>>,
  RRB: LeaseMut<ReqResBuffer>,
  S: Stream,
  SF: Future<Output = crate::Result<S>>,
{
  type CreateAux = str;
  type Error = crate::Error;
  type RecycleAux = str;
  type Resource = Http2<HD, true>;

  #[inline]
  async fn create(&self, aux: &Self::CreateAux) -> Result<Self::Resource, Self::Error> {
    Http2::connect(
      HB::default(),
      Http2Params::default(),
      (self.stream)(UriString::new(aux.into())).await?,
    )
    .await
  }

  #[inline]
  async fn is_invalid(&self, resource: &Self::Resource) -> bool {
    resource.connection_state().await.is_closed()
  }

  #[inline]
  async fn recycle(
    &self,
    aux: &Self::RecycleAux,
    resource: &mut Self::Resource,
  ) -> Result<(), Self::Error> {
    let mut buffer = HB::default();
    resource._swap_buffers(&mut buffer).await;
    let stream = (self.stream)(UriString::new(aux.into())).await?;
    *resource = Http2::connect(buffer, Http2Params::default(), stream).await?;
    Ok(())
  }
}
