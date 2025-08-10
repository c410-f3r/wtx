// Common elements shared between pure WebSocket structures. Tunneling protocols should use
// the functions provided in `web_socket_reader` and `web_socket_writer`.

pub(crate) mod web_socket_generic;
pub(crate) mod web_socket_mut;
pub(crate) mod web_socket_owned;
