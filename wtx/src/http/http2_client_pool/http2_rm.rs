use crate::{
  collection::Vector,
  executor::{Executor, TcpStream},
  http::{
    HttpRecvParams,
    http2_client_pool::{Http2ClientPoolResource, Http2Resource},
  },
  http2::{Http2, Http2Buffer},
  misc::UriRef,
  pool::ResourceManager,
  rng::{ChaCha20, CryptoSeedableRng},
  sync::AtomicCell,
  tls::{TlsConfig, TlsConnector, TlsMode},
};
use alloc::boxed::Box;

/// Resource manager for `ClientPool`.
#[derive(Debug)]
pub struct Http2RM<EX, TM> {
  pub(crate) executor: EX,
  pub(crate) hrp: HttpRecvParams,
  pub(crate) rng: AtomicCell<ChaCha20>,
  pub(crate) tls_mode: TM,
}

// * `AA`: Auxiliary Argument
// * `AF`: Auxiliary Function
// * `AO`: Auxiliary Output
impl<EX, TM> ResourceManager for Http2RM<EX, TM>
where
  EX: Executor,
  EX::TcpStream: 'static,
  TM: Clone + TlsMode,
  <EX::TcpStream as TcpStream>::ReadHalf: Send,
  <EX::TcpStream as TcpStream>::WriteHalf: Send,
{
  type CreateAux = str;
  type Error = crate::Error;
  type RecycleAux = str;
  type Resource = Http2Resource<<EX::TcpStream as TcpStream>::WriteHalf>;

  #[inline]
  async fn create(&self, ca: &Self::CreateAux) -> Result<Self::Resource, Self::Error> {
    let uri = UriRef::new(ca);
    let mut rng = ChaCha20::from_crypto_rng(&mut &self.rng)?;
    let stream = EX::TcpStream::connect(uri.hostname_with_implied_port()).await?;
    let mut hb = Http2Buffer::default();
    let tls_stream = TlsConnector::from_stream(stream)
      .tls_mode(self.tls_mode.clone())
      .connect(&mut hb.pfb, None, &mut rng, &&TlsConfig::uncertified(), &mut Vector::new())
      .await?;
    let (frame_reader, http2) = Http2::connect(
      hb,
      self.hrp,
      tls_stream.into_split(|local_stream| local_stream.into_split())?,
    )
    .await?;
    let _jh = self.executor.spawn(Box::pin(frame_reader));
    Ok(Http2ClientPoolResource { client: http2 })
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
    let mut rng = ChaCha20::from_crypto_rng(&mut &self.rng)?;
    let stream = EX::TcpStream::connect(uri.hostname_with_implied_port()).await?;
    let mut hb = Http2Buffer::default();
    let tls_stream = TlsConnector::from_stream(stream)
      .tls_mode(self.tls_mode.clone())
      .connect(&mut hb.pfb, None, &mut rng, &&TlsConfig::uncertified(), &mut Vector::new())
      .await?;
    resource.client.swap_buffers(&mut hb).await;
    let (frame_reader, http2) = Http2::connect(
      hb,
      self.hrp,
      tls_stream.into_split(|local_stream| local_stream.into_split())?,
    )
    .await?;
    let _jh = self.executor.spawn(Box::pin(frame_reader));
    resource.client = http2;
    Ok(())
  }
}
