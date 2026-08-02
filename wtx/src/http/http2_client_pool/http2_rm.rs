use crate::{
  executor::Executor,
  http::{HttpRecvParams, http2_client_pool::Http2ClientPoolResource},
  http2::{Http2, Http2Buffer},
  net::{Stream, StreamReader, StreamWriter, TcpParams, Uri, UriRef},
  pool::ResourceManager,
  rng::ChaCha20,
  sync::{AsyncMutex, AtomicCell},
  tls::{TlsConfig, TlsConnectorBuilder, TlsCtx, TlsStream},
};
use core::fmt::Debug;

/// Resource manager for `ClientPool`.
#[derive(Debug)]
pub struct Http2RM<AUX, EX, TCX> {
  pub(crate) aux_fn: fn() -> AUX,
  pub(crate) disable_auto_sni: bool,
  pub(crate) executor: EX,
  pub(crate) hrp: HttpRecvParams,
  pub(crate) rng: AtomicCell<ChaCha20>,
  pub(crate) tcp_params: TcpParams,
  pub(crate) tls_config: AsyncMutex<TlsConfig<TCX>>,
}

impl<AUX, EX, TCX> Http2RM<AUX, EX, TCX>
where
  EX: Executor,
  TCX: TlsCtx,
{
  async fn tls_stream(&self, aux: &str) -> crate::Result<TlsStream<EX::TcpStream, TCX, true>> {
    let uri = UriRef::new(aux);
    let mut tls_config = self.tls_config.lock().await;
    if !self.disable_auto_sni {
      push_server_name(&mut tls_config, &uri)?;
    }
    Ok(
      TlsConnectorBuilder::new(EX::default(), uri)
        .set_tcp_params(self.tcp_params)
        .build(&*tls_config, &self.rng)
        .await?
        .connect()
        .await?
        .tls_stream,
    )
  }
}

impl<AUX, EX, TCX> ResourceManager for Http2RM<AUX, EX, TCX>
where
  EX: Executor,
  EX::TcpStream: 'static,
  TCX: Send + TlsCtx + 'static,
  <EX::TcpStream as Stream>::ReadHalfOwned: Send,
  <EX::TcpStream as Stream>::WriteHalfOwned: Send,
  <<EX::TcpStream as Stream>::ReadHalfOwned as StreamReader>::read(..): Send,
  <<EX::TcpStream as Stream>::WriteHalfOwned as StreamWriter>::write_all(..): Send,
  <<EX::TcpStream as Stream>::WriteHalfOwned as StreamWriter>::write_all_vectored(..): Send,
{
  type CreateAux = str;
  type Error = crate::Error;
  type RecycleAux = str;
  type Resource = Http2ClientPoolResource<AUX, <EX::TcpStream as Stream>::WriteHalfOwned, TCX>;

  #[inline]
  async fn create(&self, aux: &Self::CreateAux) -> Result<Self::Resource, Self::Error> {
    let tls_stream = self.tls_stream(aux).await?;
    let tuple = Http2::connect(Http2Buffer::default(), self.hrp, tls_stream.into_split()?).await?;
    let _jh = self.executor.spawn(tuple.0);
    Ok(Http2ClientPoolResource { aux: (self.aux_fn)(), client: tuple.1 })
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
    let tls_stream = self.tls_stream(aux).await?;
    let mut hb = Http2Buffer::default();
    resource.client.swap_buffers(&mut hb).await;
    let (frame_reader, http2) = Http2::connect(hb, self.hrp, tls_stream.into_split()?).await?;
    let _jh = self.executor.spawn(frame_reader);
    resource.client = http2;
    Ok(())
  }
}

fn push_server_name<S, TCX>(tc: &mut TlsConfig<TCX>, uri: &Uri<S>) -> crate::Result<()>
where
  S: crate::misc::Lease<str>,
{
  if tc.server_name_mut().is_some() {
    return Ok(());
  }
  tc.server_name_mut()
    .get_or_insert_default()
    .server_name_list
    .push(crate::tls::ServerName::from_name(uri.hostname().try_into()?))?;
  Ok(())
}
