use crate::{
  database::client::mysql::{
    MysqlError,
    misc::{fetch_msg, write_packet},
  },
  misc::{
    ArrayVector, Stream, Vector, from_utf8_basic,
    partitioned_filled_buffer::PartitionedFilledBuffer,
  },
};
use digest::{Digest, FixedOutputReset, Update, generic_array::GenericArray};
use rsa::{Oaep, RsaPublicKey, pkcs8::DecodePublicKey};

#[derive(Clone, Copy, Debug)]
pub(crate) enum AuthPlugin {
  CachingSha2,
  MysqlClear,
  MysqlNativePassword,
}

impl AuthPlugin {
  #[inline]
  pub(crate) async fn manage_caching_sha2<E, S, const IS_TLS: bool>(
    self,
    auth_plugin_data: ([u8; 8], &[u8]),
    [a, b]: [u8; 2],
    (capabilities, sequence_id): (&mut u64, &mut u8),
    encode_buffer: &mut Vector<u8>,
    net_buffer: &mut PartitionedFilledBuffer,
    password: &str,
    stream: &mut S,
  ) -> Result<bool, E>
  where
    E: From<crate::Error>,
    S: Stream,
  {
    match self {
      AuthPlugin::CachingSha2 if a == 1 => match b {
        3 => Ok(true),
        4 => {
          let mut pw_array: ArrayVector<u8, 32> = password.as_bytes().try_into()?;
          pw_array.push(b'\0')?;
          if IS_TLS {
            write_packet((capabilities, sequence_id), encode_buffer, &pw_array[..], stream).await?;
            return Ok(false);
          }

          write_packet((capabilities, sequence_id), encode_buffer, &[2][..], stream).await?;

          let _ = fetch_msg(*capabilities, net_buffer, sequence_id, stream).await?;
          let rsa_pub_key = net_buffer._current().get(1..).unwrap_or_default();

          Self::xor_slice((&auth_plugin_data.0, auth_plugin_data.1), &mut pw_array);

          let pkey = RsaPublicKey::from_public_key_pem(
            from_utf8_basic(rsa_pub_key).map_err(crate::Error::from)?,
          )
          .map_err(crate::Error::from)?;
          let padding = Oaep::new::<sha1::Sha1>();
          let bytes = pkey
            .encrypt(&mut rand_0_8::rngs::OsRng, padding, &pw_array)
            .map_err(crate::Error::from)?;
          let payload = bytes.as_slice();
          write_packet((capabilities, sequence_id), encode_buffer, payload, stream).await?;

          Ok(false)
        }
        _ => return Err(E::from(MysqlError::InvalidAuthPluginBytes.into())),
      },
      _ => return Err(E::from(MysqlError::InvalidAuthPluginBytes.into())),
    }
  }

  #[inline]
  pub(crate) fn mask_pw(
    self,
    auth_plugin_data: (&[u8], &[u8]),
    pw: &[u8],
  ) -> crate::Result<ArrayVector<u8, 32>> {
    match self {
      AuthPlugin::CachingSha2 => {
        Ok(Self::mask(sha2::Sha256::new(), auth_plugin_data, pw).as_slice().try_into()?)
      }
      AuthPlugin::MysqlNativePassword => {
        Ok(Self::mask(sha1::Sha1::new(), auth_plugin_data, pw).as_slice().try_into()?)
      }
      AuthPlugin::MysqlClear => {
        let mut rslt: ArrayVector<u8, 32> = pw.try_into()?;
        rslt.push(0)?;
        Ok(rslt)
      }
    }
  }

  #[inline]
  fn mask<T, const N: usize>(mut ctx: T, data: (&[u8], &[u8]), pw: &[u8]) -> [u8; N]
  where
    T: Digest + FixedOutputReset,
    [u8; N]: From<GenericArray<u8, <T as digest::OutputSizeUser>::OutputSize>>,
  {
    Update::update(&mut ctx, pw);
    let mut hash = ctx.finalize_reset();
    Update::update(&mut ctx, hash.as_ref());
    let another_hash = ctx.finalize_reset();
    Update::update(&mut ctx, data.0);
    Update::update(&mut ctx, data.1);
    Update::update(&mut ctx, another_hash.as_ref());
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
      AuthPlugin::MysqlClear => "mysql_clear_password",
      AuthPlugin::MysqlNativePassword => "mysql_native_password",
    }
  }
}

impl TryFrom<&[u8]> for AuthPlugin {
  type Error = crate::Error;

  #[inline]
  fn try_from(from: &[u8]) -> Result<Self, Self::Error> {
    Ok(match from {
      b"caching_sha2_password" => AuthPlugin::CachingSha2,
      b"mysql_clear_password" => AuthPlugin::MysqlClear,
      b"mysql_native_password" => AuthPlugin::MysqlNativePassword,
      _ => return Err(MysqlError::UnknownAuthPlugin.into()),
    })
  }
}
