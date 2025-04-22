use crate::{
  http::{
    client_pool::{ClientPool, ClientPoolRM, NoAuxFn},
    conn_params::ConnParams,
  },
  misc::Lock,
  pool::{ResourceManager, SimplePool, SimplePoolResource},
};
use core::marker::PhantomData;

/// Allows the customization of parameters that control HTTP requests and responses.
#[derive(Debug)]
pub struct ClientPoolBuilder<A, AI, RL, S> {
  aux: A,
  aux_input: AI,
  cp: ConnParams,
  len: usize,
  phantom: PhantomData<(RL, S)>,
}

impl<A, AI, RL, S> ClientPoolBuilder<A, AI, RL, S> {
  /// Auxiliary callback.
  #[inline]
  pub fn aux<NA, NAI>(self, aux: NA, aux_input: NAI) -> ClientPoolBuilder<NA, NAI, RL, S> {
    ClientPoolBuilder { cp: self.cp, aux, aux_input, len: self.len, phantom: self.phantom }
  }

  _conn_params_methods!();
}

impl<RL, S> ClientPoolBuilder<NoAuxFn, (), RL, S> {
  #[inline]
  pub(crate) fn _no_fun(len: usize) -> Self {
    fn fun(_: &()) {}
    Self { cp: ConnParams::default(), aux: fun, aux_input: (), len, phantom: PhantomData }
  }
}

impl<A, AI, RL, S> ClientPoolBuilder<A, AI, RL, S>
where
  RL: Lock<Resource = SimplePoolResource<<ClientPoolRM<A, AI, S> as ResourceManager>::Resource>>,
  for<'any> A: 'any,
  for<'any> AI: 'any,
  for<'any> RL: 'any,
  for<'any> S: 'any,
  ClientPoolRM<A, AI, S>: ResourceManager,
{
  /// Creates a new client with inner parameters.
  #[inline]
  pub fn build(self) -> ClientPool<RL, ClientPoolRM<A, AI, S>> {
    ClientPool {
      pool: SimplePool::new(
        self.len,
        ClientPoolRM {
          _cp: self.cp,
          _aux: self.aux,
          _aux_input: self.aux_input,
          _phantom: PhantomData,
        },
      ),
    }
  }
}
