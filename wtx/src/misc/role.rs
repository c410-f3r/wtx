/// If an instance is a client or a server
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Role {
  /// Connects to a remote peer.
  Client,
  /// Accepts a remote connection.
  Server,
}

impl Role {
  /// Instance shortcut.
  #[inline]
  pub fn from_is_client(is_client: bool) -> Self {
    if is_client {
      Self::Client
    } else {
      Self::Server
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
