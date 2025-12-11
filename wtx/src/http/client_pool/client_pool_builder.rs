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
pub struct ClientPoolBuilder<A, AI, S> {
  aux: A,
  aux_input: AI,
  cert: Option<Vector<u8>>,
  cp: ConnParams,
  len: usize,
  phantom: PhantomData<S>,
}

impl<A, AI, S> ClientPoolBuilder<A, AI, S> {
  /// Auxiliary callback.
  #[inline]
  pub fn aux<NA, NAI>(self, aux: NA, aux_input: NAI) -> ClientPoolBuilder<NA, NAI, S> {
    ClientPoolBuilder {
      cert: None,
      cp: self.cp,
      aux,
      aux_input,
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

#[cfg(all(feature = "http-client-pool", feature = "tokio"))]
impl<S> ClientPoolBuilder<crate::http::client_pool::NoAuxFn, (), S> {
  pub(crate) const fn no_fun(len: usize) -> Self {
    const fn fun(_: &()) {}
    Self { cert: None, cp: ConnParams::new(), aux: fun, aux_input: (), len, phantom: PhantomData }
  }
}

impl<A, AI, S> ClientPoolBuilder<A, AI, S>
where
  for<'any> A: 'any,
  for<'any> AI: 'any,
  for<'any> S: 'any,
  ClientPoolRM<A, AI, S>: ResourceManager,
{
  /// Creates a new client with inner parameters.
  #[inline]
  pub fn build(self) -> ClientPool<ClientPoolRM<A, AI, S>> {
    ClientPool {
      pool: SimplePool::new(
        self.len,
        ClientPoolRM {
          _cert: self.cert,
          _cp: self.cp,
          _aux: self.aux,
          _aux_input: self.aux_input,
          _phantom: PhantomData,
        },
      ),
    }
  }
}
