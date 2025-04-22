use crate::http::client_pool::ConnParams;
use core::marker::PhantomData;

/// Resource manager for `ClientPool`.
#[derive(Debug)]
pub struct ClientPoolRM<A, AI, S> {
  pub(crate) _aux: A,
  pub(crate) _aux_input: AI,
  pub(crate) _cp: ConnParams,
  pub(crate) _phantom: PhantomData<S>,
}
