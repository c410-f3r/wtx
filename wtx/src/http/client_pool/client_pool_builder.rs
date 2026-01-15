use crate::{
  collection::Vector,
  http::{
    client_pool::{ClientPool, ClientPoolRM},
    conn_params::ConnParams,
  },
  pool::{ResourceManager, SimplePool},
};
use core::marker::PhantomData;

/// Allows the customization of parameters that control HTTP requests and responses.
#[derive(Debug)]
pub struct ClientPoolBuilder<AA, AF, S> {
  aux_arg: AA,
  aux_fun: AF,
  cert: Option<Vector<u8>>,
  cp: ConnParams,
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
  pub fn cert(mut self, cert: Vector<u8>) -> Self {
    self.cert = Some(cert);
    self
  }

  _conn_params_methods!();
}

#[cfg(all(feature = "http-client-pool", feature = "tls", feature = "tokio"))]
impl<S> ClientPoolBuilder<(), crate::http::client_pool::tokio::NoAuxFn, S> {
  pub(crate) const fn no_fun(len: usize) -> Self {
    const fn fun(_: &()) {}
    Self { cert: None, cp: ConnParams::new(), aux_arg: (), aux_fun: fun, len, phantom: PhantomData }
  }
}

impl<AA, AF, S> ClientPoolBuilder<AA, AF, S>
where
  ClientPoolRM<AA, AF, S>: ResourceManager,
{
  /// Creates a new client with inner parameters.
  #[inline]
  pub fn build(self) -> ClientPool<ClientPoolRM<AA, AF, S>> {
    ClientPool {
      pool: SimplePool::new(
        self.len,
        ClientPoolRM {
          _aux_arg: self.aux_arg,
          _aux_fun: self.aux_fun,
          _cert: self.cert,
          _cp: self.cp,
          _phantom: PhantomData,
        },
      ),
    }
  }
}
