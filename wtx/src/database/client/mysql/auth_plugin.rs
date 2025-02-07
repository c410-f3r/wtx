use crate::{
  database::client::mysql::MysqlError,
  misc::{LeaseMut, Stream},
};

#[derive(Clone, Copy, Debug)]
pub enum AuthPlugin {
  CachingSha2,
  MySqlClear,
  MySqlNative,
  Sha256,
}

//impl AuthPlugin {
//  pub(super) async fn mask_pw<S>(
//    self,
//    data: ([u8; 8], &[u8]),
//    pw: &[u8],
//    stream: &mut S,
//  ) -> crate::Result<Vec<u8>>
//  where
//    S: Stream
//  {
//    match self {
//      AuthPlugin::CachingSha2 => Ok(scramble_sha256(pw, nonce).to_vec()),
//      AuthPlugin::MySqlNative => Ok(scramble_sha1(pw, nonce).to_vec()),
//      AuthPlugin::Sha256 => encrypt_rsa(stream, 0x01, pw, nonce).await,
//      AuthPlugin::MySqlClear => {
//        let mut pw_bytes = pw.as_bytes().to_owned();
//        pw_bytes.push(0); // null terminate
//        Ok(pw_bytes)
//      }
//    }
//  }
//
//  pub(super) async fn handle<S>(
//    self,
//    nonce: &Chain<Bytes, Bytes>,
//    packet: Packet<Bytes>,
//    pw: &[u8],
//    stream: &mut S,
//  ) -> crate::Result<bool>
//  where
//    S: Stream
//  {
//    match self {
//      AuthPlugin::CachingSha2 if packet[0] == 0x01 => {
//        match packet[1] {
//          // AUTH_OK
//          0x03 => Ok(true),
//          // AUTH_CONTINUE
//          0x04 => {
//            let payload = encrypt_rsa(stream, 0x02, password, nonce).await?;
//
//            stream.write_packet(&*payload)?;
//            stream.flush().await?;
//
//            Ok(false)
//          }
//          v => Err(err_protocol!(
//            "unexpected result from fast authentication 0x{:x} when expecting \
//                    0x03 (AUTH_OK) or 0x04 (AUTH_CONTINUE)",
//            v
//          )),
//        }
//      }
//      _ => Err(err_protocol!(
//        "unexpected packet 0x{:02x} for auth plugin '{}' during authentication",
//        packet[0],
//        self.name()
//      )),
//    }
//  }
//}

impl From<AuthPlugin> for &'static str {
  #[inline]
  fn from(from: AuthPlugin) -> Self {
    match from {
      AuthPlugin::CachingSha2 => "caching_sha2_password",
      AuthPlugin::MySqlClear => "mysql_clear_password",
      AuthPlugin::MySqlNative => "mysql_native_password",
      AuthPlugin::Sha256 => "sha256_password",
    }
  }
}

impl TryFrom<&[u8]> for AuthPlugin {
  type Error = crate::Error;

  #[inline]
  fn try_from(from: &[u8]) -> Result<Self, Self::Error> {
    Ok(match from {
      b"caching_sha2_password" => AuthPlugin::CachingSha2,
      b"mysql_clear_password" => AuthPlugin::MySqlClear,
      b"mysql_native_password" => AuthPlugin::MySqlNative,
      b"sha256_password" => AuthPlugin::Sha256,
      _ => return Err(MysqlError::UnknownAuthPlugin.into()),
    })
  }
}
