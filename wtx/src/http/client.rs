use crate::{
  http::{Headers, Method, Request, Response},
  http2::{Http2, Http2Buffer, Http2Data, ReqResBuffer},
  misc::{Lock, LockGuard, RefCounter, SingleTypeStorage, Stream, Uri},
  pool::{Pool, SimpleRM},
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
  P: Pool<
    ResourceManager = SimpleRM<
      crate::Error,
      (),
      (ReqResBuffer, Http2<Http2Buffer, S, SDC, true>),
    >,
  >,
  S: Stream,
  SDC: RefCounter,
  SDC::Item: Lock<Http2Data<Http2Buffer, S, true>>,
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
      <P::Guard<'this> as LockGuard<
        'this,
        (ReqResBuffer, Http2<Http2Buffer, S, SDC, true>),
      >>::Mapped<ReqResBuffer>,
    >,
  > {
    let mut guard = self.pool.get(&(), &()).await?;
    let (buffer, http2) = &mut *guard;
    let stream = http2.stream().await.unwrap();
    stream
      .send_req(Request::http2(&[], &Headers::new(0), Method::Get, Uri::new(uri)))
      .await
      .unwrap();
    let res = stream.recv_res(buffer).await?;
    let status = res.status_code;
    let version = res.version;
    Ok(Response {
      data: <P as Pool>::Guard::map(guard, |el| &mut el.0),
      status_code: status,
      version,
    })
  }
}
