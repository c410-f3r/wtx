/// Without this extension, TLS specifies a fixed maximum plaintext
/// fragment length of 2^14 bytes. It may be desirable for constrained
/// clients to negotiate a smaller maximum fragment length due to memory
/// limitations or bandwidth limitations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MaxFragmentLength {
  /// 512 bytes
  _512 = 1,
  /// 1024 bytes
  _1024 = 2,
  /// 2048 bytes
  _2048 = 3,
  /// 4096 bytes
  _4096 = 4,
}
