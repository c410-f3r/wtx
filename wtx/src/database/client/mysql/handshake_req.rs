use crate::database::client::mysql::auth_plugin::AuthPlugin;

pub(crate) struct HandshakeReq<'bytes> {
  pub(crate) auth_plugin: Option<AuthPlugin>,
  pub(crate) auth_response: Option<&'bytes [u8]>,
  pub(crate) collation: u8,
  pub(crate) database: Option<&'bytes [u8]>,
  pub(crate) max_packet_size: u32,
  pub(crate) username: &'bytes [u8],
}

impl<'bytes> HandshakeReq<'bytes> {
  pub(crate) async fn write(&self) -> crate::Result<()> {
    Ok(())
  }
}
