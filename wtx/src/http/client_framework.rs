mod client_builder;
mod client_params;
#[cfg(all(
  feature = "_integration-tests",
  feature = "tokio-rustls",
  feature = "webpki-roots",
  test
))]
mod integration_tests;
mod req_builder;

use crate::{
  http::{Method, ReqResBuffer, ReqResData, ReqUri, Request, Response},
  http2::{Http2, Http2Buffer, Http2Data, Http2Params},
  misc::{LeaseMut, Lock, RefCounter, Stream},
  pool::{Pool, ResourceManager, SimplePool, SimplePoolResource},
};
use core::marker::PhantomData;

pub use client_builder::ClientBuilder;
pub(crate) use client_params::ClientParams;
pub use req_builder::ReqBuilder;
#[cfg(feature = "tokio")]
pub use tokio::ClientTokio;
#[cfg(all(feature = "tokio-rustls", feature = "webpki-roots"))]
pub use tokio_rustls::ClientTokioRustls;

/// An optioned pool of different HTTP/2 connections lazily constructed from different URIs.
///
/// Currently supports only one domain with multiple connections.
#[derive(Clone, Debug)]
pub struct ClientFramework<RL, RM> {
  pool: SimplePool<RL, RM>,
}

/// Resource manager for [`Client`].
#[derive(Debug)]
pub struct ClientFrameworkRM<S> {
  _cp: ClientParams,
  _phantom: PhantomData<S>,
}

impl<HD, RL, RM, RRB, S> ClientFramework<RL, RM>
where
  HD: RefCounter + 'static,
  HD::Item: Lock<Resource = Http2Data<Http2Buffer<RRB>, RRB, S, true>>,
  RL: Lock<Resource = SimplePoolResource<RM::Resource>>,
  RM: ResourceManager<
    CreateAux = str,
    Error = crate::Error,
    RecycleAux = str,
    Resource = Http2<HD, true>,
  >,
  RRB: LeaseMut<ReqResBuffer> + ReqResData,
  S: Stream,
  for<'any> RL: 'any,
  for<'any> RM: 'any,
{
  /// Sends an arbitrary request.
  ///
  /// If the pool is full, then this method will block until a connection is available.
  #[inline]
  pub async fn send(
    &self,
    method: Method,
    rrb: RRB,
    req_uri: impl Into<ReqUri<'_>>,
  ) -> crate::Result<Response<RRB>> {
    let actual_req_uri = req_uri.into();
    let uri = match actual_req_uri {
      ReqUri::Data => &rrb.lease().uri(),
      ReqUri::Param(elem) => elem,
    };
    let mut guard = self.pool.get(uri.as_str(), uri.as_str()).await?;
    let mut stream = guard.stream().await?;
    stream.send_req(Request::http2(method, rrb.lease()), actual_req_uri).await?;
    let (res_rrb, status_code) = stream.recv_res(rrb).await?;
    Ok(Response::http2(res_rrb, status_code))
  }
}

#[cfg(feature = "tokio")]
mod tokio {
  use crate::{
    http::{
      client_framework::_hp, ClientBuilder, ClientFramework, ClientFrameworkRM, ReqResBuffer,
    },
    http2::{Http2Buffer, Http2Tokio},
    misc::UriRef,
    pool::{ResourceManager, SimplePoolResource},
  };
  use tokio::{net::TcpStream, sync::Mutex};

  /// A [`Client`] using the elements of `tokio`.
  pub type ClientTokio =
    ClientFramework<Mutex<SimplePoolResource<Instance>>, ClientFrameworkRM<TcpStream>>;
  type Instance = Http2Tokio<Http2Buffer<ReqResBuffer>, ReqResBuffer, TcpStream, true>;

  impl ClientTokio {
    /// Creates a new builder with the maximum number of connections delimited by `len`.
    ///
    /// Connection is established using the elements provided by the `tokio` project.
    #[inline]
    pub fn tokio(len: usize) -> ClientBuilder<Mutex<SimplePoolResource<Instance>>, TcpStream> {
      ClientBuilder::_new(len)
    }
  }

  impl ResourceManager for ClientFrameworkRM<TcpStream> {
    type CreateAux = str;
    type Error = crate::Error;
    type RecycleAux = str;
    type Resource = Instance;

    #[inline]
    async fn create(&self, aux: &Self::CreateAux) -> Result<Self::Resource, Self::Error> {
      let uri = UriRef::new(aux);
      Http2Tokio::connect(
        Http2Buffer::default(),
        _hp(&self._cp),
        TcpStream::connect(uri.host()).await?,
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
      let uri = UriRef::new(aux);
      let mut buffer = Http2Buffer::default();
      resource._swap_buffers(&mut buffer).await;
      let stream = TcpStream::connect(uri.host()).await?;
      *resource = Http2Tokio::connect(buffer, _hp(&self._cp), stream).await?;
      Ok(())
    }
  }
}

#[cfg(all(feature = "tokio-rustls", feature = "webpki-roots"))]
mod tokio_rustls {
  use crate::{
    http::{
      client_framework::_hp, ClientBuilder, ClientFramework, ClientFrameworkRM, ReqResBuffer,
    },
    http2::{Http2Buffer, Http2Tokio},
    misc::{TokioRustlsConnector, UriRef},
    pool::{ResourceManager, SimplePoolResource},
  };
  use tokio::{net::TcpStream, sync::Mutex};
  use tokio_rustls::client::TlsStream;

  /// A [`Client`] using the elements of `tokio-rustls`.
  pub type ClientTokioRustls =
    ClientFramework<Mutex<SimplePoolResource<Instance>>, ClientFrameworkRM<TlsStream<TcpStream>>>;
  type Instance = Http2Tokio<Http2Buffer<ReqResBuffer>, ReqResBuffer, TlsStream<TcpStream>, true>;

  impl ClientTokioRustls {
    /// Creates a new builder with the maximum number of connections delimited by `len`.
    ///
    /// Connection is established using the elements provided by the `tokio-rustls` project.
    #[inline]
    pub fn tokio_rustls(
      len: usize,
    ) -> ClientBuilder<Mutex<SimplePoolResource<Instance>>, TlsStream<TcpStream>> {
      ClientBuilder::_new(len)
    }
  }

  impl ResourceManager for ClientFrameworkRM<TlsStream<TcpStream>> {
    type CreateAux = str;
    type Error = crate::Error;
    type RecycleAux = str;
    type Resource = Instance;

    #[inline]
    async fn create(&self, aux: &Self::CreateAux) -> Result<Self::Resource, Self::Error> {
      let uri = UriRef::new(aux);
      Http2Tokio::connect(
        Http2Buffer::default(),
        _hp(&self._cp),
        TokioRustlsConnector::from_webpki_roots()
          .http2()
          .with_tcp_stream(uri.host(), uri.hostname())
          .await?,
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
      let uri = UriRef::new(aux);
      let mut buffer = Http2Buffer::default();
      resource._swap_buffers(&mut buffer).await;
      let stream = TokioRustlsConnector::from_webpki_roots()
        .http2()
        .with_tcp_stream(uri.host(), uri.hostname())
        .await?;
      *resource = Http2Tokio::connect(buffer, _hp(&self._cp), stream).await?;
      Ok(())
    }
  }
}

#[inline]
fn _hp(cp: &ClientParams) -> Http2Params {
  Http2Params::default()
    .set_initial_window_len(cp._initial_window_len)
    .set_max_body_len(cp._max_body_len)
    .set_max_frame_len(cp._max_frame_len)
    .set_max_headers_len(cp._max_headers_len)
}
