use crate::http::client_pool::ConnParams;
use core::marker::PhantomData;

/// Resource manager for `ClientPool`.
#[derive(Debug)]
pub struct ClientPoolRM<F, S> {
  pub(crate) _cp: ConnParams,
  pub(crate) _fun: F,
  pub(crate) _phantom: PhantomData<S>,
}
