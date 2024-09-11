use crate::{
  http::{
    client_framework::{ClientFramework, ClientFrameworkRM},
    ConnParams,
  },
  misc::Lock,
  pool::{ResourceManager, SimplePool, SimplePoolResource},
};
use core::marker::PhantomData;

/// Allows the customization of parameters that control HTTP requests and responses.
#[derive(Debug)]
pub struct ClientFrameworkBuilder<RL, S> {
  cp: ConnParams,
  len: usize,
  phantom: PhantomData<(RL, S)>,
}

impl<RL, S> ClientFrameworkBuilder<RL, S>
where
  ClientFrameworkRM<S>: ResourceManager,
  RL: Lock<Resource = SimplePoolResource<<ClientFrameworkRM<S> as ResourceManager>::Resource>>,
  for<'any> RL: 'any,
  for<'any> S: 'any,
{
  #[inline]
  pub(crate) fn _new(len: usize) -> Self {
    Self { cp: ConnParams::default(), len, phantom: PhantomData }
  }

  /// Creates a new client with inner parameters.
  #[inline]
  pub fn build(self) -> ClientFramework<RL, ClientFrameworkRM<S>> {
    ClientFramework {
      pool: SimplePool::new(self.len, ClientFrameworkRM { _cp: self.cp, _phantom: PhantomData }),
    }
  }

  _conn_params_methods!();
}
