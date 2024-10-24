//! Client framework

mod client_framework_builder;
#[cfg(all(
  feature = "_async-tests",
  feature = "_integration-tests",
  feature = "tokio-rustls",
  feature = "webpki-roots",
  test
))]
mod integration_tests;
mod req_builder;

use crate::{
  http::{conn_params::ConnParams, Method, ReqResBuffer, ReqUri, Request, Response},
  http2::{Http2, Http2Buffer, Http2Data, Http2ErrorCode, Http2RecvStatus},
  misc::{Lock, RefCounter, StreamWriter},
  pool::{Pool, ResourceManager, SimplePool, SimplePoolResource},
};
use core::marker::PhantomData;

pub use client_framework_builder::ClientFrameworkBuilder;
pub use req_builder::ReqBuilder;
#[cfg(feature = "tokio")]
pub use tokio::ClientFrameworkTokio;
#[cfg(feature = "tokio-rustls")]
pub use tokio_rustls::ClientFrameworkTokioRustls;

/// An optioned pool of different HTTP connections lazily constructed from different URIs.
///
/// Currently supports only one domain with multiple connections.
#[derive(Clone, Debug)]
pub struct ClientFramework<RL, RM> {
  pool: SimplePool<RL, RM>,
}

/// Resource manager for [`ClientFramework`].
#[derive(Debug)]
pub struct ClientFrameworkRM<S> {
  _cp: ConnParams,
  _phantom: PhantomData<S>,
}

impl<HD, RL, RM, SW> ClientFramework<RL, RM>
where
  HD: RefCounter + 'static,
  HD::Item: Lock<Resource = Http2Data<Http2Buffer, SW, true>>,
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
  /// Closes all active connections
  #[inline]
  pub async fn close_all(&self) {
    self
      .pool
      ._into_for_each(|elem| async move {
        elem.send_go_away(Http2ErrorCode::NoError).await;
      })
      .await;
  }

  /// Sends an arbitrary request.
  ///
  /// If the pool is full, then this method will block until a connection is available.
  #[inline]
  pub async fn send(
    &self,
    method: Method,
    rrb: ReqResBuffer,
    req_uri: impl Into<ReqUri<'_>>,
  ) -> crate::Result<Response<ReqResBuffer>> {
    let actual_req_uri = req_uri.into();
    let uri = match actual_req_uri {
      ReqUri::Data => &rrb.uri.to_ref(),
      ReqUri::Param(elem) => elem,
    };
    let mut guard = self.pool.get(uri.as_str(), uri.as_str()).await?;
    let mut stream = guard.stream().await?;
    if stream.send_req(Request::http2(method, &rrb), actual_req_uri).await?.is_closed() {
      return Err(crate::Error::ClosedConnection);
    }
    let (hrs, res_rrb) = stream.recv_res(rrb).await?;
    let status_code = match hrs {
      Http2RecvStatus::Eos(elem) => elem,
      _ => return Err(crate::Error::ClosedConnection),
    };
    stream.common().clear(false).await?;
    Ok(Response::http2(res_rrb, status_code))
  }
}

#[cfg(feature = "tokio")]
mod tokio {
  use crate::{
    http::client_framework::{ClientFramework, ClientFrameworkBuilder, ClientFrameworkRM},
    http2::{Http2Buffer, Http2Tokio},
    misc::UriRef,
    pool::{ResourceManager, SimplePoolResource},
  };
  use tokio::{
    net::{tcp::OwnedWriteHalf, TcpStream},
    sync::Mutex,
  };

  /// A [`ClientFramework`] using the elements of `tokio`.
  pub type ClientFrameworkTokio =
    ClientFramework<Mutex<SimplePoolResource<Instance>>, ClientFrameworkRM<TcpStream>>;
  type Instance = Http2Tokio<Http2Buffer, OwnedWriteHalf, true>;

  impl ClientFrameworkTokio {
    /// Creates a new builder with the maximum number of connections delimited by `len`.
    ///
    /// Connection is established using the elements provided by the `tokio` project.
    #[inline]
    pub fn tokio(
      len: usize,
    ) -> ClientFrameworkBuilder<Mutex<SimplePoolResource<Instance>>, TcpStream> {
      ClientFrameworkBuilder::_new(len)
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
      let (frame_reader, http2) = Http2Tokio::connect(
        Http2Buffer::default(),
        self._cp._to_hp(),
        TcpStream::connect(uri.hostname_with_implied_port()).await?.into_split(),
      )
      .await?;
      let _jh = tokio::spawn(frame_reader);
      Ok(http2)
    }

    #[inline]
    async fn is_invalid(&self, resource: &Self::Resource) -> bool {
      resource.connection_state().is_closed()
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
      let (frame_reader, http2) = Http2Tokio::connect(
        buffer,
        self._cp._to_hp(),
        TcpStream::connect(uri.hostname_with_implied_port()).await?.into_split(),
      )
      .await?;
      let _jh = tokio::spawn(frame_reader);
      *resource = http2;
      Ok(())
    }
  }
}

#[cfg(feature = "tokio-rustls")]
mod tokio_rustls {
  use crate::{
    http::client_framework::{ClientFramework, ClientFrameworkBuilder, ClientFrameworkRM},
    http2::{Http2Buffer, Http2Tokio},
    misc::{TokioRustlsConnector, UriRef},
    pool::{ResourceManager, SimplePoolResource},
  };
  use tokio::{io::WriteHalf, net::TcpStream, sync::Mutex};
  use tokio_rustls::client::TlsStream;

  /// A [`ClientFramework`] using the elements of `tokio-rustls`.
  pub type ClientFrameworkTokioRustls =
    ClientFramework<Mutex<SimplePoolResource<Instance>>, ClientFrameworkRM<Writer>>;
  type Instance = Http2Tokio<Http2Buffer, Writer, true>;
  type Writer = WriteHalf<TlsStream<TcpStream>>;

  impl ClientFrameworkTokioRustls {
    /// Creates a new builder with the maximum number of connections delimited by `len`.
    ///
    /// Connection is established using the elements provided by the `tokio-rustls` project.
    #[inline]
    pub fn tokio_rustls(
      len: usize,
    ) -> ClientFrameworkBuilder<Mutex<SimplePoolResource<Instance>>, Writer> {
      ClientFrameworkBuilder::_new(len)
    }
  }

  impl ResourceManager for ClientFrameworkRM<Writer> {
    type CreateAux = str;
    type Error = crate::Error;
    type RecycleAux = str;
    type Resource = Instance;

    #[inline]
    async fn create(&self, aux: &Self::CreateAux) -> Result<Self::Resource, Self::Error> {
      let uri = UriRef::new(aux);
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
      Ok(http2)
    }

    #[inline]
    async fn is_invalid(&self, resource: &Self::Resource) -> bool {
      resource.connection_state().is_closed()
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
      *resource = http2;
      Ok(())
    }
  }
}
