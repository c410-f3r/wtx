create_enum! {
  #[derive(Clone, Copy, Debug, PartialEq)]
  pub(crate) enum RecordContentType<u8> {
    ChangeCipherSpec = (20),
    Alert = (21),
    Handshake = (22),
    ApplicationData = (23),
  }
}
