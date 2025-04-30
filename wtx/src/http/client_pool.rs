//! Structures used to construct a pool of HTTP connections

mod client_pool_builder;
mod client_pool_resource;
mod client_pool_rm;
#[cfg(all(
  feature = "_async-tests",
  feature = "_integration-tests",
  feature = "tokio-rustls",
  feature = "webpki-roots",
  test
))]
mod integration_tests;

use crate::{
  http::conn_params::ConnParams,
  http2::{Http2, Http2Buffer, Http2Data, Http2ErrorCode},
  misc::{Lock, RefCounter, UriRef},
  pool::{Pool, ResourceManager, SimplePool, SimplePoolGetElem, SimplePoolResource},
  stream::StreamWriter,
};
pub use client_pool_builder::ClientPoolBuilder;
pub use client_pool_resource::ClientPoolResource;
pub use client_pool_rm::ClientPoolRM;
#[cfg(feature = "tokio")]
pub use tokio::ClientPoolTokio;
#[cfg(feature = "tokio-rustls")]
pub use tokio_rustls::ClientPoolTokioRustls;

type NoAuxFn = fn(&());

/// An optioned pool of different HTTP connections lazily constructed from different URIs.
///
/// Currently supports only one domain with multiple connections.
#[derive(Clone, Debug)]
pub struct ClientPool<RL, RM> {
  pool: SimplePool<RL, RM>,
}

impl<AUX, HD, RL, RM, SW> ClientPool<RL, RM>
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
  /// Closes all active connections
  #[inline]
  pub async fn close_all(&self) {
    self
      .pool
      ._into_for_each(|elem| async move {
        elem.client.send_go_away(Http2ErrorCode::NoError).await;
      })
      .await;
  }

  /// Returns a guard that contains the internal elements.
  #[inline]
  pub async fn lock(
    &self,
    uri: &UriRef<'_>,
  ) -> crate::Result<SimplePoolGetElem<<RL as Lock>::Guard<'_>>> {
    self.pool.get(uri.as_str(), uri.as_str()).await
  }
}

#[cfg(feature = "tokio")]
mod tokio {
  use crate::{
    http::client_pool::{ClientPool, ClientPoolBuilder, ClientPoolRM, ClientPoolResource, NoAuxFn},
    http2::{Http2Buffer, Http2Tokio},
    misc::UriRef,
    pool::{ResourceManager, SimplePoolResource},
  };
  use tokio::{
    net::{TcpStream, tcp::OwnedWriteHalf},
    sync::Mutex,
  };

  /// A [`ClientPool`] using the elements of `tokio`.
  pub type ClientPoolTokio<A, AI, AO> =
    ClientPool<Mutex<SimplePoolResource<Resource<AO>>>, ClientPoolRM<A, AI, TcpStream>>;
  type Resource<AUX> = ClientPoolResource<AUX, Http2Tokio<Http2Buffer, OwnedWriteHalf, true>>;

  impl<AUX> ClientPoolBuilder<NoAuxFn, (), Mutex<SimplePoolResource<Resource<AUX>>>, TcpStream> {
    /// Creates a new builder with the maximum number of connections delimited by `len`.
    ///
    /// Connection is established using the elements provided by the `tokio` project.
    #[inline]
    pub fn tokio(len: usize) -> Self {
      Self::_no_fun(len)
    }
  }

  impl<A, AI, AO> ResourceManager for ClientPoolRM<A, AI, TcpStream>
  where
    A: Fn(&AI) -> AO,
  {
    type CreateAux = str;
    type Error = crate::Error;
    type RecycleAux = str;
    type Resource = Resource<AO>;

    #[inline]
    async fn create(&self, ca: &Self::CreateAux) -> Result<Self::Resource, Self::Error> {
      let uri = UriRef::new(ca);
      let (frame_reader, http2) = Http2Tokio::connect(
        Http2Buffer::default(),
        self._cp._to_hp(),
        TcpStream::connect(uri.hostname_with_implied_port()).await?.into_split(),
      )
      .await?;
      let _jh = tokio::spawn(frame_reader);
      Ok(ClientPoolResource { aux: (self._aux)(&self._aux_input), client: http2 })
    }

    #[inline]
    async fn is_invalid(&self, resource: &Self::Resource) -> bool {
      resource.client.connection_state().is_closed()
    }

    #[inline]
    async fn recycle(
      &self,
      ra: &Self::RecycleAux,
      resource: &mut Self::Resource,
    ) -> Result<(), Self::Error> {
      let uri = UriRef::new(ra);
      let mut buffer = Http2Buffer::default();
      resource.client._swap_buffers(&mut buffer).await;
      let (frame_reader, http2) = Http2Tokio::connect(
        buffer,
        self._cp._to_hp(),
        TcpStream::connect(uri.hostname_with_implied_port()).await?.into_split(),
      )
      .await?;
      let _jh = tokio::spawn(frame_reader);
      resource.client = http2;
      Ok(())
    }
  }
}

