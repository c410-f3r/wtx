use cl_aux::Insert;
use futures_lite::Future;
use hashbrown::HashMap;
use simd_json::ValueAccess;

use crate::{
  http::{Headers, Method, Request, Response},
  http2::{ConnectParams, Http2, Http2Buffer, Http2Data, ReqResBuffer},
  misc::{
    Lock, LockGuard, RefCounter, SingleTypeStorage, Stream, TokioRustlsConnector, Uri, UriRef,
  },
  pool::{Pool, ResourceManager, SimpleRM},
  rng::StdRng,
};
use core::marker::PhantomData;

/// A pool of different HTTP connections lazily constructed from different URIs.
#[derive(Debug)]
pub struct Client<P, S> {
  phantom: PhantomData<S>,
  pool: P,
}

impl<P, S, SDC, SDL> Client<P, S>
where
  P: Pool<ResourceManager = SimpleRM<crate::Error, (), (ReqResBuffer, Http2<S, SDC, true>)>>,
  S: Stream,
  SDC: RefCounter<SDL> + SingleTypeStorage<Item = SDL>,
  SDL: Lock<Http2Data<S, true>>,
{
  #[cfg(feature = "tokio-rustls")]
  #[inline]
  pub fn tokio_rustls() -> Self {
    todo!()
  }

  /// Sends a GET request to the specified `url`.
  #[inline]
  pub async fn get<'this>(
    &'this self,
    uri: &str,
  ) -> crate::Result<
    Response<
      <P::Guard<'this> as LockGuard<'this, (ReqResBuffer, Http2<S, SDC, true>)>>::Mapped<
        ReqResBuffer,
      >,
    >,
  > {
    let mut guard = self.pool.get(&(), &()).await?;
    let (buffer, http2) = &mut *guard;
    let stream = http2.stream().await.unwrap();
    let res = stream
      .send_req(Request::http2(&(), &Headers::new(0), Method::Get, Uri::new(uri)), buffer)
      .await
      .unwrap();
    let status = res.status_code;
    let version = res.version;
    Ok(Response {
      data: <P as Pool>::Guard::map(guard, |el| &mut el.0),
      status_code: status,
      version,
    })
  }
}
