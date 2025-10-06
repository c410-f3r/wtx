use crate::http::OperationMode;

/// Parameters found in a matched route
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RouteMatch {
  pub(crate) idx: u8,
  pub(crate) om: OperationMode,
  pub(crate) path: &'static str,
}

impl RouteMatch {
  pub(crate) const fn new(idx: u8, om: OperationMode, path: &'static str) -> Self {
    Self { idx, om, path }
  }
}
