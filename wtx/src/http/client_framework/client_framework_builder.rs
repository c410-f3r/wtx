use crate::{
  http::{ClientFramework, ClientFrameworkRM, ClientParams},
  misc::Lock,
  pool::{ResourceManager, SimplePool, SimplePoolResource},
};
use core::marker::PhantomData;

/// Allows the customization of parameters that control HTTP requests and responses.
#[derive(Debug)]
pub struct ClientFrameworkBuilder<RL, S> {
  cp: ClientParams,
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
    Self { cp: ClientParams::default(), len, phantom: PhantomData }
  }

  /// Creates a new client with inner parameters.
  #[inline]
  pub fn build(self) -> ClientFramework<RL, ClientFrameworkRM<S>> {
    ClientFramework {
      pool: SimplePool::new(self.len, ClientFrameworkRM { _cp: self.cp, _phantom: PhantomData }),
    }
  }

  /// The maximum number of data bytes or the sum of all frames that composed the body data;.
  #[inline]
  pub fn max_body_len(mut self, elem: u32) -> Self {
    self.cp._max_body_len = elem;
    self
  }

  /// The maximum number of bytes of the entire set of headers.
  #[inline]
  pub fn max_headers_len(mut self, elem: u32) -> Self {
    self.cp._max_headers_len = elem;
    self
  }
}
