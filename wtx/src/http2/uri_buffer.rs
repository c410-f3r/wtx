use crate::{
  http::{_MAX_AUTHORITY_LEN, _MAX_PATH_LEN, _MAX_SCHEME_LEN},
  misc::ArrayString,
};

#[derive(Debug)]
pub(crate) struct UriBuffer {
  pub(crate) authority: ArrayString<_MAX_AUTHORITY_LEN>,
  pub(crate) path: ArrayString<_MAX_PATH_LEN>,
  pub(crate) scheme: ArrayString<_MAX_SCHEME_LEN>,
}

impl UriBuffer {
  pub(crate) const fn new() -> Self {
    Self { authority: ArrayString::new(), path: ArrayString::new(), scheme: ArrayString::new() }
  }

  pub(crate) fn clear(&mut self) {
    let Self { authority, path, scheme } = self;
    authority.clear();
    path.clear();
    scheme.clear();
  }
}
