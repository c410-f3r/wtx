use crate::{
  collection::{ArrayVectorU8, IndexedStorageMut},
  database::client::mysql::{
    auth_plugin::AuthPlugin,
    capability::Capability,
    collation::Collation,
    misc::encoded_len,
    mysql_protocol::{
      MysqlProtocol, encode_wrapper_protocol::EncodeWrapperProtocol, initial_req::InitialReq,
    },
  },
  de::Encode,
};

pub(crate) struct HandshakeReq<'bytes> {
  pub(crate) auth_plugin: Option<AuthPlugin>,
  pub(crate) auth_response: Option<ArrayVectorU8<u8, 32>>,
  pub(crate) collation: Collation,
  pub(crate) database: Option<&'bytes str>,
  pub(crate) max_packet_size: u32,
  pub(crate) username: &'bytes str,
}

impl<E> Encode<MysqlProtocol<(), E>> for HandshakeReq<'_>
where
  E: From<crate::Error>,
{
  #[inline]
  fn encode(&self, aux: &mut (), ew: &mut EncodeWrapperProtocol<'_>) -> Result<(), E> {
    if self.auth_plugin.is_none() {
      *ew.capabilities &= !u64::from(Capability::PluginAuth);
    }

    let req = InitialReq { collation: self.collation, max_packet_size: self.max_packet_size };
    req.encode(aux, ew)?;
    let _ = ew.encode_buffer.extend_from_copyable_slices([self.username.as_bytes(), b"\0"])?;

    let connect_n = u64::from(Capability::ConnectWithDb);
    let plugin_auth = u64::from(Capability::PluginAuth);
    let plugin_auth_lenenc_n = u64::from(Capability::PluginAuthLenencData);
    let secure_n = u64::from(Capability::SecureConnection);

    if *ew.capabilities & plugin_auth_lenenc_n == plugin_auth_lenenc_n {
      let auth_response = self.auth_response.as_deref().unwrap_or_default();
      let array = [&encoded_len(auth_response.len())?, auth_response];
      let _ = ew.encode_buffer.extend_from_copyable_slices(array)?;
    } else if *ew.capabilities & secure_n == secure_n {
      let auth_response = self.auth_response.as_deref().unwrap_or_default();
      let len = u8::try_from(auth_response.len()).map_err(crate::Error::from)?;
      let _ = ew.encode_buffer.extend_from_copyable_slices([&[len][..], auth_response])?;
    } else {
      ew.encode_buffer.extend_from_copyable_slice("\0".as_bytes())?;
    }

    if *ew.capabilities & connect_n == connect_n {
      if let Some(database) = self.database {
        let _ = ew.encode_buffer.extend_from_copyable_slices([database.as_bytes(), b"\0"])?;
      } else {
        ew.encode_buffer.extend_from_copyable_slice("\0".as_bytes())?;
      }
    }

    if *ew.capabilities & plugin_auth == plugin_auth {
      if let Some(auth_plugin) = self.auth_plugin {
        let array = [<&str>::from(auth_plugin).as_bytes(), b"\0"];
        let _ = ew.encode_buffer.extend_from_copyable_slices(array)?;
      } else {
        ew.encode_buffer.extend_from_copyable_slice("\0".as_bytes())?;
      }
    }

    Ok(())
  }
}
