use crate::misc::ArrayString;

pub(crate) const MAX_AUTHORITY_LEN: usize = 64;
pub(crate) const MAX_PATH_LEN: usize = 128;
pub(crate) const MAX_SCHEME_LEN: usize = 16;
pub(crate) const MAX_URI_LEN: usize = MAX_SCHEME_LEN + MAX_AUTHORITY_LEN + MAX_PATH_LEN;

#[derive(Debug)]
pub(crate) struct UriBuffer {
  pub(crate) authority: ArrayString<MAX_AUTHORITY_LEN>,
  pub(crate) path: ArrayString<MAX_PATH_LEN>,
  pub(crate) scheme: ArrayString<MAX_SCHEME_LEN>,
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
