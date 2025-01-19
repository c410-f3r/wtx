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
pub struct ClientPoolBuilder<F, RL, S> {
  cp: ConnParams,
  fun: F,
  len: usize,
  phantom: PhantomData<(RL, S)>,
}

impl<F, RL, S> ClientPoolBuilder<F, RL, S> {
  /// Auxiliary structure returned by a function.
  #[inline]
  pub fn aux<NF>(self, fun: NF) -> ClientPoolBuilder<NF, RL, S> {
    ClientPoolBuilder { cp: self.cp, fun, len: self.len, phantom: self.phantom }
  }

  _conn_params_methods!();
}

impl<RL, S> ClientPoolBuilder<NoAuxFn, RL, S> {
  #[inline]
  pub(crate) fn _no_aux_fun(len: usize) -> Self {
    fn fun() {}
    Self { cp: ConnParams::default(), fun, len, phantom: PhantomData }
  }
}

impl<F, RL, S> ClientPoolBuilder<F, RL, S>
where
  RL: Lock<Resource = SimplePoolResource<<ClientPoolRM<F, S> as ResourceManager>::Resource>>,
  for<'any> F: 'any,
  for<'any> RL: 'any,
  for<'any> S: 'any,
  ClientPoolRM<F, S>: ResourceManager,
{
  /// Creates a new client with inner parameters.
  #[inline]
  pub fn build(self) -> ClientPool<RL, ClientPoolRM<F, S>> {
    ClientPool {
      pool: SimplePool::new(
        self.len,
        ClientPoolRM { _cp: self.cp, _fun: self.fun, _phantom: PhantomData },
      ),
    }
  }
}
