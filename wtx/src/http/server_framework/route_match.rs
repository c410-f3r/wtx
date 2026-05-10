use crate::{collection::ShortStrU8, http::OperationMode};

/// Parameters found in a matched route
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RouteMatch {
  pub(crate) idx: u8,
  pub(crate) om: OperationMode,
  pub(crate) path: ShortStrU8<'static>,
}

impl RouteMatch {
  pub(crate) const fn new(idx: u8, om: OperationMode, path: ShortStrU8<'static>) -> Self {
    Self { idx, om, path }
  }
}
