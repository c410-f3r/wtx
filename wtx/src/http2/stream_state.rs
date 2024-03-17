#[derive(Debug, Clone, Copy)]
pub(crate) enum StreamState {
  Closed,
  HalfClosedLocal(Peer),
  HalfClosedRemote(Peer),
  Idle,
  Open { local: Peer, remote: Peer },
  ReservedLocal,
  ReservedRemote,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) enum Peer {
  #[default]
  AwaitingHeaders,
  Streaming,
}
