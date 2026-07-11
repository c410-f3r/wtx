use crate::{
  executor::Executor,
  http::{
    HttpRecvParams,
    http2_client_pool::{Http2ClientPoolResource, Http2Resource},
  },
  http2::{Http2, Http2Buffer},
  misc::{Lease as _, TcpParams, UriRef},
  pool::ResourceManager,
  rng::ChaCha20,
  stream::{Stream, StreamReader, StreamWriter},
  sync::{Arc, AtomicCell},
  tls::{TlsConfig, TlsConnectorBuilder, TlsMode},
};
use core::fmt::Debug;

/// Resource manager for `ClientPool`.
#[derive(Debug)]
pub struct Http2RM<EX, TM> {
  pub(crate) disable_auto_sni: bool,
  pub(crate) executor: EX,
  pub(crate) hrp: HttpRecvParams,
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
    let mut tc_placeholder;
    let tc = if self.disable_auto_sni {
      &*self.tls_config
    } else {
      tc_placeholder = self.tls_config.lease().clone();
      push_server_name(&mut tc_placeholder, &uri)?;
      &tc_placeholder
    };
    let tls_stream = TlsConnectorBuilder::new(EX::default(), uri)
      .set_tcp_params(self.tcp_params)
      .build(&tc, &self.rng)
      .await?
      .connect()
      .await?
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
    let mut hb = Http2Buffer::default();
    let mut tc_placeholder;
    let tc = if self.disable_auto_sni {
      &*self.tls_config
    } else {
      tc_placeholder = self.tls_config.lease().clone();
      push_server_name(&mut tc_placeholder, &uri)?;
      &tc_placeholder
    };
    let tls_stream = TlsConnectorBuilder::new(EX::default(), uri)
      .set_tcp_params(self.tcp_params)
      .build(&tc, &self.rng)
      .await?
      .connect()
      .await?
      .tls_stream;
    resource.client.swap_buffers(&mut hb).await;
    let (frame_reader, http2) = Http2::connect(hb, self.hrp, tls_stream.into_split()?).await?;
    let _jh = self.executor.spawn(frame_reader);
    resource.client = http2;
    Ok(())
  }
}

fn push_server_name<S, TM>(tc: &mut TlsConfig<TM>, uri: &crate::misc::Uri<S>) -> crate::Result<()>
where
  S: crate::misc::Lease<str>,
{
  tc.server_name_mut()
    .get_or_insert_default()
    .server_name_list
    .push(crate::tls::ServerName::from_name(uri.hostname().try_into()?))?;
  Ok(())
}
