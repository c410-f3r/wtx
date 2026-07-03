use crate::{
  executor::{Executor, TcpStream as _},
  http::{
    HttpRecvParams,
    http2_client_pool::{Http2ClientPoolResource, Http2Resource},
  },
  http2::{Http2, Http2Buffer},
  misc::{TcpParams, UriRef},
  pool::ResourceManager,
  rng::ChaCha20,
  stream::{Stream, StreamReader, StreamWriter},
  sync::{Arc, AtomicCell},
  tls::{Psk, TlsConfig, TlsConnector, TlsMode},
};
use core::fmt::Debug;

/// Resource manager for `ClientPool`.
#[derive(Debug)]
pub struct Http2RM<EX, TM> {
  pub(crate) executor: EX,
  pub(crate) hrp: HttpRecvParams,
  pub(crate) psk: Option<AtomicCell<Psk>>,
  pub(crate) rng: AtomicCell<ChaCha20>,
  pub(crate) tcp_params: TcpParams,
  pub(crate) tls_config: Arc<TlsConfig<TM>>,
}

impl<EX, TM> ResourceManager for Http2RM<EX, TM>
where
  EX: Executor,
  EX::TcpStream: 'static,
  TM: Clone + Send + TlsMode + 'static,
  <EX::TcpStream as Stream>::ReadHalfOwned: Send,
  <EX::TcpStream as Stream>::WriteHalfOwned: Send,
  <<EX::TcpStream as Stream>::ReadHalfOwned as StreamReader>::read(..): Send,
  <<EX::TcpStream as Stream>::WriteHalfOwned as StreamWriter>::write_all(..): Send,
  <<EX::TcpStream as Stream>::WriteHalfOwned as StreamWriter>::write_all_vectored(..): Send,
{
  type CreateAux = str;
  type Error = crate::Error;
  type RecycleAux = str;
  type Resource = Http2Resource<<EX::TcpStream as Stream>::WriteHalfOwned, TM>;

  #[inline]
  async fn create(&self, aux: &Self::CreateAux) -> Result<Self::Resource, Self::Error> {
    let uri = UriRef::new(aux);
    let stream = EX::TcpStream::connect(uri.hostname_with_implied_port(), self.tcp_params).await?;
    let tls_stream = TlsConnector::new(&*self.tls_config, &self.rng, stream)
      .set_psk(self.psk.as_ref().map(AtomicCell::load))
      .connect()
      .await?
      .rslt()?
      .tls_stream;
    let tuple = Http2::connect(Http2Buffer::default(), self.hrp, tls_stream.into_split()?).await?;
    let _jh = self.executor.spawn(tuple.0);
    Ok(Http2ClientPoolResource { client: tuple.1 })
  }

  #[inline]
  fn is_invalid(&self, resource: &Self::Resource) -> bool {
    resource.client.connection_state().is_closed()
  }

  #[inline]
  async fn recycle(
    &self,
    aux: &Self::RecycleAux,
    resource: &mut Self::Resource,
  ) -> Result<(), Self::Error> {
    let uri = UriRef::new(aux);
    let stream = EX::TcpStream::connect(uri.hostname_with_implied_port(), self.tcp_params).await?;
    let mut hb = Http2Buffer::default();
    let tcr = TlsConnector::new(&*self.tls_config, &self.rng, stream)
      .set_psk(self.psk.as_ref().map(AtomicCell::load))
      .connect()
      .await?
      .rslt()?;
    resource.client.swap_buffers(&mut hb).await;
    let (frame_reader, http2) = Http2::connect(hb, self.hrp, tcr.tls_stream.into_split()?).await?;
    let _jh = self.executor.spawn(frame_reader);
    resource.client = http2;
    Ok(())
  }
}
