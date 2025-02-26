use crate::{
  database::client::mysql::{
    MysqlError,
    misc::{fetch_msg, write_packet},
  },
  misc::{ArrayVector, Stream, Vector, partitioned_filled_buffer::PartitionedFilledBuffer},
};
use sha2::{Digest, Sha256};
//use rsa::{pkcs8::DecodePublicKey, Oaep, RsaPublicKey};

#[derive(Clone, Copy, Debug)]
pub(crate) enum AuthPlugin {
  CachingSha2,
  MySqlClear,
}

impl AuthPlugin {
  #[inline]
  pub(crate) async fn manage_caching_sha2<E, S, const IS_TLS: bool>(
    self,
    auth_plugin_data: ([u8; 8], &[u8]),
    bytes: [u8; 2],
    (capabilities, sequence_id): (&mut u64, &mut u8),
    enc_buffer: &mut Vector<u8>,
    net_buffer: &mut PartitionedFilledBuffer,
    password: &str,
    stream: &mut S,
  ) -> Result<bool, E>
  where
    E: From<crate::Error>,
    S: Stream,
  {
    let [a, b] = bytes;
    match self {
      AuthPlugin::CachingSha2 if a == 1 => match b {
        3 => Ok(true),
        4 => {
          let mut pw_array: ArrayVector<u8, 32> = password.as_bytes().try_into()?;
          pw_array.push(0)?;
          if IS_TLS {
            //return Ok(to_asciz(password));
          }

          write_packet((capabilities, sequence_id), enc_buffer, &[2][..], stream).await?;

          fetch_msg(net_buffer, sequence_id, stream).await?;
          //let rsa_pub_key = net_buffer._current().get(1..).unwrap_or_default();

          Self::xor_slice((&auth_plugin_data.0, auth_plugin_data.1), &mut pw_array);

          //let pkey = RsaPublicKey::from_public_key_pem(std::str::from_utf8(rsa_pub_key)?)?;
          //let padding = Oaep::new::<sha1::Sha1>();
          //pkey.encrypt(&mut thread_rng(), padding, &pass[..]).map_err(Error::protocol);

          Ok(false)
        }
        _ => panic!(),
      },
      _ => panic!(),
    }
  }

  #[inline]
  pub(crate) fn mask_pw(
    self,
    auth_plugin_data: (&[u8], &[u8]),
    pw: &[u8],
  ) -> crate::Result<ArrayVector<u8, 32>> {
    match self {
      AuthPlugin::CachingSha2 => Ok(Self::sha2_mask(auth_plugin_data, pw).as_slice().try_into()?),
      AuthPlugin::MySqlClear => {
        let mut rslt: ArrayVector<u8, 32> = pw.try_into()?;
        rslt.push(0)?;
        Ok(rslt)
      }
    }
  }

  #[inline]
  fn sha2_mask(data: (&[u8], &[u8]), pw: &[u8]) -> [u8; 32] {
    let mut ctx = Sha256::new();
    ctx.update(pw);
    let mut hash = ctx.finalize_reset();
    ctx.update(hash);
    let another_hash = ctx.finalize_reset();
    ctx.update(data.0);
    ctx.update(data.1);
    ctx.update(another_hash);
    let with_seed_hash = ctx.finalize();
    Self::xor_slice((&with_seed_hash, &[]), &mut hash);
    hash.into()
  }

  #[inline]
  fn xor_slice((from0, from1): (&[u8], &[u8]), to: &mut [u8]) {
    let from_iter = from0.iter().chain(from1).cycle();
    for (to, from) in to.iter_mut().zip(from_iter) {
      *to ^= *from;
    }
  }
}

impl From<AuthPlugin> for &'static str {
  #[inline]
  fn from(from: AuthPlugin) -> Self {
    match from {
      AuthPlugin::CachingSha2 => "caching_sha2_password",
      AuthPlugin::MySqlClear => "mysql_clear_password",
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
      _ => return Err(MysqlError::UnknownAuthPlugin.into()),
    })
  }
}
