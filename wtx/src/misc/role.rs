/// Abstraction that tells if a local protocol instance is a client or a server
pub trait Role {
  /// See [`RoleTy`].
  const TY: RoleTy;
}

/// Local instance is a client, i.e., opens connections.
#[derive(Debug)]
pub struct Client;
impl Role for Client {
  const TY: RoleTy = RoleTy::Client;
}

/// Local instance is a server, i.e., listens for connections.
#[derive(Debug)]
pub struct Server;
impl Role for Server {
  const TY: RoleTy = RoleTy::Server;
}

/// Represents the type of role a local protocol instance fulfills.
#[derive(Debug, Eq, PartialEq)]
pub enum RoleTy {
  /// The local instance is a client that opens connections.
  Client,
  /// The local instance is a server that listens for connections.
  Server,
}
