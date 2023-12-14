#[derive(Clone, Copy, Debug)]
pub(crate) enum Role {
  Client,
  Server,
}

impl Role {
  pub(crate) fn from_is_client(is_client: bool) -> Self {
    match is_client {
      true => Self::Client,
      false => Self::Server,
    }
  }
}

impl From<Role> for &'static str {
  #[inline]
  fn from(from: Role) -> Self {
    match from {
      Role::Client => "Client",
      Role::Server => "Server",
    }
  }
}
