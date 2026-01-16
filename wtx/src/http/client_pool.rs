//! Structures used to construct a pool of HTTP connections

mod client_pool_builder;
mod client_pool_resource;
mod client_pool_rm;
#[cfg(all(feature = "_integration-tests", feature = "webpki-roots", test))]
mod integration_tests;

use crate::{
  http::conn_params::ConnParams,
  http2::{Http2, Http2Buffer, Http2ErrorCode},
  misc::{LeaseMut, UriRef},
  pool::{ResourceManager, SimplePool, SimplePoolGetElem, SimplePoolResource},
  stream::StreamWriter,
  sync::AsyncMutexGuard,
};
pub use client_pool_builder::ClientPoolBuilder;
pub use client_pool_resource::ClientPoolResource;
pub use client_pool_rm::ClientPoolRM;
#[cfg(all(feature = "tls", feature = "tokio"))]
pub use tokio::ClientPoolTokio;

/// An optioned pool of different HTTP connections lazily constructed from different URIs.
///
/// Currently supports only one domain with multiple connections.
#[derive(Clone, Debug)]
pub struct ClientPool<RM>
where
  RM: ResourceManager,
{
  pool: SimplePool<RM>,
}

impl<AUX, HB, RM, SW> ClientPool<RM>
where
  HB: LeaseMut<Http2Buffer>,
  RM: ResourceManager<
      CreateAux = str,
      Error = crate::Error,
      RecycleAux = str,
      Resource = ClientPoolResource<AUX, Http2<HB, SW, true>>,
    >,
  SW: StreamWriter,
{
  /// Closes all active connections
  #[inline]
  pub async fn close_all(&self) {
    self
      .pool
      .into_for_each(|elem| async move {
        elem.client.send_go_away(Http2ErrorCode::NoError).await;
      })
      .await;
  }

  /// Returns a guard that contains the internal elements.
  #[inline]
  pub async fn lock<'this>(
    &'this self,
    uri: &UriRef<'_>,
  ) -> crate::Result<SimplePoolGetElem<AsyncMutexGuard<'this, SimplePoolResource<RM::Resource>>>>
  where
    AUX: 'this,
    HB: 'this,
    SW: 'this,
  {
    self.pool.get(uri.as_str(), uri.as_str()).await
  }
}

#[cfg(all(feature = "tls", feature = "tokio"))]
mod tokio {
  use crate::{
    http::client_pool::{ClientPool, ClientPoolBuilder, ClientPoolRM, ClientPoolResource},
    http2::{Http2, Http2Buffer},
    misc::UriRef,
    pool::ResourceManager,
    rng::{ChaCha20, SeedableRng},
    tls::{
      TlsBuffer, TlsConfig, TlsConnector, TlsMode, TlsModeVerifyFull, TlsStream, TlsStreamWriter,
    },
  };
  use tokio::net::{TcpStream, tcp::OwnedWriteHalf};

  /// AF [`ClientPool`] using the elements of `tokio`.
  pub type ClientPoolTokio<AA, AF> =
    ClientPool<ClientPoolRM<AA, AF, TlsStream<TcpStream, TlsBuffer, TlsModeVerifyFull, true>>>;
  pub(crate) type NoAuxFn = fn(&());
  type Resource<AUX> =
    ClientPoolResource<AUX, Http2<Http2Buffer, TlsStreamWriter<OwnedWriteHalf>, true>>;

  impl<TM> ClientPoolBuilder<(), NoAuxFn, TlsStream<TcpStream, TlsBuffer, TM, true>> {
    /// Creates a new builder with the maximum number of connections delimited by `len`.
    ///
    /// Connection is established using the elements provided by the `tokio` project.
    #[inline]
    pub const fn tokio(len: usize) -> Self {
      Self::no_fun(len)
    }
  }

  impl<AA, AF, AO> ResourceManager for ClientPoolRM<AA, AF, TlsStream<TcpStream, TlsBuffer, AO, true>>
  where
    AF: Fn(&AA) -> AO,
    AO: TlsMode,
  {
    type CreateAux = str;
    type Error = crate::Error;
    type RecycleAux = str;
    type Resource = Resource<AO>;

    #[inline]
    async fn create(&self, ca: &Self::CreateAux) -> Result<Self::Resource, Self::Error> {
      let uri = UriRef::new(ca);
      let mut rng = ChaCha20::from_rng(&mut &self._rng)?;
      let stream = TcpStream::connect(uri.hostname_with_implied_port()).await?;
      let tls_stream = TlsConnector::default()
        .set_tls_mode((self._aux_fun)(&self._aux_arg))
        .connect(&mut rng, stream, &TlsConfig::default())
        .await?;
      let (frame_reader, http2) = Http2::connect(
        Http2Buffer::default(),
        self._cp._to_hp(),
        tls_stream.into_split(|local_stream| local_stream.into_split()),
      )
      .await?;
      let _jh = tokio::spawn(frame_reader);
      Ok(ClientPoolResource { aux: (self._aux_fun)(&self._aux_arg), client: http2 })
    }

    #[inline]
    fn is_invalid(&self, resource: &Self::Resource) -> bool {
      resource.client.connection_state().is_closed()
    }

    #[inline]
    async fn recycle(
      &self,
      ra: &Self::RecycleAux,
      resource: &mut Self::Resource,
    ) -> Result<(), Self::Error> {
      let uri = UriRef::new(ra);
      let mut rng = ChaCha20::from_rng(&mut &self._rng)?;
      let stream = TcpStream::connect(uri.hostname_with_implied_port()).await?;
      let tls_stream = TlsConnector::default()
        .set_plain_text()
        .connect(&mut rng, stream, &TlsConfig::default())
        .await?;
      let mut buffer = Http2Buffer::default();
      resource.client.swap_buffers(&mut buffer).await;
      let (frame_reader, http2) = Http2::connect(
        buffer,
        self._cp._to_hp(),
        tls_stream.into_split(|local_stream| local_stream.into_split()),
      )
      .await?;
      let _jh = tokio::spawn(frame_reader);
      resource.client = http2;
      Ok(())
    }
  }
}
