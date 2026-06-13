create_enum! {
  #[derive(Clone, Copy, Debug)]
  pub enum RecordContentType<u8> {
    ChangeCipherSpec = (20),
    Alert = (21),
    Handshake = (22),
    ApplicationData = (23),
  }
}