#[cfg(feature = "tokio-rustls")]
mod tokio_rustls {
  use crate::{
    http::client_pool::{ClientPool, ClientPoolBuilder, ClientPoolRM, ClientPoolResource, NoAuxFn},
    http2::{Http2Buffer, Http2Tokio},
    misc::{TokioRustlsConnector, UriRef},
    pool::{ResourceManager, SimplePoolResource},
  };
  use tokio::{io::WriteHalf, net::TcpStream, sync::Mutex};
  use tokio_rustls::client::TlsStream;

  /// A [`ClientPool`] using the elements of `tokio-rustls`.
  pub type ClientPoolTokioRustls<A, AI, AO> =
    ClientPool<Mutex<SimplePoolResource<Resource<AO>>>, ClientPoolRM<A, AI, Writer>>;
  type Resource<AUX> = ClientPoolResource<AUX, Http2Tokio<Http2Buffer, Writer, true>>;
  type Writer = WriteHalf<TlsStream<TcpStream>>;

  impl<AUX> ClientPoolBuilder<NoAuxFn, (), Mutex<SimplePoolResource<Resource<AUX>>>, Writer> {
    /// Creates a new builder with the maximum number of connections delimited by `len`.
    ///
    /// Connection is established using the elements provided by the `tokio-rustls` project.
    #[inline]
    pub fn tokio_rustls(len: usize) -> Self {
      Self::_no_fun(len)
    }
  }

  impl<A, AI, AO> ResourceManager for ClientPoolRM<A, AI, Writer>
  where
    A: Fn(&AI) -> AO,
  {
    type CreateAux = str;
    type Error = crate::Error;
    type RecycleAux = str;
    type Resource = Resource<AO>;

    #[inline]
    async fn create(&self, ca: &Self::CreateAux) -> Result<Self::Resource, Self::Error> {
      let uri = UriRef::new(ca);
      let (frame_reader, http2) = Http2Tokio::connect(
        Http2Buffer::default(),
        self._cp._to_hp(),
        tokio::io::split(
          TokioRustlsConnector::from_auto()?
            .http2()
            .connect_without_client_auth(
              uri.hostname(),
              TcpStream::connect(uri.hostname_with_implied_port()).await?,
            )
            .await?,
        ),
      )
      .await?;
      let _jh = tokio::spawn(frame_reader);
      Ok(ClientPoolResource { aux: (self._aux)(&self._aux_input), client: http2 })
    }

    #[inline]
    async fn is_invalid(&self, resource: &Self::Resource) -> bool {
      resource.client.connection_state().is_closed()
    }

    #[inline]
    async fn recycle(
      &self,
      ra: &Self::RecycleAux,
      resource: &mut Self::Resource,
    ) -> Result<(), Self::Error> {
      let uri = UriRef::new(ra);
      let mut buffer = Http2Buffer::default();
      resource.client._swap_buffers(&mut buffer).await;
      let (frame_reader, http2) = Http2Tokio::connect(
        Http2Buffer::default(),
        self._cp._to_hp(),
        tokio::io::split(
          TokioRustlsConnector::from_auto()?
            .http2()
            .connect_without_client_auth(
              uri.hostname(),
              TcpStream::connect(uri.hostname_with_implied_port()).await?,
            )
            .await?,
        ),
      )
      .await?;
      let _jh = tokio::spawn(frame_reader);
      resource.client = http2;
      Ok(())
    }
  }
}
