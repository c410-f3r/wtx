use crate::{collection::Vector, http::client_pool::HttpRecvParams};
use core::marker::PhantomData;

/// Resource manager for `ClientPool`.
#[derive(Debug)]
pub struct ClientPoolRM<A, AI, S> {
  pub(crate) _aux: A,
  pub(crate) _aux_input: AI,
  pub(crate) _cert: Option<Vector<u8>>,
  pub(crate) _cp: HttpRecvParams,
  pub(crate) _phantom: PhantomData<S>,
}
