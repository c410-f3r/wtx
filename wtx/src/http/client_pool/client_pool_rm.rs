use crate::{collection::Vector, http::HttpRecvParams, rng::ChaCha20, sync::AtomicCell};
use core::marker::PhantomData;

/// Resource manager for `ClientPool`.
#[derive(Debug)]
pub struct ClientPoolRM<AA, AF, S> {
  pub(crate) _aux_arg: AA,
  pub(crate) _aux_fun: AF,
  pub(crate) _cert: Option<Vector<u8>>,
  pub(crate) _cp: HttpRecvParams,
  pub(crate) _phantom: PhantomData<S>,
  pub(crate) _rng: AtomicCell<ChaCha20>,
}
