use crate::{
  http::{Client, ClientParams, ClientRM},
  misc::Lock,
  pool::{ResourceManager, SimplePool, SimplePoolResource},
};
use std::marker::PhantomData;

/// Allows the customization of parameters that control HTTP requests and responses.
#[derive(Debug)]
pub struct ClientBuilder<RL> {
  cp: ClientParams,
  len: usize,
  phantom: PhantomData<RL>,
}

impl<RL> ClientBuilder<RL>
where
  RL: Lock<Resource = SimplePoolResource<<ClientRM as ResourceManager>::Resource>>,
  for<'any> RL: 'any,
{
  #[inline]
  pub(crate) fn new(len: usize) -> Self {
    Self { cp: ClientParams::default(), len, phantom: PhantomData }
  }

  /// Creates a new client with inner parameters.
  #[inline]
  pub fn build(self) -> Client<RL, ClientRM> {
    Client { pool: SimplePool::new(self.len, ClientRM { cp: self.cp }) }
  }

  /// The maximum number of data bytes or the sum of all frames that composed the body data;.
  #[inline]
  pub fn max_body_len(mut self, elem: u32) -> Self {
    self.cp.max_body_len = elem;
    self
  }

  /// The maximum number of bytes of the entire set of headers.
  #[inline]
  pub fn max_headers_len(mut self, elem: u32) -> Self {
    self.cp.max_headers_len = elem;
    self
  }
}
