use crate::{
  collection::Vector,
  http::{
    HttpRecvParams,
    client_pool::{ClientPool, ClientPoolRM},
  },
  pool::{ResourceManager, SimplePool},
  rng::ChaCha20,
  sync::AtomicCell,
};
use core::marker::PhantomData;

/// Allows the customization of parameters that control HTTP requests and responses.
#[derive(Debug)]
pub struct ClientPoolBuilder<AA, AF, S> {
  aux_arg: AA,
  aux_fun: AF,
  cert: Option<Vector<u8>>,
  cp: HttpRecvParams,
  len: usize,
  phantom: PhantomData<S>,
}

impl<AA, AF, S> ClientPoolBuilder<AA, AF, S> {
  /// Auxiliary callback.
  #[inline]
  pub fn aux<NAA, NAF>(self, aux_arg: NAA, aux_fun: NAF) -> ClientPoolBuilder<NAA, NAF, S> {
    ClientPoolBuilder {
      aux_arg,
      aux_fun,
      cert: self.cert,
      cp: self.cp,
      len: self.len,
      phantom: self.phantom,
    }
  }

  /// Sets a TLS certificate
  #[inline]
  pub fn cert(mut self, value: Vector<u8>) -> Self {
    self.cert = Some(value);
    self
  }

  /// See [`HttpRecvParams`].
  #[inline]
  pub fn http_conn_params(mut self, value: HttpRecvParams) -> Self {
    self.cp = value;
    self
  }
}

#[cfg(all(feature = "http-client-pool", feature = "tls", feature = "tokio"))]
impl<S> ClientPoolBuilder<(), crate::http::client_pool::tokio::NoAuxFn, S> {
  pub(crate) const fn no_fun(len: usize) -> Self {
    const fn fun(_: &()) {}
    Self {
      aux_arg: (),
      aux_fun: fun,
      cert: None,
      cp: HttpRecvParams::with_optioned_params(),
      len,
      phantom: PhantomData,
    }
  }
}

impl<AA, AF, S> ClientPoolBuilder<AA, AF, S>
where
  ClientPoolRM<AA, AF, S>: ResourceManager,
{
  /// Creates a new client with inner parameters.
  #[inline]
  pub fn build(self, rng: ChaCha20) -> ClientPool<ClientPoolRM<AA, AF, S>> {
    ClientPool {
      pool: SimplePool::new(
        self.len,
        ClientPoolRM {
          _aux_arg: self.aux_arg,
          _aux_fun: self.aux_fun,
          _cert: self.cert,
          _cp: self.cp,
          _phantom: PhantomData,
          _rng: AtomicCell::new(rng),
        },
      ),
    }
  }
}
